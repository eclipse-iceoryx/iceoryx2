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

use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::sync::atomic::Ordering;
use core::time::Duration;
use core::{fmt::Debug, marker::PhantomData};

use crate::shared_memory::{
    AllocationStrategy, SegmentId, SharedMemoryForPoolAllocator, ShmPointer,
};
use crate::shared_memory::{
    PointerOffset, SharedMemory, SharedMemoryBuilder, SharedMemoryCreateError,
    SharedMemoryOpenError, ShmAllocator,
};
use crate::shm_allocator::pool_allocator::PoolAllocator;
use crate::shm_allocator::ShmAllocationError;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_container::slotmap::{SlotMap, SlotMapKey};
use iceoryx2_bb_elementary_traits::allocator::AllocationError;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_bb_log::{fail, warn};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicU64, IoxAtomicUsize};

use super::{
    NamedConcept, NamedConceptBuilder, NamedConceptDoesExistError, NamedConceptListError,
    NamedConceptMgmt, NamedConceptRemoveError, ResizableSharedMemory, ResizableSharedMemoryBuilder,
    ResizableSharedMemoryForPoolAllocator, ResizableSharedMemoryView,
    ResizableSharedMemoryViewBuilder, ResizableShmAllocationError,
};

const MAX_NUMBER_OF_REALLOCATIONS: usize = SegmentId::max_segment_id() as usize + 1;
const SEGMENT_ID_SEPARATOR: &[u8] = b"__";
const MANAGEMENT_SUFFIX: &[u8] = b"mgmt";
const INVALID_KEY: usize = usize::MAX;

#[repr(C)]
#[derive(Debug)]
struct SharedState {
    allocation_strategy: AllocationStrategy,
    max_number_of_chunks_hint: IoxAtomicU64,
    max_chunk_size_hint: IoxAtomicU64,
    max_chunk_alignment_hint: IoxAtomicU64,
}

#[derive(Debug)]
struct MemoryConfig<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    base_name: FileName,
    shm: Shm::Configuration,
    allocator_config_hint: Allocator::Configuration,
}

#[derive(Debug)]
struct ViewConfig<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    base_name: FileName,
    shm: Shm::Configuration,
    shm_builder_timeout: Duration,
    _data: PhantomData<Allocator>,
}

#[derive(Debug)]
struct InternalState<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    builder_config: MemoryConfig<Allocator, Shm>,
    shared_state: SharedState,
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
pub struct DynamicViewBuilder<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    config: ViewConfig<Allocator, Shm>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    NamedConceptBuilder<DynamicMemory<Allocator, Shm>> for DynamicViewBuilder<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    fn new(name: &FileName) -> Self {
        Self {
            config: ViewConfig {
                base_name: name.clone(),
                shm: Shm::Configuration::default(),
                shm_builder_timeout: Duration::ZERO,
                _data: PhantomData,
            },
        }
    }

    fn config(mut self, config: &Shm::Configuration) -> Self {
        self.config.shm = config.clone();
        self
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    ResizableSharedMemoryViewBuilder<
        Allocator,
        Shm,
        DynamicMemory<Allocator, Shm>,
        DynamicView<Allocator, Shm>,
    > for DynamicViewBuilder<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    fn timeout(mut self, value: Duration) -> Self {
        self.config.shm_builder_timeout = value;
        self
    }

    fn open(self) -> Result<DynamicView<Allocator, Shm>, SharedMemoryOpenError> {
        let origin = format!("{self:?}");
        let msg = "Unable to open ResizableSharedMemoryView";

        let adjusted_name =
            DynamicMemory::<Allocator, Shm>::managment_segment_name(&self.config.base_name);
        let mgmt_segment = fail!(from origin, when Shm::Builder::new(&adjusted_name)
                                                        .config(&self.config.shm)
                                                        .has_ownership(false)
                                                        .open(),
                                    "{msg} since the managment segment could not be opened.");

        let shared_memory_map = SlotMap::new(MAX_NUMBER_OF_REALLOCATIONS);

        Ok(DynamicView {
            view_config: self.config,
            _mgmt_segment: mgmt_segment,
            shared_memory_map: UnsafeCell::new(shared_memory_map),
            current_idx: IoxAtomicUsize::new(INVALID_KEY),
            _data: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct DynamicMemoryBuilder<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
where
    Allocator: Debug,
{
    config: MemoryConfig<Allocator, Shm>,
    shared_state: SharedState,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    NamedConceptBuilder<DynamicMemory<Allocator, Shm>> for DynamicMemoryBuilder<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    fn new(name: &FileName) -> Self {
        Self {
            config: MemoryConfig {
                base_name: name.clone(),
                allocator_config_hint: Allocator::Configuration::default(),
                shm: Shm::Configuration::default(),
            },
            shared_state: SharedState {
                allocation_strategy: AllocationStrategy::default(),
                max_number_of_chunks_hint: IoxAtomicU64::new(1),
                max_chunk_size_hint: IoxAtomicU64::new(1),
                max_chunk_alignment_hint: IoxAtomicU64::new(1),
            },
        }
    }

    fn config(mut self, config: &Shm::Configuration) -> Self {
        self.config.shm = config.clone();
        self
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    ResizableSharedMemoryBuilder<Allocator, Shm, DynamicMemory<Allocator, Shm>>
    for DynamicMemoryBuilder<Allocator, Shm>
where
    Allocator: Debug,
    Shm::Builder: Debug,
{
    fn max_chunk_layout_hint(self, value: Layout) -> Self {
        self.shared_state
            .max_chunk_size_hint
            .store(value.size() as u64, Ordering::Relaxed);
        self.shared_state
            .max_chunk_alignment_hint
            .store(value.align() as u64, Ordering::Relaxed);
        self
    }

    fn max_number_of_chunks_hint(self, value: usize) -> Self {
        self.shared_state
            .max_number_of_chunks_hint
            .store(value as u64, Ordering::Relaxed);
        self
    }

    fn allocation_strategy(mut self, value: AllocationStrategy) -> Self {
        self.shared_state.allocation_strategy = value;
        self
    }

    fn create(mut self) -> Result<DynamicMemory<Allocator, Shm>, SharedMemoryCreateError> {
        let msg = "Unable to create ResizableSharedMemory";
        let origin = format!("{self:?}");

        let hint = Allocator::initial_setup_hint(Layout::new::<u8>(), 1);
        let adjusted_name =
            DynamicMemory::<Allocator, Shm>::managment_segment_name(&self.config.base_name);
        let mgmt_segment = fail!(from origin, when Shm::Builder::new(&adjusted_name)
                                                    .size(hint.payload_size)
                                                    .config(&self.config.shm)
                                                    .has_ownership(true)
                                                    .create(&hint.config),
                            "{msg} since the management segment could not be created.");

        let hint = Allocator::initial_setup_hint(
            unsafe {
                Layout::from_size_align_unchecked(
                    self.shared_state
                        .max_chunk_size_hint
                        .load(Ordering::Relaxed) as usize,
                    self.shared_state
                        .max_chunk_alignment_hint
                        .load(Ordering::Relaxed) as usize,
                )
            },
            self.shared_state
                .max_number_of_chunks_hint
                .load(Ordering::Relaxed) as usize,
        );
        self.config.allocator_config_hint = hint.config;

        let shm = fail!(from origin, when DynamicMemory::create_segment(&self.config, SegmentId::new(0), hint.payload_size),
            "Unable to create ResizableSharedMemory since the underlying shared memory could not be created.");
        let mut shared_memory_map = SlotMap::new(MAX_NUMBER_OF_REALLOCATIONS);
        let current_idx = fatal_panic!(from origin, when shared_memory_map.insert(ShmEntry::new(shm)).ok_or(""),
                "This should never happen! {msg} since the newly constructed SlotMap does not have space for one insert.");

        Ok(DynamicMemory {
            state: UnsafeCell::new(InternalState {
                builder_config: self.config,
                shared_memory_map,
                current_idx,
                shared_state: self.shared_state,
            }),
            _mgmt_segment: mgmt_segment,
            _data: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct DynamicView<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    view_config: ViewConfig<Allocator, Shm>,
    _mgmt_segment: Shm,
    shared_memory_map: UnsafeCell<SlotMap<ShmEntry<Allocator, Shm>>>,
    current_idx: IoxAtomicUsize,
    _data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> DynamicView<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    fn release_old_unused_segments(
        shared_memory_map: &mut SlotMap<ShmEntry<Allocator, Shm>>,
        old_idx: usize,
    ) {
        if old_idx == INVALID_KEY {
            return;
        }

        let old_key = SlotMapKey::new(old_idx);
        if let Some(shm) = shared_memory_map.get(old_key) {
            if shm.chunk_count.load(Ordering::Relaxed) == 0 {
                shared_memory_map.remove(old_key);
            }
        }
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>
    ResizableSharedMemoryView<Allocator, Shm> for DynamicView<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    unsafe fn register_and_translate_offset(
        &self,
        offset: PointerOffset,
    ) -> Result<*const u8, SharedMemoryOpenError> {
        let msg = "Unable to translate";
        let segment_id = offset.segment_id();
        let offset = offset.offset();
        let key = SlotMapKey::new(segment_id.value() as usize);
        let shared_memory_map = unsafe { &mut *self.shared_memory_map.get() };

        let payload_start_address = match shared_memory_map.get(key) {
            None => {
                let shm = fail!(from self,
                                when DynamicMemory::open_segment(&self.view_config, segment_id),
                                "{msg} {:?} since the corresponding shared memory segment could not be opened.", offset);
                let payload_start_address = shm.payload_start_address();
                let entry = ShmEntry::new(shm);
                entry.register_offset();
                shared_memory_map.insert_at(key, entry);
                Self::release_old_unused_segments(
                    shared_memory_map,
                    self.current_idx.swap(key.value(), Ordering::Relaxed),
                );

                payload_start_address
            }
            Some(entry) => {
                entry.register_offset();
                entry.shm.payload_start_address()
            }
        };

        Ok((offset + payload_start_address) as *const u8)
    }

    unsafe fn unregister_offset(&self, offset: PointerOffset) {
        let segment_id = offset.segment_id();
        let key = SlotMapKey::new(segment_id.value() as usize);
        let shared_memory_map = unsafe { &mut *self.shared_memory_map.get() };

        match shared_memory_map.get(key) {
            Some(entry) => {
                let state = entry.unregister_offset();
                if state == ShmEntryState::Empty
                    && self.current_idx.load(Ordering::Relaxed) != key.value()
                {
                    shared_memory_map.remove(key);
                }
            }
            None => {
                warn!(from self,
                      "Unable to unregister offset {:?} since the segment id is not mapped.", offset);
            }
        }
    }

    fn number_of_active_segments(&self) -> usize {
        let shared_memory_map = unsafe { &mut *self.shared_memory_map.get() };
        shared_memory_map.len()
    }
}

#[derive(Debug)]
pub struct DynamicMemory<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    state: UnsafeCell<InternalState<Allocator, Shm>>,
    _mgmt_segment: Shm,
    _data: PhantomData<Allocator>,
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> NamedConcept
    for DynamicMemory<Allocator, Shm>
{
    fn name(&self) -> &FileName {
        unsafe { &(*self.state.get()).builder_config.base_name }
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
        let msg = format!("Unable to remove ResizableSharedMemory {name:?}");

        let mgmt_name = Self::managment_segment_name(name);
        let mut shm_removed = fail!(from origin, when Shm::remove_cfg(&mgmt_name, config),
            "{msg} since the underlying managment segment could not be removed.");

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
        let msg = format!("Unable to determine if ResizableSharedMemory {name:?} exists");

        let mgmt_name = Self::managment_segment_name(name);
        Ok(
            fail!(from origin, when Shm::does_exist_cfg(&mgmt_name, config),
            "{msg} since the existance of the underlying managment segment could not be verified."),
        )
    }

    fn list_cfg(config: &Shm::Configuration) -> Result<Vec<FileName>, NamedConceptListError> {
        let origin = "resizable_shared_memory::Dynamic::list_cfg()";
        let mut names = vec![];
        let raw_names = fail!(from origin, when Shm::list_cfg(config),
                            "Unable to list ResizableSharedMemories since the underlying SharedMemories could not be listed.");

        for raw_name in &raw_names {
            if let Some(name) = Self::extract_name_from_management_segment(raw_name) {
                names.push(name);
            }
        }

        Ok(names)
    }

    fn remove_path_hint(value: &Path) -> Result<(), super::NamedConceptPathHintRemoveError> {
        Shm::remove_path_hint(value)
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> DynamicMemory<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    fn managment_segment_name(base_name: &FileName) -> FileName {
        let origin = "resizable_shared_memory::DynamicMemory::managment_segment_name()";
        let msg = "Unable to construct management segment name";
        let mut adjusted_name = base_name.clone();
        fatal_panic!(from origin, when adjusted_name.push_bytes(SEGMENT_ID_SEPARATOR),
                        "This should never happen! {msg} since it would result in an invalid file name.");
        fatal_panic!(from origin, when adjusted_name.push_bytes(MANAGEMENT_SUFFIX),
                        "This should never happen! {msg} since it would result in an invalid file name.");
        adjusted_name
    }

    fn extract_name_from_management_segment(name: &FileName) -> Option<FileName> {
        let mut name = name.clone();
        if let Ok(true) = name.strip_suffix(MANAGEMENT_SUFFIX) {
            if let Ok(true) = name.strip_suffix(SEGMENT_ID_SEPARATOR) {
                return Some(name);
            }
        }

        None
    }

    fn extract_name_and_segment_id(name: &FileName) -> Option<(FileName, SegmentId)> {
        let origin = "resizable_shared_memory::DynamicMemory::extract_name_and_segment_id()";
        let msg = "Unable to extract name and segment id";
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

            let mut raw_segment_id = *name.as_string();
            raw_segment_id.remove_range(0, segment_id_start_pos);

            // check nymber of digits
            for byte in raw_segment_id.as_bytes() {
                if byte.is_ascii_digit() {
                    return None;
                }
            }

            let segment_id_value = fatal_panic!(from origin,
                    when String::from_utf8_lossy(raw_segment_id.as_bytes()).parse::<u64>(),
                    "This should never happen! {msg} since the segment_id raw value is not an unsigned integer.");

            if segment_id_value > SegmentId::max_segment_id() as u64 {
                return None;
            }

            let mut name = name.clone();
            fatal_panic!(from origin,
                when name.remove_range(pos, name.len() - pos),
                "This should never happen! {msg} since the shared memory segment is an invalid file name without the segment id suffix.");

            return Some((name, SegmentId::new(segment_id_value as u8)));
        }

        None
    }

    #[allow(clippy::mut_from_ref)] // internal convenience function
    fn state_mut(&self) -> &mut InternalState<Allocator, Shm> {
        unsafe { &mut *self.state.get() }
    }

    fn state(&self) -> &InternalState<Allocator, Shm> {
        unsafe { &*self.state.get() }
    }

    fn create_segment(
        config: &MemoryConfig<Allocator, Shm>,
        segment_id: SegmentId,
        payload_size: usize,
    ) -> Result<Shm, SharedMemoryCreateError> {
        Self::segment_builder(&config.base_name, &config.shm, segment_id)
            .has_ownership(true)
            .size(payload_size)
            .create(&config.allocator_config_hint)
    }

    fn open_segment(
        config: &ViewConfig<Allocator, Shm>,
        segment_id: SegmentId,
    ) -> Result<Shm, SharedMemoryOpenError> {
        Self::segment_builder(&config.base_name, &config.shm, segment_id)
            .has_ownership(false)
            .timeout(config.shm_builder_timeout)
            .open()
    }

    fn segment_builder(
        base_name: &FileName,
        config: &Shm::Configuration,
        segment_id: SegmentId,
    ) -> Shm::Builder {
        let msg = "This should never happen! Unable to create additional shared memory segment since it would result in an invalid shared memory name.";
        let mut adjusted_name = base_name.clone();
        fatal_panic!(from config, when adjusted_name.push_bytes(SEGMENT_ID_SEPARATOR), "{msg}");
        fatal_panic!(from config, when adjusted_name.push_bytes(segment_id.value().to_string().as_bytes()), "{msg}");
        Shm::Builder::new(&adjusted_name).config(config)
    }

    fn create_resized_segment(
        &self,
        shm: &Shm,
        layout: Layout,
    ) -> Result<(), ResizableShmAllocationError> {
        let msg = "Unable to create resized segment for";
        let state = self.state_mut();
        let adjusted_segment_setup = shm
            .allocator()
            .resize_hint(layout, state.shared_state.allocation_strategy);
        let new_number_of_reallocations = state.current_idx.value() + 1;
        let segment_id = if new_number_of_reallocations < MAX_NUMBER_OF_REALLOCATIONS {
            SlotMapKey::new(new_number_of_reallocations)
        } else {
            fail!(from self, with ResizableShmAllocationError::MaxReallocationsReached,
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
            Some(segment) => {
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
            if state.shared_state.allocation_strategy == AllocationStrategy::Static {
                fail!(from self, with e.into(),
                                    "{msg} since there is not enough memory left ({:?}) and the allocation strategy {:?} forbids reallocation.",
                                    e, state.shared_state.allocation_strategy);
            } else {
                self.create_resized_segment(shm, layout)?;
                Ok(())
            }
        } else {
            fail!(from self, with e.into(), "{msg} due to {:?}.", e);
        }
    }

    unsafe fn perform_deallocation<F: FnMut(&ShmEntry<Allocator, Shm>)>(
        &self,
        offset: PointerOffset,
        mut deallocation_call: F,
    ) {
        let segment_id = SlotMapKey::new(offset.segment_id().value() as usize);
        let state = self.state_mut();
        match state.shared_memory_map.get(segment_id) {
            Some(entry) => {
                deallocation_call(entry);
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
}

impl<Shm: SharedMemoryForPoolAllocator> ResizableSharedMemoryForPoolAllocator<Shm>
    for DynamicMemory<PoolAllocator, Shm>
where
    Shm::Builder: Debug,
{
    unsafe fn deallocate_bucket(&self, offset: PointerOffset) {
        self.perform_deallocation(offset, |entry| entry.shm.deallocate_bucket(offset));
    }

    fn bucket_size(&self, segment_id: SegmentId) -> usize {
        let segment_id_key = SlotMapKey::new(segment_id.value() as usize);
        match self.state_mut().shared_memory_map.get(segment_id_key) {
            Some(entry) => entry.shm.bucket_size(),
            None => fatal_panic!(from self,
                        "This should never happen! Unable to acquire bucket size since the segment {:?} does not exist.",
                        segment_id),
        }
    }
}

impl<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> ResizableSharedMemory<Allocator, Shm>
    for DynamicMemory<Allocator, Shm>
where
    Shm::Builder: Debug,
{
    type ViewBuilder = DynamicViewBuilder<Allocator, Shm>;
    type MemoryBuilder = DynamicMemoryBuilder<Allocator, Shm>;
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
                Some(entry) => match entry.shm.allocate(layout) {
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
        self.perform_deallocation(offset, |entry| entry.shm.deallocate(offset, layout));
    }
}
