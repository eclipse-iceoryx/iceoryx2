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
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{fmt::Debug, marker::PhantomData};

use crate::shared_memory::{AllocationStrategy, SegmentId, ShmPointer};
use crate::shared_memory::{
    PointerOffset, SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError,
    SharedMemoryOpenError, ShmAllocator,
};
use crate::shm_allocator::ShmAllocationError;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};
use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU64};

use super::{
    NamedConcept, NamedConceptBuilder, NamedConceptDoesExistError, NamedConceptListError,
    NamedConceptMgmt, NamedConceptRemoveError, ResizableSharedMemory, ResizableSharedMemoryBuilder,
    ResizableSharedMemoryView, ResizableShmAllocationError,
};

const MAX_NUMBER_OF_REALLOCATIONS: usize = SegmentId::max_segment_id() as usize + 1;
const SEGMENT_ID_SEPARATOR: &[u8] = b"__";

#[derive(Debug)]
struct BuilderConfig<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    base_name: FileName,
    shm: Shm::Configuration,
    allocator_config_hint: Allocator::Configuration,
    allocation_strategy: AllocationStrategy,
    has_ownership: bool,
    shm_builder_timeout: Duration,
    max_number_of_chunks_hint: usize,
    max_chunk_layout_hint: Layout,
}

#[derive(Debug)]
struct InternalState<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    builder_config: BuilderConfig<Allocator, Shm>,
    shared_memory_map: SlotMap<ShmEntry<Allocator, Shm>>,
    current_idx: SlotMapKey,
}

#[derive(Debug)]
struct ShmEntry<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    shm: Shm,
    chunk_count: IoxAtomicU64,
    _data: PhantomData<Allocator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShmEntryState {
    Empty,
    NonEmpty,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> ShmEntry<Allocator, Shm> {
    fn new(shm: Shm) -> Self {
        Self {
            shm,
            chunk_count: IoxAtomicU64::new(0),
            _data: PhantomData,
        }
    }

    fn register_offset(&self) {
        self.chunk_count.fetch_add(1, Ordering::Relaxed);
    }

    fn unregister_offset(&self) -> ShmEntryState {
        match self.chunk_count.fetch_sub(1, Ordering::Relaxed) {
            1 => ShmEntryState::Empty,
            _ => ShmEntryState::NonEmpty,
        }
    }
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
where
    Shm::Builder: Debug,
{
    fn new(name: &FileName) -> Self {
        let mut first_shm_segment = *name;
        first_shm_segment
            .push_bytes(SEGMENT_ID_SEPARATOR)
            .expect("Adding the segment id separator results in a valid file name.");
        first_shm_segment
            .push_bytes(b"0")
            .expect("Adding the segment id results in a valid file name");
        Self {
            config: BuilderConfig {
                base_name: *name,
                has_ownership: true,
                allocation_strategy: AllocationStrategy::default(),
                shm_builder_timeout: Duration::ZERO,
                allocator_config_hint: Allocator::Configuration::default(),
                shm: Shm::Configuration::default(),
                max_number_of_chunks_hint: 1,
                max_chunk_layout_hint: Layout::new::<u8>(),
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

    fn max_chunk_layout_hint(mut self, value: Layout) -> Self {
        self.config.max_chunk_layout_hint = value;
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

    fn create(mut self) -> Result<DynamicMemory<Allocator, Shm>, SharedMemoryCreateError> {
        let hint = Allocator::initial_setup_hint(
            self.config.max_chunk_layout_hint,
            self.config.max_number_of_chunks_hint,
        );
        self.config.allocator_config_hint = hint.config;

        let origin = format!("{:?}", self);
        let shm = fail!(from origin, when DynamicMemory::create_segment(&self.config, SegmentId::new(0), hint.payload_size),
            "Unable to create ResizableSharedMemory since the underlying shared memory could not be created.");

        let mut shared_memory_map = SlotMap::new(MAX_NUMBER_OF_REALLOCATIONS);
        let current_idx = shared_memory_map
            .insert(ShmEntry::new(shm))
            .expect("MAX_NUMBER_OF_REALLOCATIONS is greater or equal 1");

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
        let shm = fail!(from origin, when DynamicMemory::open_segment(&self.config, SegmentId::new(0)),
            "Unable to open ResizableSharedMemoryView since the underlying shared memory could not be opened.");

        let mut shared_memory_map = SlotMap::new(MAX_NUMBER_OF_REALLOCATIONS);
        let current_idx = shared_memory_map
            .insert(ShmEntry::new(shm))
            .expect("MAX_NUMBER_OF_REALLOCATIONS is greater or equal 1");

        Ok(DynamicView {
            builder_config: self.config,
            shared_memory_map,
            current_idx,
            _data: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct DynamicView<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    builder_config: BuilderConfig<Allocator, Shm>,
    shared_memory_map: SlotMap<ShmEntry<Allocator, Shm>>,
    current_idx: SlotMapKey,
    _data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    ResizableSharedMemoryView<Allocator, Shm> for DynamicView<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    fn register_and_translate_offset(
        &mut self,
        offset: PointerOffset,
    ) -> Result<*const u8, SharedMemoryOpenError> {
        let msg = "Unable to translate";
        let segment_id = offset.segment_id();
        let offset = offset.offset();
        let key = SlotMapKey::new(segment_id.value() as usize);

        let payload_start_address = match self.shared_memory_map.get(key) {
            None => {
                let shm = fail!(from self,
                                when DynamicMemory::open_segment(&self.builder_config, segment_id),
                                "{msg} {:?} since the corresponding shared memory segment could not be opened.", offset);
                let payload_start_address = shm.payload_start_address();
                let entry = ShmEntry::new(shm);
                entry.register_offset();
                self.shared_memory_map.insert_at(key, entry);
                self.current_idx = key;
                payload_start_address
            }
            Some(entry) => {
                entry.register_offset();
                entry.shm.payload_start_address()
            }
        };

        println!(
            "start address of segment {:?}: {}",
            segment_id, payload_start_address
        );

        Ok((offset + payload_start_address) as *const u8)
    }

    fn unregister_offset(&mut self, offset: PointerOffset) {
        let segment_id = offset.segment_id();
        let key = SlotMapKey::new(segment_id.value() as usize);

        match self.shared_memory_map.get(key) {
            Some(entry) => {
                if entry.unregister_offset() == ShmEntryState::Empty && self.current_idx != key {
                    self.shared_memory_map.remove(key);
                }
            }
            None => {
                warn!(from self,
                      "Unable to unregister offset {:?} since the segment id is not mapped.", offset);
            }
        }
    }

    fn number_of_active_segments(&self) -> usize {
        self.shared_memory_map.len()
    }
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
where
    Shm::Builder: Debug,
{
    type Configuration = Shm::Configuration;

    unsafe fn remove_cfg(
        name: &FileName,
        config: &Shm::Configuration,
    ) -> Result<bool, NamedConceptRemoveError> {
        let origin = "resizable_shared_memory::Dynamic::remove_cfg()";
        let msg = format!("Unable to remove ResizableSharedMemory {:?}", name);
        let raw_names = match Shm::list_cfg(config) {
            Ok(names) => names,
            Err(NamedConceptListError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptRemoveError::InsufficientPermissions,
                    "{msg} due to insufficient permissions while listing the underlying SharedMemories.");
            }
            Err(e) => {
                fail!(from origin, with NamedConceptRemoveError::InsufficientPermissions,
                    "{msg} due to an internal error ({:?}) while listing the underlying SharedMemories.", e);
            }
        };

        let mut shm_removed = false;
        for raw_name in &raw_names {
            if let Some((extracted_name, _)) = Self::extract_name_and_segment_id(raw_name) {
                if *name == extracted_name {
                    fail!(from origin, when Shm::remove_cfg(raw_name, config),
                        "{msg} since the underlying SharedMemory could not be removed.");
                    shm_removed = true;
                }
            }
        }

        Ok(shm_removed)
    }

    fn does_exist_cfg(
        name: &FileName,
        config: &Shm::Configuration,
    ) -> Result<bool, NamedConceptDoesExistError> {
        let origin = "resizable_shared_memory::Dynamic::does_exist_cfg()";
        let msg = format!(
            "Unable to determine if ResizableSharedMemory {:?} exists",
            name
        );

        let names = match Self::list_cfg(config) {
            Ok(names) => names,
            Err(NamedConceptListError::InsufficientPermissions) => {
                fail!(from origin, with NamedConceptDoesExistError::InsufficientPermissions,
                    "{msg} due to insufficient permissions while acquiring a list of all ResizableSharedMemories.");
            }
            Err(e) => {
                fail!(from origin, with NamedConceptDoesExistError::InternalError,
                    "{msg} due to an internal error ({:?}) while acquiring a list of all ResizableSharedMemories.", e);
            }
        };

        Ok(names.iter().find(|n| *n == name).is_some())
    }

    fn list_cfg(config: &Shm::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        let origin = "resizable_shared_memory::Dynamic::list_cfg()";
        let mut names = HashSet::new();
        let raw_names = fail!(from origin, when Shm::list_cfg(config),
                            "Unable to list ResizableSharedMemories since the underlying SharedMemories could not be listed.");

        for raw_name in &raw_names {
            if let Some((name, _)) = Self::extract_name_and_segment_id(raw_name) {
                names.insert(name);
            }
        }

        Ok(Vec::from_iter(names.into_iter()))
    }

    fn remove_path_hint(value: &Path) -> Result<(), super::NamedConceptPathHintRemoveError> {
        Shm::remove_path_hint(value)
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> DynamicMemory<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    fn extract_name_and_segment_id(name: &FileName) -> Option<(FileName, SegmentId)> {
        if let Some(pos) = name.rfind(SEGMENT_ID_SEPARATOR) {
            let segment_id_start_pos = pos + SEGMENT_ID_SEPARATOR.len();
            if name.len() < segment_id_start_pos {
                return None;
            }

            let number_of_segment_id_digits =
                SegmentId::max_segment_id().checked_ilog10().unwrap_or(0) + 1;

            if name.len() > segment_id_start_pos + number_of_segment_id_digits as usize {
                return None;
            }

            let mut raw_segment_id = name.as_string().clone();
            raw_segment_id.remove_range(0, segment_id_start_pos);

            // check nymber of digits
            for byte in raw_segment_id.as_bytes() {
                let is_a_digit = b'0' <= *byte && *byte <= b'9';
                if !is_a_digit {
                    return None;
                }
            }

            let segment_id_value = String::from_utf8_lossy(raw_segment_id.as_bytes())
                .parse::<u64>()
                .expect("Contains a valid u64 integer.");

            if segment_id_value > SegmentId::max_segment_id() as u64 {
                return None;
            }

            let mut name = name.clone();
            name.remove_range(pos, name.len() - pos)
                .expect("Is a valid file name without segment id suffix");

            return Some((name, SegmentId::new(segment_id_value as u8)));
        }

        None
    }

    fn state_mut(&self) -> &mut InternalState<Allocator, Shm> {
        unsafe { &mut *self.state.get() }
    }

    fn state(&self) -> &InternalState<Allocator, Shm> {
        unsafe { &*self.state.get() }
    }

    fn create_segment(
        config: &BuilderConfig<Allocator, Shm>,
        segment_id: SegmentId,
        payload_size: usize,
    ) -> Result<Shm, SharedMemoryCreateError> {
        Self::segment_builder(config, segment_id)
            .size(payload_size)
            .create(&config.allocator_config_hint)
    }

    fn open_segment(
        config: &BuilderConfig<Allocator, Shm>,
        segment_id: SegmentId,
    ) -> Result<Shm, SharedMemoryOpenError> {
        Self::segment_builder(config, segment_id).open()
    }

    fn segment_builder(
        config: &BuilderConfig<Allocator, Shm>,
        segment_id: SegmentId,
    ) -> Shm::Builder {
        let msg = "This should never happen! Unable to create additional shared memory segment since it would result in an invalid shared memory name.";
        let mut adjusted_name = config.base_name;
        fatal_panic!(from config, when adjusted_name.push_bytes(SEGMENT_ID_SEPARATOR), "{msg}");
        fatal_panic!(from config, when adjusted_name.push_bytes(segment_id.value().to_string().as_bytes()), "{msg}");
        Shm::Builder::new(&adjusted_name)
            .has_ownership(config.has_ownership)
            .timeout(config.shm_builder_timeout)
            .config(&config.shm)
    }

    fn create_resized_segment(
        &self,
        shm: &Shm,
        layout: Layout,
    ) -> Result<(), SharedMemoryCreateError> {
        let msg = "Unable to create resized segment for";
        let state = self.state_mut();
        let adjusted_segment_setup = shm
            .allocator()
            .resize_hint(layout, state.builder_config.allocation_strategy);
        let segment_id = if state.current_idx.value() < MAX_NUMBER_OF_REALLOCATIONS {
            SlotMapKey::new(state.current_idx.value() + 1)
        } else {
            fail!(from self, with SharedMemoryCreateError::InternalError,
                "{msg} {:?} since it would exceed the maximum amount of reallocations of {}. With a better configuration hint, this issue can be avoided.",
                layout, Self::max_number_of_reallocations());
        };

        state.builder_config.allocator_config_hint = adjusted_segment_setup.config;
        let shm = Self::create_segment(
            &state.builder_config,
            SegmentId::new(segment_id.value() as u8),
            adjusted_segment_setup.payload_size,
        )?;

        match state.shared_memory_map.get(state.current_idx) {
            Some(ref segment) => {
                if segment.chunk_count.load(Ordering::Relaxed) == 0 {
                    state.shared_memory_map.remove(state.current_idx);
                }
            }
            None => {
                fatal_panic!(from self,
                        "This should never happen! {msg} {:?} since the current segment id is unavailable.",
                        layout)
            }
        }

        state
            .shared_memory_map
            .insert_at(segment_id, ShmEntry::new(shm));
        state.current_idx = segment_id;

        Ok(())
    }

    fn handle_reallocation(
        &self,
        e: ShmAllocationError,
        state: &InternalState<Allocator, Shm>,
        layout: Layout,
        shm: &Shm,
    ) -> Result<(), ResizableShmAllocationError> {
        let msg = "Unable to allocate memory";
        if e == ShmAllocationError::AllocationError(AllocationError::OutOfMemory)
            || e == ShmAllocationError::ExceedsMaxSupportedAlignment
            || e == ShmAllocationError::AllocationError(AllocationError::SizeTooLarge)
        {
            if state.builder_config.allocation_strategy == AllocationStrategy::Static {
                fail!(from self, with e.into(),
                                    "{msg} since there is not enough memory left ({:?}) and the allocation strategy {:?} forbids reallocation.",
                                    e, state.builder_config.allocation_strategy);
            } else {
                self.create_resized_segment(shm, layout)?;
                Ok(())
            }
        } else {
            fail!(from self, with e.into(), "{msg} due to {:?}.", e);
        }
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> ResizableSharedMemory<Allocator, Shm>
    for DynamicMemory<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    type Builder = DynamicBuilder<Allocator, Shm>;
    type View = DynamicView<Allocator, Shm>;

    fn max_number_of_reallocations() -> usize {
        MAX_NUMBER_OF_REALLOCATIONS
    }

    fn number_of_active_segments(&self) -> usize {
        self.state().shared_memory_map.len()
    }

    fn allocate(&self, layout: Layout) -> Result<ShmPointer, ResizableShmAllocationError> {
        let msg = "Unable to allocate memory";
        let state = self.state_mut();

        loop {
            match state.shared_memory_map.get(state.current_idx) {
                Some(ref entry) => match entry.shm.allocate(layout) {
                    Ok(mut ptr) => {
                        entry.register_offset();
                        ptr.offset
                            .set_segment_id(SegmentId::new(state.current_idx.value() as u8));
                        return Ok(ptr);
                    }
                    Err(e) => self.handle_reallocation(e, state, layout, &entry.shm)?,
                },
                None => fatal_panic!(from self,
                        "This should never happen! {msg} since the current shared memory segment is not available!"),
            }
        }
    }

    unsafe fn deallocate(&self, offset: PointerOffset, layout: Layout) {
        let segment_id = SlotMapKey::new(offset.segment_id().value() as usize);
        let state = self.state_mut();
        match state.shared_memory_map.get(segment_id) {
            Some(entry) => {
                entry.shm.deallocate(offset, layout);
                if entry.unregister_offset() == ShmEntryState::Empty
                    && segment_id != state.current_idx
                {
                    state.shared_memory_map.remove(segment_id);
                }
            }
            None => fatal_panic!(from self,
                        "This should never happen! Unable to deallocate {:?} since the corresponding shared memory segment is not available!", offset),
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
