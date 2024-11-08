// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

use std::alloc::Layout;
use std::fmt::Debug;
use std::sync::atomic::Ordering;
use std::time::Duration;

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};
use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::shared_memory::{
    SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError, SharedMemoryOpenError, ShmPointer,
};
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::shm_allocator::{PointerOffset, ShmAllocationError};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

pub struct DynamicPointerOffset {
    value: u64,
}

impl DynamicPointerOffset {
    pub fn new(offset: usize, segment_id: u8) -> Self {
        Self {
            value: (offset as u64) << 8 | segment_id as u64,
        }
    }

    pub fn offset(&self) -> PointerOffset {
        PointerOffset::new((self.value >> 8) as usize)
    }

    pub fn segment_id(&self) -> u8 {
        (self.value & 0x00000000000000ff) as u8
    }
}

const MAX_DATASEGMENTS: usize = 256;

#[derive(Default)]
pub enum AllocationStrategy {
    #[default]
    PowerOfTwo,
    Static,
}

#[derive(Debug)]
pub struct ResizableSharedMemoryBuilder<T: SharedMemory<PoolAllocator>>
where
    T::Builder: Debug,
{
    base_name: FileName,
    builder: T::Builder,
    bucket_layout_hint: Layout,
    number_of_buckets_hint: usize,
}

impl<T: SharedMemory<PoolAllocator>> ResizableSharedMemoryBuilder<T>
where
    T::Builder: Debug,
{
    pub fn new(name: &FileName) -> Self {
        let mut first_shm_segment = *name;
        first_shm_segment
            .push_bytes(b"__0")
            .expect("Adding __0 results in a valid file name");
        Self {
            builder: T::Builder::new(name),
            base_name: *name,
            bucket_layout_hint: Layout::new::<u8>(),
            number_of_buckets_hint: 1,
        }
    }

    pub fn config(mut self, config: &T::Configuration) -> Self {
        self.builder = self.builder.config(config);
        self
    }

    /// Defines if a newly created [`SharedMemory`] owns the underlying resources
    pub fn has_ownership(mut self, value: bool) -> Self {
        self.builder = self.builder.has_ownership(value);
        self
    }

    pub fn bucket_layout_hint(mut self, layout: Layout) -> Self {
        self.bucket_layout_hint = layout;
        self
    }

    pub fn number_of_buckets_hint(mut self, value: usize) -> Self {
        self.number_of_buckets_hint = value.max(1);
        self
    }

    /// The timeout defines how long the [`SharedMemoryBuilder`] should wait for
    /// [`SharedMemoryBuilder::create()`] to finialize
    /// the initialization. This is required when the [`SharedMemory`] is created and initialized
    /// concurrently from another process. By default it is set to [`Duration::ZERO`] for no
    /// timeout.
    pub fn timeout(mut self, value: Duration) -> Self {
        self.builder = self.builder.timeout(value);
        self
    }

    /// Creates new [`SharedMemory`]. If it already exists the method will fail.
    pub fn create(self) -> Result<ResizableSharedMemory<T>, SharedMemoryCreateError> {
        let initial_size = self.number_of_buckets_hint * self.bucket_layout_hint.size();
        let initial_allocator_config = iceoryx2_cal::shm_allocator::pool_allocator::Config {
            bucket_layout: self.bucket_layout_hint,
        };

        let origin = format!("{:?}", self);
        let shm = fail!(from origin, when self
            .builder
            .size(initial_size)
            .create(&initial_allocator_config),
            "Unable to create ResizableSharedMemory since the underlying shared memory could not be created.");

        let mut shared_memory_map = SlotMap::new(MAX_DATASEGMENTS);
        let current_idx = shared_memory_map
            .insert(shm)
            .expect("MAX_DATASEGMENTS is greater or equal 1");

        Ok(ResizableSharedMemory {
            base_name: self.base_name,
            shared_memory_map,
            current_idx,
            number_of_buckets: self.number_of_buckets_hint,
            bucket_layout: self.bucket_layout_hint,
            has_ownership: IoxAtomicBool::new(true),
        })
    }

    /// Opens already existing [`SharedMemory`]. If it does not exist or the initialization is not
    /// yet finished the method will fail.
    pub fn open(self) -> Result<ResizableSharedMemoryView<T>, SharedMemoryOpenError> {
        let origin = format!("{:?}", self);
        let shm = fail!(from origin, when self
            .builder
            .open(),
            "Unable to open ResizableSharedMemoryView since the underlying shared memory could not be opened.");

        let mut shared_memory_map = SlotMap::new(MAX_DATASEGMENTS);
        let current_idx = shared_memory_map
            .insert(shm)
            .expect("MAX_DATASEGMENTS is greater or equal 1");

        Ok(ResizableSharedMemoryView {
            base_name: self.base_name,
            shared_memory_map,
            current_idx,
        })
    }
}

pub struct ResizableSharedMemoryView<T: SharedMemory<PoolAllocator>> {
    base_name: FileName,
    shared_memory_map: SlotMap<T>,
    current_idx: SlotMapKey,
}

impl<T: SharedMemory<PoolAllocator>> ResizableSharedMemoryView<T> {
    fn update_map_view(&mut self) {}

    pub fn translate_offset(&mut self, offset: DynamicPointerOffset) -> usize {
        let segment_id = offset.segment_id();
        let offset = offset.offset().value();

        match self
            .shared_memory_map
            .get(SlotMapKey::new(segment_id as usize))
        {
            None => {
                self.update_map_view();
                todo!()
            }
            Some(shm) => offset + shm.payload_start_address(),
        }
    }
}

pub struct ResizableSharedMemory<T: SharedMemory<PoolAllocator>> {
    base_name: FileName,
    shared_memory_map: SlotMap<T>,
    current_idx: SlotMapKey,
    number_of_buckets: usize,
    bucket_layout: Layout,
    has_ownership: IoxAtomicBool,
}

impl<T: SharedMemory<PoolAllocator>> ResizableSharedMemory<T> {
    unsafe fn remove(
        name: &FileName,
        config: &T::Configuration,
    ) -> Result<bool, NamedConceptRemoveError> {
        todo!()
    }

    fn does_exist(
        name: &FileName,
        config: &T::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError> {
        todo!()
    }

    fn list<F: FnMut(&FileName) -> CallbackProgression>(
        config: &T::Configuration,
        callback: F,
    ) -> Result<(), NamedConceptListError> {
        todo!()
    }

    fn allocate(&self, layout: std::alloc::Layout) -> Result<ShmPointer, ShmAllocationError> {
        todo!()
    }

    /// Release previously allocated memory
    ///
    /// # Safety
    ///
    ///  * the offset must be acquired with [`SharedMemory::allocate()`] - extracted from the
    ///    [`ShmPointer`]
    ///  * the layout must be identical to the one used in [`SharedMemory::allocate()`]
    unsafe fn deallocate(&self, offset: PointerOffset, layout: std::alloc::Layout) {
        todo!()
    }

    /// Returns if the [`SharedMemory`] supports persistency, meaning that the underlying OS
    /// resource remain even when every [`SharedMemory`] instance in every process was removed.
    pub fn does_support_persistency() -> bool {
        T::does_support_persistency()
    }

    /// Returns true if the [`SharedMemory`] holds the ownership, otherwise false
    pub fn has_ownership(&self) -> bool {
        self.has_ownership.load(Ordering::Relaxed)
    }

    /// Acquires the ownership of the [`SharedMemory`]. When the object goes out of scope the
    /// underlying resources will be removed.
    pub fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
    }

    /// Releases the ownership of the [`SharedMemory`] meaning when it goes out of scope the
    /// underlying resource will not be removed.
    pub fn release_ownership(&self) {
        self.has_ownership.store(false, Ordering::Relaxed);
    }
}
