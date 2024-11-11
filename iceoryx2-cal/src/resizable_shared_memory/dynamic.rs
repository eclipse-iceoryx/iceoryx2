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

use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{fmt::Debug, marker::PhantomData};

use crate::shared_memory::ShmPointer;
use crate::shared_memory::{
    PointerOffset, SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError,
    SharedMemoryOpenError, ShmAllocator,
};
use crate::shm_allocator::ShmAllocationError;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

use super::{
    NamedConcept, NamedConceptBuilder, NamedConceptDoesExistError, NamedConceptListError,
    NamedConceptMgmt, NamedConceptRemoveError, ResizableSharedMemory, ResizableSharedMemoryBuilder,
    ResizableSharedMemoryView, MAX_DATASEGMENTS,
};

#[derive(Debug)]
pub struct DynamicBuilder<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
where
    Allocator: Debug,
{
    base_name: FileName,
    builder: Shm::Builder,
    allocator_config_hint: Allocator::Configuration,
    max_number_of_chunks_hint: usize,
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
            builder: Shm::Builder::new(name),
            allocator_config_hint: Allocator::Configuration::default(),
            max_number_of_chunks_hint: 1,
            base_name: *name,
        }
    }

    fn config(mut self, config: &Shm::Configuration) -> Self {
        self.builder = self.builder.config(config);
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
        self.builder = self.builder.has_ownership(value);
        self
    }

    fn allocator_config_hint(mut self, value: Allocator::Configuration) -> Self {
        self.allocator_config_hint = value;
        self
    }

    fn max_number_of_chunks_hint(mut self, value: usize) -> Self {
        self.max_number_of_chunks_hint = value;
        self
    }

    fn timeout(mut self, value: Duration) -> Self {
        self.builder = self.builder.timeout(value);
        self
    }

    fn create(self) -> Result<DynamicMemory<Allocator, Shm>, SharedMemoryCreateError> {
        let initial_size = Allocator::payload_size_hint(
            &self.allocator_config_hint,
            self.max_number_of_chunks_hint,
        );

        let origin = format!("{:?}", self);
        let shm = fail!(from origin, when self
            .builder
            .size(initial_size)
            .create(&self.allocator_config_hint),
            "Unable to create ResizableSharedMemory since the underlying shared memory could not be created.");

        let mut shared_memory_map = SlotMap::new(MAX_DATASEGMENTS);
        let current_idx = shared_memory_map
            .insert(shm)
            .expect("MAX_DATASEGMENTS is greater or equal 1");

        Ok(DynamicMemory {
            base_name: self.base_name,
            shared_memory_map,
            current_idx,
            has_ownership: IoxAtomicBool::new(true),
            _data: PhantomData,
        })
    }

    fn open(self) -> Result<DynamicView<Allocator, Shm>, SharedMemoryOpenError> {
        let origin = format!("{:?}", self);
        let shm = fail!(from origin, when self
            .builder
            .open(),
            "Unable to open ResizableSharedMemoryView since the underlying shared memory could not be opened.");

        let mut shared_memory_map = SlotMap::new(MAX_DATASEGMENTS);
        let current_idx = shared_memory_map
            .insert(shm)
            .expect("MAX_DATASEGMENTS is greater or equal 1");

        Ok(DynamicView {
            base_name: self.base_name,
            shared_memory_map,
            current_idx,
            _data: PhantomData,
        })
    }
}

pub struct DynamicView<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    base_name: FileName,
    shared_memory_map: SlotMap<Shm>,
    current_idx: SlotMapKey,
    _data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> DynamicView<Allocator, Shm> {
    fn update_map_view(&mut self) {}
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    ResizableSharedMemoryView<Allocator, Shm> for DynamicView<Allocator, Shm>
{
    fn translate_offset(&mut self, offset: PointerOffset) -> *const u8 {
        let segment_id = offset.segment_id();
        let offset = offset.offset();

        match self
            .shared_memory_map
            .get(SlotMapKey::new(segment_id as usize))
        {
            None => {
                self.update_map_view();
                todo!()
            }
            Some(shm) => (offset + shm.payload_start_address()) as *const u8,
        }
    }
}

#[derive(Debug)]
pub struct DynamicMemory<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    base_name: FileName,
    shared_memory_map: SlotMap<Shm>,
    current_idx: SlotMapKey,
    has_ownership: IoxAtomicBool,
    _data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> NamedConcept
    for DynamicMemory<Allocator, Shm>
{
    fn name(&self) -> &FileName {
        &self.base_name
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

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> ResizableSharedMemory<Allocator, Shm>
    for DynamicMemory<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    type Builder = DynamicBuilder<Allocator, Shm>;
    type View = DynamicView<Allocator, Shm>;

    fn allocate(&self, layout: std::alloc::Layout) -> Result<ShmPointer, ShmAllocationError> {
        match self.shared_memory_map.get(self.current_idx) {
            Some(shm) => shm.allocate(layout),
            None => fatal_panic!(from self,
                        "This should never happen! Current shared memory segment is not available!"),
        }
    }

    unsafe fn deallocate(&self, offset: PointerOffset, layout: std::alloc::Layout) {
        match self.shared_memory_map.get(self.current_idx) {
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
