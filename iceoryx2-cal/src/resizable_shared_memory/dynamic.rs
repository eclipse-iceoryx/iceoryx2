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
use std::cell::UnsafeCell;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{fmt::Debug, marker::PhantomData};

use crate::shared_memory::{AllocationStrategy, ShmPointer};
use crate::shared_memory::{
    PointerOffset, SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError,
    SharedMemoryOpenError, ShmAllocator,
};
use crate::shm_allocator::ShmAllocationError;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};
use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

use super::{
    NamedConcept, NamedConceptBuilder, NamedConceptDoesExistError, NamedConceptListError,
    NamedConceptMgmt, NamedConceptRemoveError, ResizableSharedMemory, ResizableSharedMemoryBuilder,
    ResizableSharedMemoryView, ResizableShmAllocationError, MAX_DATASEGMENTS,
};

#[derive(Debug)]
struct BuilderConfig<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    base_name: FileName,
    shm: Shm::Configuration,
    allocator_config_hint: Allocator::Configuration,
    allocation_strategy: AllocationStrategy,
    has_ownership: bool,
    shm_builder_timeout: Duration,
    max_number_of_chunks_hint: usize,
}

#[derive(Debug)]
pub struct DynamicBuilder<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
where
    Allocator: Debug,
{
    config: BuilderConfig<Allocator, Shm>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    NamedConceptBuilder<DynamicMemory<Allocator, Shm>> for DynamicBuilder<Allocator, Shm>
{
    fn new(name: &FileName) -> Self {
        let mut first_shm_segment = *name;
        first_shm_segment
            .push_bytes(b"__0")
            .expect("Adding __0 results in a valid file name");
        Self {
            config: BuilderConfig {
                base_name: *name,
                has_ownership: true,
                allocation_strategy: AllocationStrategy::default(),
                shm_builder_timeout: Duration::ZERO,
                allocator_config_hint: Allocator::Configuration::default(),
                shm: Shm::Configuration::default(),
                max_number_of_chunks_hint: 1,
            },
        }
    }

    fn config(mut self, config: &Shm::Configuration) -> Self {
        self.config.shm = config.clone();
        self
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    ResizableSharedMemoryBuilder<
        Allocator,
        Shm,
        DynamicMemory<Allocator, Shm>,
        DynamicView<Allocator, Shm>,
    > for DynamicBuilder<Allocator, Shm>
where
    Allocator: Debug,
    Shm::Builder: Debug,
{
    fn has_ownership(mut self, value: bool) -> Self {
        self.config.has_ownership = value;
        self
    }

    fn allocator_config_hint(mut self, value: Allocator::Configuration) -> Self {
        self.config.allocator_config_hint = value;
        self
    }

    fn max_number_of_chunks_hint(mut self, value: usize) -> Self {
        self.config.max_number_of_chunks_hint = value;
        self
    }

    fn allocation_strategy(mut self, value: AllocationStrategy) -> Self {
        self.config.allocation_strategy = value;
        self
    }

    fn timeout(mut self, value: Duration) -> Self {
        self.config.shm_builder_timeout = value;
        self
    }

    fn create(self) -> Result<DynamicMemory<Allocator, Shm>, SharedMemoryCreateError> {
        let initial_size = Allocator::payload_size_hint(
            &self.config.allocator_config_hint,
            self.config.max_number_of_chunks_hint,
        );

        let origin = format!("{:?}", self);
        let shm = fail!(from origin, when DynamicMemory::create_segment(&self.config, 0, initial_size),
            "Unable to create ResizableSharedMemory since the underlying shared memory could not be created.");

        let mut shared_memory_map = SlotMap::new(MAX_DATASEGMENTS);
        let current_idx = shared_memory_map
            .insert(shm)
            .expect("MAX_DATASEGMENTS is greater or equal 1");

        Ok(DynamicMemory {
            state: UnsafeCell::new(InternalState {
                builder_config: self.config,
                shared_memory_map,
                current_idx,
            }),
            has_ownership: IoxAtomicBool::new(true),
            _data: PhantomData,
        })
    }

    fn open(self) -> Result<DynamicView<Allocator, Shm>, SharedMemoryOpenError> {
        let origin = format!("{:?}", self);
        let shm = fail!(from origin, when DynamicMemory::open_segment(&self.config, 0),
            "Unable to open ResizableSharedMemoryView since the underlying shared memory could not be opened.");

        let mut shared_memory_map = SlotMap::new(MAX_DATASEGMENTS);
        shared_memory_map
            .insert(shm)
            .expect("MAX_DATASEGMENTS is greater or equal 1");

        Ok(DynamicView {
            builder_config: self.config,
            shared_memory_map,
            _data: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct DynamicView<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    builder_config: BuilderConfig<Allocator, Shm>,
    shared_memory_map: SlotMap<Shm>,
    _data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    ResizableSharedMemoryView<Allocator, Shm> for DynamicView<Allocator, Shm>
{
    fn translate_offset(
        &mut self,
        offset: PointerOffset,
    ) -> Result<*const u8, SharedMemoryOpenError> {
        let msg = "Unable to translate";
        let segment_id = offset.segment_id();
        let offset = offset.offset();
        let key = SlotMapKey::new(segment_id as usize);

        let payload_start_address = match self.shared_memory_map.get(key) {
            None => {
                let shm = fail!(from self,
                                when DynamicMemory::open_segment(&self.builder_config, segment_id),
                                "{msg} {:?} since the corresponding shared memory segment could not be opened.", offset);
                let payload_start_address = shm.payload_start_address();
                self.shared_memory_map.insert_at(key, shm);
                payload_start_address
            }
            Some(shm) => shm.payload_start_address(),
        };

        Ok((offset + payload_start_address) as *const u8)
    }
}

#[derive(Debug)]
struct InternalState<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    builder_config: BuilderConfig<Allocator, Shm>,
    shared_memory_map: SlotMap<Shm>,
    current_idx: SlotMapKey,
}

#[derive(Debug)]
pub struct DynamicMemory<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    state: UnsafeCell<InternalState<Allocator, Shm>>,
    has_ownership: IoxAtomicBool,
    _data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> NamedConcept
    for DynamicMemory<Allocator, Shm>
{
    fn name(&self) -> &FileName {
        unsafe { &(&*self.state.get()).builder_config.base_name }
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> NamedConceptMgmt
    for DynamicMemory<Allocator, Shm>
{
    type Configuration = Shm::Configuration;

    unsafe fn remove_cfg(
        name: &FileName,
        config: &Shm::Configuration,
    ) -> Result<bool, NamedConceptRemoveError> {
        todo!()
    }

    fn does_exist_cfg(
        name: &FileName,
        config: &Shm::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError> {
        todo!()
    }

    fn list_cfg(config: &Shm::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        todo!()
    }

    fn remove_path_hint(value: &Path) -> Result<(), super::NamedConceptPathHintRemoveError> {
        Shm::remove_path_hint(value)
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> DynamicMemory<Allocator, Shm> {
    fn state_mut(&self) -> &mut InternalState<Allocator, Shm> {
        unsafe { &mut *self.state.get() }
    }

    fn create_segment(
        config: &BuilderConfig<Allocator, Shm>,
        segment_id: u8,
        payload_size: usize,
    ) -> Result<Shm, SharedMemoryCreateError> {
        Self::segment_builder(config, segment_id)
            .size(payload_size)
            .create(&config.allocator_config_hint)
    }

    fn open_segment(
        config: &BuilderConfig<Allocator, Shm>,
        segment_id: u8,
    ) -> Result<Shm, SharedMemoryOpenError> {
        Self::segment_builder(config, segment_id).open()
    }

    fn segment_builder(config: &BuilderConfig<Allocator, Shm>, segment_id: u8) -> Shm::Builder {
        let msg = "This should never happen! Unable to create additional shared memory segment since it would result in an invalid shared memory name.";
        let mut adjusted_name = config.base_name;
        fatal_panic!(from config, when adjusted_name.push_bytes(b"__"), "{msg}");
        fatal_panic!(from config, when adjusted_name.push_bytes(segment_id.to_string().as_bytes()), "{msg}");
        Shm::Builder::new(&adjusted_name)
            .has_ownership(config.has_ownership)
            .timeout(config.shm_builder_timeout)
            .config(&config.shm)
    }

    fn create_resized_segment(
        &self,
        shm: &Shm,
        layout: Layout,
        segment_id: u8,
    ) -> Result<(), SharedMemoryCreateError> {
        let state = self.state_mut();
        let adjusted_segment_setup = shm
            .allocator()
            .resize_hint(layout, state.builder_config.allocation_strategy);
        let segment_id = segment_id + 1;

        state.builder_config.allocator_config_hint = adjusted_segment_setup.config;
        let shm = Self::create_segment(
            &state.builder_config,
            segment_id,
            adjusted_segment_setup.payload_size,
        )?;

        let key = SlotMapKey::new(segment_id as usize);
        state.shared_memory_map.insert_at(key, shm);
        state.current_idx = key;

        Ok(())
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> ResizableSharedMemory<Allocator, Shm>
    for DynamicMemory<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    type Builder = DynamicBuilder<Allocator, Shm>;
    type View = DynamicView<Allocator, Shm>;

    fn allocate(&self, layout: Layout) -> Result<ShmPointer, ResizableShmAllocationError> {
        let state = self.state_mut();

        loop {
            match state.shared_memory_map.get(state.current_idx) {
                Some(ref shm) => match shm.allocate(layout) {
                    Ok(mut ptr) => {
                        ptr.offset.set_segment_id(state.current_idx.value() as u8);
                        return Ok(ptr);
                    }
                    Err(ShmAllocationError::AllocationError(AllocationError::OutOfMemory)) => {
                        self.create_resized_segment(
                            shm,
                            layout,
                            state.current_idx.value() as u8 + 1,
                        )?;
                    }
                    Err(e) => return Err(e.into()),
                },
                None => fatal_panic!(from self,
                        "This should never happen! Current shared memory segment is not available!"),
            }
        }
    }

    unsafe fn deallocate(&self, offset: PointerOffset, layout: Layout) {
        let state = self.state_mut();
        match state.shared_memory_map.get(state.current_idx) {
            Some(shm) => shm.deallocate(offset, layout),
            None => fatal_panic!(from self,
                        "This should never happen! Current shared memory segment is not available!"),
        }
    }

    fn does_support_persistency() -> bool {
        Shm::does_support_persistency()
    }

    fn has_ownership(&self) -> bool {
        self.has_ownership.load(Ordering::Relaxed)
    }

    fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
    }

    fn release_ownership(&self) {
        self.has_ownership.store(false, Ordering::Relaxed);
    }
}
