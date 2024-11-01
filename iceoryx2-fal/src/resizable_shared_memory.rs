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
use std::time::Duration;

use iceoryx2_bb_elementary::CallbackProgression;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::named_concept::*;
use iceoryx2_cal::shared_memory::{
    SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError, SharedMemoryOpenError, ShmPointer,
};
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::shm_allocator::{PointerOffset, ShmAllocationError};

#[derive(Debug)]
pub struct UniqueIndexVector<T: Debug> {
    data: Vec<T>,
    idx_to_data: Vec<usize>,
    current_idx: usize,
}

impl<T: Debug> UniqueIndexVector<T> {
    pub fn new() -> Self {
        Self {
            data: vec![],
            idx_to_data: vec![],
            current_idx: 0,
        }
    }

    pub fn push(&mut self, value: T) -> usize {
        todo!()
    }

    pub fn write_at(&mut self, value: T, index: usize) {}

    pub fn remove(&mut self, index: usize) {}

    pub fn get(&self, index: usize) -> &T {
        todo!()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        todo!()
    }
}

impl<T: Debug, const N: usize> From<[T; N]> for UniqueIndexVector<T> {
    fn from(value: [T; N]) -> Self {
        todo!()
    }
}

pub struct DynamicPointerOffset {
    value: u64,
}

impl DynamicPointerOffset {
    pub fn offset(&self) -> PointerOffset {
        todo!()
    }

    pub fn segment_id(&self) -> u8 {
        todo!()
    }

    pub fn has_segment_update(&self) -> u8 {
        todo!()
    }
}

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
    builder: T::Builder,
    bucket_layout_hint: Layout,
    number_of_buckets_hint: usize,
}

impl<T: SharedMemory<PoolAllocator>> ResizableSharedMemoryBuilder<T>
where
    T::Builder: Debug,
{
    pub fn new(name: &FileName) -> Self {
        Self {
            builder: T::Builder::new(name),
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

        Ok(ResizableSharedMemory {
            shared_memory_vec: UniqueIndexVector::from([shm]),
            current_idx: 0,
            number_of_buckets: self.number_of_buckets_hint,
            bucket_layout: self.bucket_layout_hint,
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

        Ok(ResizableSharedMemoryView {
            shared_memory_vec: UniqueIndexVector::from([shm]),
        })
    }
}

pub struct ResizableSharedMemoryView<T: SharedMemory<PoolAllocator>> {
    shared_memory_vec: UniqueIndexVector<T>,
}

impl<T: SharedMemory<PoolAllocator>> ResizableSharedMemoryView<T> {
    fn translate_offset(&self, offset: DynamicPointerOffset) -> usize {
        // updates and remaps shared memories
        //
        todo!()
    }
}

pub struct ResizableSharedMemory<T: SharedMemory<PoolAllocator>> {
    shared_memory_vec: UniqueIndexVector<T>,
    current_idx: u8,
    number_of_buckets: usize,
    bucket_layout: Layout,
}

impl<T: SharedMemory<PoolAllocator>> ResizableSharedMemory<T> {
    fn name(&self) -> &FileName {
        todo!()
    }

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
    fn does_support_persistency() -> bool {
        todo!()
    }

    /// Returns true if the [`SharedMemory`] holds the ownership, otherwise false
    fn has_ownership(&self) -> bool {
        todo!()
    }

    /// Acquires the ownership of the [`SharedMemory`]. When the object goes out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self) {
        todo!()
    }

    /// Releases the ownership of the [`SharedMemory`] meaning when it goes out of scope the
    /// underlying resource will not be removed.
    fn release_ownership(&self) {
        todo!()
    }
}
