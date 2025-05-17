// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#[doc(hidden)]
pub mod details {
    use core::cell::UnsafeCell;
    use core::fmt::Debug;
    use core::marker::PhantomData;
    use core::sync::atomic::Ordering;
    use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};
    use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
    use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicU64, IoxAtomicU8, IoxAtomicUsize};

    use crate::dynamic_storage::{
        DynamicStorage, DynamicStorageBuilder, DynamicStorageCreateError, DynamicStorageOpenError,
        DynamicStorageOpenOrCreateError,
    };
    use crate::named_concept::*;
    use crate::shared_memory::SegmentId;
    pub use crate::zero_copy_connection::*;
    use iceoryx2_bb_container::vec::RelocatableVec;
    use iceoryx2_bb_elementary_traits::relocatable_container::RelocatableContainer;
    use iceoryx2_bb_lock_free::spsc::{
        index_queue::RelocatableIndexQueue,
        safely_overflowing_index_queue::RelocatableSafelyOverflowingIndexQueue,
    };
    use iceoryx2_bb_log::{fail, fatal_panic};
    use iceoryx2_bb_posix::adaptive_wait::AdaptiveWaitBuilder;

    use self::used_chunk_list::RelocatableUsedChunkList;

    #[derive(Debug, PartialEq, Eq, Copy)]
    pub struct Configuration<Storage: DynamicStorage<SharedManagementData>> {
        dynamic_storage_config: Storage::Configuration,
        _data: PhantomData<Storage>,
    }

    impl<Storage: DynamicStorage<SharedManagementData>> Clone for Configuration<Storage> {
        fn clone(&self) -> Self {
            Self {
                dynamic_storage_config: self.dynamic_storage_config.clone(),
                _data: PhantomData,
            }
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> Default for Configuration<Storage> {
        fn default() -> Self {
            Self {
                dynamic_storage_config: Storage::Configuration::default()
                    .path_hint(&Connection::<Storage>::default_path_hint())
                    .prefix(&Connection::<Storage>::default_prefix())
                    .suffix(&Connection::<Storage>::default_suffix()),
                _data: PhantomData,
            }
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> NamedConceptConfiguration
        for Configuration<Storage>
    {
        fn prefix(mut self, value: &FileName) -> Self {
            self.dynamic_storage_config = self.dynamic_storage_config.prefix(value);
            self
        }

        fn get_prefix(&self) -> &FileName {
            self.dynamic_storage_config.get_prefix()
        }

        fn suffix(mut self, value: &FileName) -> Self {
            self.dynamic_storage_config = self.dynamic_storage_config.suffix(value);
            self
        }

        fn path_hint(mut self, value: &Path) -> Self {
            self.dynamic_storage_config = self.dynamic_storage_config.path_hint(value);
            self
        }

        fn get_suffix(&self) -> &FileName {
            self.dynamic_storage_config.get_suffix()
        }

        fn get_path_hint(&self) -> &Path {
            self.dynamic_storage_config.get_path_hint()
        }

        fn path_for(&self, value: &FileName) -> FilePath {
            self.dynamic_storage_config.path_for(value)
        }

        fn extract_name_from_file(&self, value: &FileName) -> Option<FileName> {
            self.dynamic_storage_config.extract_name_from_file(value)
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(u8)]
    enum State {
        None = 0b00000000,
        Sender = 0b00000001,
        Receiver = 0b00000010,
        MarkedForDestruction = 0b10000000,
    }

    impl State {
        fn value(&self) -> u8 {
            *self as u8
        }
    }

    fn cleanup_shared_memory<Storage: DynamicStorage<SharedManagementData>>(
        storage: &Storage,
        state_to_remove: State,
    ) {
        let mgmt_data = storage.get();
        if mgmt_data.remove_state(state_to_remove) == State::MarkedForDestruction.value() {
            storage.acquire_ownership()
        }
    }

    #[derive(Debug)]
    struct SegmentDetails {
        used_chunk_list: RelocatableUsedChunkList,
        sample_size: IoxAtomicUsize,
    }

    impl SegmentDetails {
        fn new_uninit(number_of_samples: usize) -> Self {
            Self {
                used_chunk_list: unsafe { RelocatableUsedChunkList::new_uninit(number_of_samples) },
                sample_size: IoxAtomicUsize::new(0),
            }
        }

        const fn const_memory_size(number_of_samples: usize) -> usize {
            RelocatableUsedChunkList::const_memory_size(number_of_samples)
        }

        unsafe fn init<T: BaseAllocator>(&mut self, allocator: &T) -> Result<(), AllocationError> {
            self.used_chunk_list.init(allocator)
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    struct Channel {
        submission_queue: RelocatableSafelyOverflowingIndexQueue,
        completion_queue: RelocatableIndexQueue,
        state: IoxAtomicU64,
    }

    impl Channel {
        fn new(submission_queue_capacity: usize, completion_queue_capacity: usize) -> Self {
            Self {
                submission_queue: unsafe {
                    RelocatableSafelyOverflowingIndexQueue::new_uninit(submission_queue_capacity)
                },
                completion_queue: unsafe {
                    RelocatableIndexQueue::new_uninit(completion_queue_capacity)
                },
                state: IoxAtomicU64::new(INITIAL_CHANNEL_STATE),
            }
        }

        const fn const_memory_size(
            submission_queue_capacity: usize,
            completion_queue_capacity: usize,
        ) -> usize {
            RelocatableIndexQueue::const_memory_size(completion_queue_capacity)
                + RelocatableSafelyOverflowingIndexQueue::const_memory_size(
                    submission_queue_capacity,
                )
        }

        fn init(&mut self, allocator: &mut BumpAllocator) {
            let msg = "Failed to initialize channel";
            fatal_panic!(from self, when unsafe { self.submission_queue.init(allocator) },
                        "{} since the submission queue allocation failed. - This is an implementation bug!", msg);
            fatal_panic!(from self, when unsafe { self.completion_queue.init(allocator) },
                        "{} since the completion queue allocation failed. - This is an implementation bug!", msg);
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    pub struct SharedManagementData {
        channels: RelocatableVec<Channel>,
        segment_details: RelocatableVec<SegmentDetails>,
        state: IoxAtomicU8,
        max_borrowed_samples: usize,
        number_of_samples_per_segment: usize,
        number_of_segments: u8,
        enable_safe_overflow: bool,
    }

    impl SharedManagementData {
        fn new(
            enable_safe_overflow: bool,
            max_borrowed_samples: usize,
            number_of_samples_per_segment: usize,
            number_of_segments: u8,
            number_of_channels: usize,
        ) -> Self {
            Self {
                channels: unsafe { RelocatableVec::new_uninit(number_of_channels) },
                segment_details: unsafe {
                    RelocatableVec::new_uninit(number_of_segments as usize * number_of_channels)
                },
                enable_safe_overflow,
                max_borrowed_samples,
                number_of_samples_per_segment,
                number_of_segments,
                state: IoxAtomicU8::new(State::None.value()),
            }
        }

        fn get_segment_details(&self, segment_id: usize, channel_id: usize) -> &SegmentDetails {
            let idx = channel_id * self.number_of_segments as usize + segment_id;
            &self.segment_details[idx]
        }

        fn is_connected(&self) -> bool {
            self.state.load(Ordering::Relaxed) == State::Sender.value() | State::Receiver.value()
        }

        fn remove_state(&self, state_to_remove: State) -> u8 {
            let mut current_state = self.state.load(Ordering::Relaxed);
            if current_state == State::MarkedForDestruction.value() {
                return State::MarkedForDestruction.value();
            }

            loop {
                let new_state = if current_state == state_to_remove.value() {
                    State::MarkedForDestruction.value()
                } else {
                    current_state & !state_to_remove.value()
                };

                match self.state.compare_exchange(
                    current_state,
                    new_state,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        current_state = new_state;
                        break;
                    }
                    Err(s) => {
                        current_state = s;
                    }
                }
            }

            current_state
        }

        fn reserve_port(&self, new_state: u8, msg: &str) -> Result<(), ZeroCopyCreationError> {
            let mut current_state = self.state.load(Ordering::Relaxed);

            loop {
                if current_state & new_state != 0 {
                    fail!(from self, with ZeroCopyCreationError::AnotherInstanceIsAlreadyConnected,
                    "{} since an instance is already connected.", msg);
                } else if current_state & State::MarkedForDestruction.value() != 0 {
                    fail!(from self, with ZeroCopyCreationError::IsBeingCleanedUp,
                    "{} since the connection is currently being cleaned up.", msg);
                }

                match self.state.compare_exchange(
                    current_state,
                    current_state | new_state,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(v) => {
                        current_state = v;
                    }
                }
            }

            Ok(())
        }

        const fn const_memory_size(
            submission_queue_capacity: usize,
            completion_queue_capacity: usize,
            number_of_samples: usize,
            number_of_segments: u8,
            number_of_channels: usize,
        ) -> usize {
            let number_of_segments = number_of_segments as usize;
            number_of_channels
                * Channel::const_memory_size(submission_queue_capacity, completion_queue_capacity)
                + RelocatableVec::<Channel>::const_memory_size(number_of_channels)
                + SegmentDetails::const_memory_size(number_of_samples)
                    * number_of_segments
                    * number_of_channels
                + RelocatableVec::<SegmentDetails>::const_memory_size(
                    number_of_segments * number_of_channels,
                )
        }

        unsafe fn init(
            &mut self,
            allocator: &mut BumpAllocator,
            submission_queue_capacity: usize,
            completion_queue_capacity: usize,
        ) {
            let msg = "Failed to initialize SharedManagementData";
            // initialize channels
            fatal_panic!(from self, when unsafe {self.channels.init(allocator)},
                "{} since the channels vector allocation failed. - This is an implementation bug!", msg);
            for n in 0..self.channels.capacity() {
                unsafe {
                    self.channels.push(Channel::new(
                        submission_queue_capacity,
                        completion_queue_capacity,
                    ))
                };
                self.channels[n].init(allocator);
            }

            // initialize segment details
            fatal_panic!(from self, when unsafe { self.segment_details.init(allocator) },
                        "{} since the used chunk list vector allocation failed. - This is an implementation bug!", msg);

            for n in 0..self.segment_details.capacity() {
                if !unsafe {
                    self.segment_details.push(SegmentDetails::new_uninit(
                        self.number_of_samples_per_segment,
                    ))
                } {
                    fatal_panic!(from self,
                        "{} since the used chunk list could not be added. - This is an implementation bug!", msg);
                }

                fatal_panic!(from self, when unsafe { self.segment_details[n].init(allocator) },
                    "{} since the used chunk list for segment id {} failed to allocate memory. - This is an implementation bug!",
                    msg, n);
            }
        }
    }

    #[derive(Debug)]
    pub struct Builder<Storage: DynamicStorage<SharedManagementData>> {
        name: FileName,
        buffer_size: usize,
        enable_safe_overflow: bool,
        max_borrowed_samples_per_channel: usize,
        number_of_samples_per_segment: usize,
        number_of_segments: u8,
        number_of_channels: usize,
        initial_channel_state: u64,
        timeout: Duration,
        config: Configuration<Storage>,
    }

    impl<Storage: DynamicStorage<SharedManagementData>> Builder<Storage> {
        fn submission_queue_size(&self) -> usize {
            self.buffer_size
        }

        fn completion_queue_size(&self) -> usize {
            self.buffer_size + self.max_borrowed_samples_per_channel + 1
        }

        fn create_or_open_shm(
            &self,
            port_to_register: State,
        ) -> Result<Storage, ZeroCopyCreationError> {
            let supplementary_size = SharedManagementData::const_memory_size(
                self.submission_queue_size(),
                self.completion_queue_size(),
                self.number_of_samples_per_segment,
                self.number_of_segments,
                self.number_of_channels,
            );

            let msg = "Failed to acquire underlying shared memory";
            let storage = <<Storage as DynamicStorage<SharedManagementData>>::Builder<'_> as NamedConceptBuilder<
            Storage,
        >>::new(&self.name)
        .config(&self.config.dynamic_storage_config)
        .timeout(self.timeout)
        .supplementary_size(supplementary_size)
        .call_drop_on_destruction(false)
        .initializer(|data, allocator| {
            unsafe { data.init(allocator, self.submission_queue_size(), self.completion_queue_size())};
            for channel in data.channels.iter() {
                channel.state.store(self.initial_channel_state, Ordering::Relaxed);
            }

            true
        })
        .open_or_create(
            SharedManagementData::new(
                                    self.enable_safe_overflow,
                                    self.max_borrowed_samples_per_channel,
                                    self.number_of_samples_per_segment,
                                    self.number_of_segments,
                                    self.number_of_channels
                                )
            );

            let storage = match storage {
                Ok(storage) => storage,
                Err(DynamicStorageOpenOrCreateError::DynamicStorageCreateError(
                    DynamicStorageCreateError::InsufficientPermissions,
                )) => {
                    fail!(from self, with ZeroCopyCreationError::InsufficientPermissions,
                    "{} due to insufficient permissions to create underlying dynamic storage.", msg);
                }
                Err(DynamicStorageOpenOrCreateError::DynamicStorageOpenError(
                    DynamicStorageOpenError::VersionMismatch,
                )) => {
                    fail!(from self, with ZeroCopyCreationError::VersionMismatch,
                    "{} since the version of the connection does not match.", msg);
                }
                Err(DynamicStorageOpenOrCreateError::DynamicStorageOpenError(
                    DynamicStorageOpenError::InitializationNotYetFinalized,
                )) => {
                    fail!(from self, with ZeroCopyCreationError::InitializationNotYetFinalized,
                    "{} since the initialization of the zero copy connection is not finalized.", msg);
                }
                Err(e) => {
                    fail!(from self, with ZeroCopyCreationError::InternalError,
                    "{} due to an internal failure ({:?}).", msg, e);
                }
            };

            storage.get().reserve_port(port_to_register.value(), msg)?;

            if storage.has_ownership() {
                storage.release_ownership();
            } else {
                let msg = "Failed to open existing connection";

                if storage.get().channels[0].submission_queue.capacity()
                    != self.submission_queue_size()
                {
                    cleanup_shared_memory(&storage, port_to_register);
                    fail!(from self, with ZeroCopyCreationError::IncompatibleBufferSize,
                        "{} since the connection has a buffer size of {} but a buffer size of {} is required.",
                        msg, storage.get().channels[0].submission_queue.capacity(), self.submission_queue_size());
                }

                if storage.get().channels[0].completion_queue.capacity()
                    != self.completion_queue_size()
                {
                    cleanup_shared_memory(&storage, port_to_register);
                    fail!(from self, with ZeroCopyCreationError::IncompatibleMaxBorrowedSamplesPerChannelSetting,
                        "{} since the max borrowed sample per channel setting is set to {} but a value of {} is required.",
                        msg, storage.get().channels[0].completion_queue.capacity() - storage.get().channels[0].submission_queue.capacity(), self.max_borrowed_samples_per_channel);
                }

                if storage.get().enable_safe_overflow != self.enable_safe_overflow {
                    cleanup_shared_memory(&storage, port_to_register);
                    fail!(from self, with ZeroCopyCreationError::IncompatibleOverflowSetting,
                        "{} since the safe overflow is set to {} but should be set to {}.",
                        msg, storage.get().enable_safe_overflow, self.enable_safe_overflow);
                }

                if storage.get().number_of_samples_per_segment != self.number_of_samples_per_segment
                {
                    cleanup_shared_memory(&storage, port_to_register);
                    fail!(from self, with ZeroCopyCreationError::IncompatibleNumberOfSamples,
                        "{} since the requested number of samples is set to {} but should be set to {}.",
                        msg, self.number_of_samples_per_segment, storage.get().number_of_samples_per_segment);
                }

                if storage.get().number_of_segments != self.number_of_segments {
                    cleanup_shared_memory(&storage, port_to_register);
                    fail!(from self, with ZeroCopyCreationError::IncompatibleNumberOfSegments,
                        "{} since the requested number of segments is set to {} but should be set to {}.",
                        msg, self.number_of_segments, storage.get().number_of_segments);
                }

                if storage.get().channels.capacity() != self.number_of_channels {
                    cleanup_shared_memory(&storage, port_to_register);
                    fail!(from self, with ZeroCopyCreationError::IncompatibleNumberOfChannels,
                        "{} since the requested number of channels is set to {} but should be set to {}.",
                        msg, self.number_of_channels, storage.get().channels.capacity());
                }
            }

            Ok(storage)
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> NamedConceptBuilder<Connection<Storage>>
        for Builder<Storage>
    {
        fn new(name: &FileName) -> Self {
            Self {
                name: name.clone(),
                buffer_size: DEFAULT_BUFFER_SIZE,
                enable_safe_overflow: DEFAULT_ENABLE_SAFE_OVERFLOW,
                max_borrowed_samples_per_channel: DEFAULT_MAX_BORROWED_SAMPLES_PER_CHANNEL,
                number_of_samples_per_segment: DEFAULT_NUMBER_OF_SAMPLES_PER_SEGMENT,
                number_of_segments: DEFAULT_MAX_SUPPORTED_SHARED_MEMORY_SEGMENTS,
                number_of_channels: DEFAULT_NUMBER_OF_CHANNELS,
                config: Configuration::default(),
                initial_channel_state: INITIAL_CHANNEL_STATE,
                timeout: Duration::ZERO,
            }
        }

        fn config(mut self, config: &Configuration<Storage>) -> Self {
            self.config = config.clone();
            self
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>>
        ZeroCopyConnectionBuilder<Connection<Storage>> for Builder<Storage>
    {
        fn max_supported_shared_memory_segments(mut self, value: u8) -> Self {
            self.number_of_segments = value.clamp(1, u8::MAX);
            self
        }

        fn initial_channel_state(mut self, value: u64) -> Self {
            self.initial_channel_state = value;
            self
        }

        fn buffer_size(mut self, value: usize) -> Self {
            self.buffer_size = value.clamp(1, usize::MAX);
            self
        }

        fn timeout(mut self, value: Duration) -> Self {
            self.timeout = value;
            self
        }

        fn enable_safe_overflow(mut self, value: bool) -> Self {
            self.enable_safe_overflow = value;
            self
        }

        fn number_of_samples_per_segment(mut self, value: usize) -> Self {
            self.number_of_samples_per_segment = value.clamp(1, usize::MAX);
            self
        }

        fn receiver_max_borrowed_samples_per_channel(mut self, value: usize) -> Self {
            self.max_borrowed_samples_per_channel = value.clamp(1, usize::MAX);
            self
        }

        fn number_of_channels(mut self, value: usize) -> Self {
            self.number_of_channels = value.clamp(1, usize::MAX);
            self
        }

        fn create_sender(
            self,
        ) -> Result<<Connection<Storage> as ZeroCopyConnection>::Sender, ZeroCopyCreationError>
        {
            let msg = "Unable to create sender";
            let storage = fail!(from self, when self.create_or_open_shm(State::Sender),
            "{} since the corresponding connection could not be created or opened", msg);

            Ok(Sender {
                storage,
                name: self.name,
            })
        }

        fn create_receiver(
            self,
        ) -> Result<<Connection<Storage> as ZeroCopyConnection>::Receiver, ZeroCopyCreationError>
        {
            let msg = "Unable to create receiver";
            let storage = fail!(from self, when self.create_or_open_shm(State::Receiver),
            "{} since the corresponding connection could not be created or opened", msg);

            Ok(Receiver {
                storage,
                borrow_counter: {
                    let mut borrow_counter = vec![];
                    for _ in 0..self.number_of_channels {
                        borrow_counter.push(UnsafeCell::new(0));
                    }
                    borrow_counter
                },
                name: self.name,
            })
        }
    }

    #[derive(Debug)]
    pub struct Sender<Storage: DynamicStorage<SharedManagementData>> {
        storage: Storage,
        name: FileName,
    }

    impl<Storage: DynamicStorage<SharedManagementData>> Drop for Sender<Storage> {
        fn drop(&mut self) {
            cleanup_shared_memory(&self.storage, State::Sender);
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> NamedConcept for Sender<Storage> {
        fn name(&self) -> &FileName {
            &self.name
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> ZeroCopyPortDetails for Sender<Storage> {
        fn buffer_size(&self) -> usize {
            self.storage.get().channels[0].submission_queue.capacity()
        }

        fn max_supported_shared_memory_segments(&self) -> u8 {
            self.storage.get().number_of_segments
        }

        fn max_borrowed_samples(&self) -> usize {
            self.storage.get().max_borrowed_samples
        }

        fn has_enabled_safe_overflow(&self) -> bool {
            self.storage.get().enable_safe_overflow
        }

        fn is_connected(&self) -> bool {
            self.storage.get().is_connected()
        }

        fn number_of_channels(&self) -> usize {
            self.storage.get().channels.capacity()
        }

        fn channel_state(&self, channel_id: ChannelId) -> &IoxAtomicU64 {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());
            &self.storage.get().channels[channel_id.value()].state
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> ZeroCopySender for Sender<Storage> {
        fn try_send(
            &self,
            ptr: PointerOffset,
            sample_size: usize,
            channel_id: ChannelId,
        ) -> Result<Option<PointerOffset>, ZeroCopySendError> {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());

            let msg = "Unable to send sample";
            let storage = self.storage.get();

            if !storage.enable_safe_overflow
                && storage.channels[channel_id.value()]
                    .submission_queue
                    .is_full()
            {
                fail!(from self, with ZeroCopySendError::ReceiveBufferFull,
                             "{} since the receive buffer is full.", msg);
            }

            let segment_id = ptr.segment_id().value() as usize;
            let segment_details = storage.get_segment_details(segment_id, channel_id.value());
            segment_details
                .sample_size
                .store(sample_size, Ordering::Relaxed);
            debug_assert!(ptr.offset() % sample_size == 0);
            let index = ptr.offset() / sample_size;

            debug_assert!(segment_id < storage.number_of_segments as usize);

            let did_not_send_same_offset_twice = segment_details.used_chunk_list.insert(index);
            debug_assert!(did_not_send_same_offset_twice);

            match unsafe {
                storage.channels[channel_id.value()]
                    .submission_queue
                    .push(ptr.as_value())
            } {
                Some(v) => {
                    let pointer_offset = PointerOffset::from_value(v);
                    let segment_id = pointer_offset.segment_id().value() as usize;

                    let segment_details =
                        storage.get_segment_details(segment_id, channel_id.value());
                    debug_assert!(
                        pointer_offset.offset()
                            % segment_details.sample_size.load(Ordering::Relaxed)
                            == 0
                    );
                    let index = pointer_offset.offset()
                        / segment_details.sample_size.load(Ordering::Relaxed);

                    if !segment_details.used_chunk_list.remove(index) {
                        fail!(from self, with ZeroCopySendError::ConnectionCorrupted,
                        "{} since the invalid offset {:?} was returned on overflow.", msg, pointer_offset);
                    }

                    Ok(Some(pointer_offset))
                }
                None => Ok(None),
            }
        }

        fn blocking_send(
            &self,
            ptr: PointerOffset,
            sample_size: usize,
            channel_id: ChannelId,
        ) -> Result<Option<PointerOffset>, ZeroCopySendError> {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());

            if !self.storage.get().enable_safe_overflow {
                AdaptiveWaitBuilder::new()
                    .create()
                    .unwrap()
                    .wait_while(|| {
                        self.storage.get().channels[channel_id.value()]
                            .submission_queue
                            .is_full()
                    })
                    .unwrap();
            }

            self.try_send(ptr, sample_size, channel_id)
        }

        fn reclaim(
            &self,
            channel_id: ChannelId,
        ) -> Result<Option<PointerOffset>, ZeroCopyReclaimError> {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());

            let msg = "Unable to reclaim sample";

            let storage = self.storage.get();
            match unsafe { storage.channels[channel_id.value()].completion_queue.pop() } {
                None => Ok(None),
                Some(v) => {
                    let pointer_offset = PointerOffset::from_value(v);
                    let segment_id = pointer_offset.segment_id().value() as usize;

                    debug_assert!(segment_id < storage.number_of_segments as usize);

                    if segment_id >= storage.segment_details.len() {
                        fail!(from self, with ZeroCopyReclaimError::ReceiverReturnedCorruptedPointerOffset,
                            "{} since the receiver returned a non-existing segment id {:?}.",
                            msg, pointer_offset);
                    }

                    let segment_details =
                        storage.get_segment_details(segment_id, channel_id.value());
                    debug_assert!(
                        pointer_offset.offset()
                            % segment_details.sample_size.load(Ordering::Relaxed)
                            == 0
                    );
                    let index = pointer_offset.offset()
                        / segment_details.sample_size.load(Ordering::Relaxed);

                    if !segment_details.used_chunk_list.remove(index) {
                        fail!(from self, with ZeroCopyReclaimError::ReceiverReturnedCorruptedPointerOffset,
                            "{} since the receiver returned a corrupted offset {:?}.",
                            msg, pointer_offset);
                    }
                    Ok(Some(pointer_offset))
                }
            }
        }

        unsafe fn acquire_used_offsets<F: FnMut(PointerOffset)>(&self, mut callback: F) {
            for (n, segment_details) in self.storage.get().segment_details.iter().enumerate() {
                segment_details.used_chunk_list.remove_all(|index| {
                    callback(PointerOffset::from_offset_and_segment_id(
                        index * segment_details.sample_size.load(Ordering::Relaxed),
                        self.segment_id_from_index(n),
                    ))
                });
            }
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> Sender<Storage> {
        fn segment_id_from_index(&self, index: usize) -> SegmentId {
            let storage = self.storage.get();
            let number_of_segments = storage.number_of_segments as usize;
            // the segment details contain an entry for every channel in every segment.
            // so it is in memory a vector of vector where the first index is the
            // channel id and the second is the segment id. this vector is, to make the
            // relocatable construction easier, mapped to a one dimension vector with the
            // same memory layout.
            //
            // example for 3 segments and 4 channels
            //
            // index       segment id      channel id
            // 0           0               0
            // 1           1               0
            // 2           2               0
            // 3           0               1
            // 4           1               1
            // 5           2               1
            // 6           0               2
            // 7           1               2
            // 8           2               2
            // 9           0               3
            // 10          1               3
            // 11          2               3
            //
            // to map an index to a segment id, we need to substract from index the
            // rounded down value of index which is rounded down to the greatest multiple
            // of number_of_segments.
            SegmentId::new((index - (index / number_of_segments) * number_of_segments) as u8)
        }
    }

    #[derive(Debug)]
    pub struct Receiver<Storage: DynamicStorage<SharedManagementData>> {
        storage: Storage,
        borrow_counter: Vec<UnsafeCell<usize>>,
        name: FileName,
    }

    impl<Storage: DynamicStorage<SharedManagementData>> Drop for Receiver<Storage> {
        fn drop(&mut self) {
            cleanup_shared_memory(&self.storage, State::Receiver);
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> Receiver<Storage> {
        #[allow(clippy::mut_from_ref)]
        // convenience to access internal mutable object
        fn borrow_counter(&self, channel_id: ChannelId) -> &mut usize {
            #[deny(clippy::mut_from_ref)]
            unsafe {
                &mut *self.borrow_counter[channel_id.value()].get()
            }
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> NamedConcept for Receiver<Storage> {
        fn name(&self) -> &FileName {
            &self.name
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> ZeroCopyPortDetails for Receiver<Storage> {
        fn buffer_size(&self) -> usize {
            self.storage.get().channels[0].submission_queue.capacity()
        }

        fn max_supported_shared_memory_segments(&self) -> u8 {
            self.storage.get().number_of_segments
        }

        fn max_borrowed_samples(&self) -> usize {
            self.storage.get().max_borrowed_samples
        }

        fn has_enabled_safe_overflow(&self) -> bool {
            self.storage.get().enable_safe_overflow
        }

        fn is_connected(&self) -> bool {
            self.storage.get().is_connected()
        }

        fn number_of_channels(&self) -> usize {
            self.storage.get().channels.capacity()
        }

        fn channel_state(&self, channel_id: ChannelId) -> &IoxAtomicU64 {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());
            &self.storage.get().channels[channel_id.value()].state
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> ZeroCopyReceiver for Receiver<Storage> {
        fn has_data(&self, channel_id: ChannelId) -> bool {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());
            !self.storage.get().channels[channel_id.value()]
                .submission_queue
                .is_empty()
        }

        fn receive(
            &self,
            channel_id: ChannelId,
        ) -> Result<Option<PointerOffset>, ZeroCopyReceiveError> {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());

            if *self.borrow_counter(channel_id) >= self.storage.get().max_borrowed_samples {
                fail!(from self, with ZeroCopyReceiveError::ReceiveWouldExceedMaxBorrowValue,
                "Unable to receive another sample since already {} samples were borrowed and this would exceed the max borrow value of {}.",
                    self.borrow_counter(channel_id), self.max_borrowed_samples());
            }

            match unsafe {
                self.storage.get().channels[channel_id.value()]
                    .submission_queue
                    .pop()
            } {
                None => Ok(None),
                Some(v) => {
                    *self.borrow_counter(channel_id) += 1;
                    Ok(Some(PointerOffset::from_value(v)))
                }
            }
        }

        fn borrow_count(&self, channel_id: ChannelId) -> usize {
            *self.borrow_counter(channel_id)
        }

        fn release(
            &self,
            ptr: PointerOffset,
            channel_id: ChannelId,
        ) -> Result<(), ZeroCopyReleaseError> {
            debug_assert!(channel_id.value() < self.storage.get().channels.capacity());

            match unsafe {
                self.storage.get().channels[channel_id.value()]
                    .completion_queue
                    .push(ptr.as_value())
            } {
                true => {
                    *self.borrow_counter(channel_id) -= 1;
                    Ok(())
                }
                false => {
                    fail!(from self, with ZeroCopyReleaseError::RetrieveBufferFull,
                    "Unable to release pointer since the retrieve buffer is full.");
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct Connection<Storage: DynamicStorage<SharedManagementData>> {
        _data: PhantomData<Storage>,
    }

    impl<Storage: DynamicStorage<SharedManagementData>> NamedConceptMgmt for Connection<Storage> {
        type Configuration = Configuration<Storage>;

        fn does_exist_cfg(
            name: &FileName,
            cfg: &Self::Configuration,
        ) -> Result<bool, crate::static_storage::file::NamedConceptDoesExistError> {
            Ok(fail!(from "ZeroCopyConnection::does_exist_cfg()",
                    when Storage::does_exist_cfg(name, &cfg.dynamic_storage_config),
                    "Failed to check if ZeroCopyConnection \"{}\" exists.",
                    name))
        }

        fn list_cfg(
            cfg: &Self::Configuration,
        ) -> Result<Vec<FileName>, crate::static_storage::file::NamedConceptListError> {
            Ok(fail!(from "ZeroCopyConnection::list_cfg()",
                    when Storage::list_cfg(&cfg.dynamic_storage_config),
                    "Failed to list all ZeroCopyConnections."))
        }

        unsafe fn remove_cfg(
            name: &FileName,
            cfg: &Self::Configuration,
        ) -> Result<bool, crate::static_storage::file::NamedConceptRemoveError> {
            Ok(fail!(from "ZeroCopyConnection::remove_cfg()",
                    when Storage::remove_cfg(name, &cfg.dynamic_storage_config),
                    "Failed to remove ZeroCopyConnection \"{}\".", name))
        }

        fn remove_path_hint(_value: &Path) -> Result<(), NamedConceptPathHintRemoveError> {
            Ok(())
        }
    }
    impl<Storage: DynamicStorage<SharedManagementData>> Connection<Storage> {
        fn open_storage(
            name: &FileName,
            config: &<Connection<Storage> as NamedConceptMgmt>::Configuration,
            msg: &str,
        ) -> Result<Storage, ZeroCopyPortRemoveError> {
            let origin = "Connection::open_storage()";
            match <<Storage as DynamicStorage<SharedManagementData>>::Builder<'_> as NamedConceptBuilder<
                    Storage>>::new(name)
                       .config(&config.dynamic_storage_config).open() {
                           Ok(storage) => Ok(storage),
                           Err(DynamicStorageOpenError::VersionMismatch) => {
                               fail!(from origin, with ZeroCopyPortRemoveError::VersionMismatch,
                                   "{msg} since the underlying dynamic storage has a different iceoryx2 version.");
                           }
                           Err(DynamicStorageOpenError::InitializationNotYetFinalized) => {
                               fail!(from origin, with ZeroCopyPortRemoveError::InsufficientPermissions,
                                   "{msg} due to insufficient permissions.");
                           }
                           Err(DynamicStorageOpenError::DoesNotExist) => {
                               fail!(from origin, with ZeroCopyPortRemoveError::DoesNotExist,
                                   "{msg} since the underlying dynamic storage does not exist.");
                           }
                           Err(DynamicStorageOpenError::InternalError) => {
                               fail!(from origin, with ZeroCopyPortRemoveError::InternalError,
                                   "{msg} due to an internal error.");
                           }
                       }
        }
    }

    impl<Storage: DynamicStorage<SharedManagementData>> ZeroCopyConnection for Connection<Storage> {
        type Sender = Sender<Storage>;
        type Builder = Builder<Storage>;
        type Receiver = Receiver<Storage>;

        unsafe fn remove_sender(
            name: &FileName,
            config: &Self::Configuration,
        ) -> Result<(), ZeroCopyPortRemoveError> {
            let storage = Self::open_storage(
                name,
                config,
                "Unable to remove forcefully the sender of the Zero Copy Connection",
            )?;
            cleanup_shared_memory(&storage, State::Sender);
            Ok(())
        }

        unsafe fn remove_receiver(
            name: &FileName,
            config: &Self::Configuration,
        ) -> Result<(), ZeroCopyPortRemoveError> {
            let storage = Self::open_storage(
                name,
                config,
                "Unable to remove forcefully the receiver of the Zero Copy Connection",
            )?;
            cleanup_shared_memory(&storage, State::Receiver);
            Ok(())
        }

        fn does_support_safe_overflow() -> bool {
            true
        }

        fn has_configurable_buffer_size() -> bool {
            true
        }
    }
}
