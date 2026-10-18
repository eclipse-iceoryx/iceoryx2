// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use crate::{
    clock::{
        Clock, ClockBlockingSleepError, ClockBuilder, ClockBuilderOpenClockError,
        ClockListenerSchedule, ClockListenerScheduleError, ClockNow, ClockNowError,
        ClockReactorSchedule, ClockReactorScheduleError, ScheduleGuard, ScheduleHandle,
        ScheduleOnHandle, SteppedClock, TimeController, TimeControllerAdvanceError,
        TimeControllerBuilder, TimeControllerBuilderCreateError, TimeControllerBuilderOpenError,
        TimeControllerBuilderOpenOrCreateError, Timeout,
        internal::{create_notifier, create_reactor_attachment, create_waiter},
    },
    dynamic_storage::{
        DynamicStorage, DynamicStorageBuilder, DynamicStorageOpenError,
        posix_shared_memory::Storage,
    },
    event::{Event, EventId, Notifier, UnixDatagramShmCountingBitSet},
    named_concept::{NamedConceptBuilder, NamedConceptConfiguration, NamedConceptMgmt},
};
use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;
use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull, time::Duration};
use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64, Ordering};
use iceoryx2_bb_container::{
    byte_atomic::RelocatableByteAtomic,
    queue::RelocatableContainer,
    vector::{RelocatableVec, Vector},
};
use iceoryx2_bb_derive_macros::{AtomicCopy, ZeroCopySend};
use iceoryx2_bb_elementary_traits::{
    atomic_copy::AtomicCopy, testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_lock_free::mpmc::{
    unique_index_set::UniqueIndexSet, unique_index_set_enums::ReleaseMode,
};
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::{file::AccessMode, file_descriptor_set::SynchronousMultiplexing};
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_system_types::{file_name::FileName, path::Path};
use iceoryx2_log::{debug, fail};

const DEFAULT_SCHEDULE_CAPACITY: usize = 100;

pub struct Guard<'a> {
    generation_counter: u64,
    index: u32,
    generation_counters: &'a RelocatableVec<AtomicU64>,
    index_list: &'a UniqueIndexSet,
}

impl<'a> ScheduleGuard<'a> for Guard<'a> {}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        if self.generation_counters[self.index as usize]
            .compare_exchange(
                self.generation_counter,
                0,
                Ordering::SeqCst,
                Ordering::SeqCst,
            )
            .is_ok()
        {
            unsafe {
                self.index_list
                    .release_raw_index(self.index, ReleaseMode::Default)
            };
        }
    }
}

#[derive(Debug, Clone)]
// TODO iox2-2100: generic parameter for storage
pub struct TimeControllerConfig {
    storage_config: <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration,
    capacity: usize,
    creation_time: TimePoint,
}

impl Default for TimeControllerConfig {
    fn default() -> Self {
        let storage_config =
            <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration::default();
        let storage_config = storage_config
            .suffix(&SimulationClock::<UnixDatagramShmCountingBitSet>::default_suffix());
        Self {
            storage_config,
            capacity: DEFAULT_SCHEDULE_CAPACITY,
            creation_time: TimePoint::new(),
        }
    }
}

impl TimeControllerConfig {
    pub fn capacity(mut self, value: usize) -> Self {
        self.capacity = value;
        self
    }

    pub fn creation_time(mut self, value: TimePoint) -> Self {
        self.creation_time = value;
        self
    }

    pub fn storage_config(
        mut self,
        value: <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration,
    ) -> Self {
        self.storage_config = value;
        self
    }

    pub fn get_storage_config(
        &self,
    ) -> &<Storage<SimulationClockState> as NamedConceptMgmt>::Configuration {
        &self.storage_config
    }
}

impl NamedConceptConfiguration for TimeControllerConfig {
    fn prefix(mut self, value: &FileName) -> Self {
        let config = self.storage_config.prefix(value);
        self.storage_config = config;
        self
    }

    fn get_prefix(&self) -> &FileName {
        self.storage_config.get_prefix()
    }

    fn suffix(mut self, value: &FileName) -> Self {
        let config = self.storage_config.suffix(value);
        self.storage_config = config;
        self
    }

    fn get_suffix(&self) -> &FileName {
        self.storage_config.get_suffix()
    }

    fn path_hint(mut self, value: &Path) -> Self {
        let config = self.storage_config.path_hint(value);
        self.storage_config = config;
        self
    }

    fn get_path_hint(&self) -> &Path {
        self.storage_config.get_path_hint()
    }
}

#[repr(C)]
#[derive(Debug, ZeroCopySend, AtomicCopy, Clone, Copy)]
pub(crate) struct ScheduledTimepoint {
    pub(crate) trigger_name: FileName,
    pub(crate) timeout_in_ns_after_creation: u64,
}

#[repr(C)]
#[derive(Debug, ZeroCopySend)]
pub struct SimulationClockState {
    creation_time: TimePoint,
    nanoseconds_since_creation: AtomicU64,
    scheduled_timepoints: RelocatableVec<RelocatableByteAtomic<ScheduledTimepoint>>,
    generation_counters: RelocatableVec<AtomicU64>,
    index_list: UniqueIndexSet,
    global_generation_counter: AtomicU64,
    name_generator_counter: AtomicU64,
    has_active_time_controller: AtomicBool, // TODO iox2-2100: use UniqueNodeId::owner_id()
}

#[derive(Debug)]
pub struct SimulationClockBuilder<E: Event> {
    name: FileName,
    config: TimeControllerConfig,
    event_config: E::Configuration,
    _event: PhantomData<E>,
}

impl<E: Event + 'static> ClockBuilder<E, SimulationClock<E>> for SimulationClockBuilder<E>
where
    E::Listener: SynchronousMultiplexing,
{
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            config: TimeControllerConfig::default(),
            event_config: E::Configuration::default(),
            _event: PhantomData,
        }
    }

    fn config(mut self, config: &TimeControllerConfig) -> Self {
        self.config = config.clone();
        self
    }

    fn event_config(mut self, config: &<E>::Configuration) -> Self {
        self.event_config = config.clone();
        self
    }

    fn open_clock(self) -> Result<SimulationClock<E>, ClockBuilderOpenClockError> {
        match self.open_or_create_clock_state_storage() {
            Ok(storage) => Ok(SimulationClock {
                name: self.name,
                state: storage,
                event_config: self.event_config,
            }),
            Err(_) => {
                // TODO iox2-2100: more fine-grained error handling
                fail!(from self, with ClockBuilderOpenClockError::InternalError,
                    "Unable to open the simulation clock due to an internal error while
                    opening/creating the clock state.");
            }
        }
    }
}

impl<E: Event> SimulationClockBuilder<E> {
    fn open_or_create_clock_state_storage(
        &self,
    ) -> Result<Storage<SimulationClockState>, TimeControllerBuilderOpenOrCreateError> {
        let capacity = self.config.capacity;
        Ok(fail!(from self,
                when
                <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(
                    &self.name,
                )
                .config(&self.config.storage_config)
                .has_ownership(false)
                .supplementary_size(
                    RelocatableVec::<RelocatableByteAtomic<ScheduledTimepoint>>::const_memory_size(capacity)
                        + RelocatableByteAtomic::<ScheduledTimepoint>::const_memory_size() * capacity
                        + RelocatableVec::<AtomicU64>::const_memory_size(capacity)
                        + UniqueIndexSet::const_memory_size(capacity),
                )
                .initializer(
                    |state: &mut MaybeUninit<SimulationClockState>, allocator: &mut BumpAllocator| {
                        state.write(SimulationClockState {
                            creation_time: self.config.creation_time,
                            nanoseconds_since_creation: AtomicU64::new(0),
                            scheduled_timepoints: unsafe {
                                RelocatableVec::<RelocatableByteAtomic<ScheduledTimepoint>>::new_uninit(capacity)
                            },
                            generation_counters: unsafe { RelocatableVec::<AtomicU64>::new_uninit(capacity) },
                            index_list: unsafe { UniqueIndexSet::new_uninit(capacity) },
                            global_generation_counter: AtomicU64::new(1),
                            name_generator_counter: AtomicU64::new(0),
                            has_active_time_controller: AtomicBool::new(false),
                        });
                        let state = unsafe { state.assume_init_mut() };

                        if unsafe { state.scheduled_timepoints.init(allocator) }.is_err()
                            || unsafe { state.index_list.init(allocator) }.is_err()
                            || unsafe { state.generation_counters.init(allocator) }.is_err()
                        {
                            debug!(from self, "Unable to initialize the simulation clock state.");
                            return false;
                        }

                        for i in 0..capacity {
                            unsafe {
                                state
                                    .scheduled_timepoints
                                    .push_unchecked(RelocatableByteAtomic::<ScheduledTimepoint>::new_uninit());
                                if state.scheduled_timepoints[i].init(allocator, ScheduledTimepoint {
                                    trigger_name: FileName::new_unchecked_const(b"empty"),
                                    timeout_in_ns_after_creation: 0,

                                }).is_err() {
                                    debug!(from self, "Unable to initialize the simulation clock state.");
                                    return false;
                                }
                                state.generation_counters.push_unchecked(AtomicU64::new(0));
                            }
                        }

                        true
                    },
                )
                .open_or_create(), // TODO iox2-2100: more fine-grained error handling
                with TimeControllerBuilderOpenOrCreateError::InternalError,
                "Unable to open/create the simulation clock state."))
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SimulationClock<E: Event> {
    name: FileName,
    state: Storage<SimulationClockState>,
    event_config: E::Configuration,
}

unsafe impl<E: Event> Send for SimulationClock<E> {}
unsafe impl<E: Event> Sync for SimulationClock<E> {}
impl<E: Event> SteppedClock for SimulationClock<E> {}

impl<E: Event> Abandonable for SimulationClock<E>
where
    E::Listener: SynchronousMultiplexing,
{
    // TODO iox2-2100: Add a `NodeDeathTest` that simulates the crash of a time controller.
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            <Storage<SimulationClockState> as Abandonable>::abandon_in_place(NonNull::from_mut(
                &mut this.state,
            ))
        }
    }
}

impl<E: Event> ClockListenerSchedule for SimulationClock<E> {
    fn schedule_on<'a>(
        &'a self,
        timeout: Timeout,
        listener_name: &FileName,
    ) -> Result<ScheduleOnHandle<'a>, ClockListenerScheduleError> {
        let msg = "Failed to schedule a timeout on a listener";
        let state = self.state.get();
        let timeout = self.translate_timeout(timeout);

        if timeout <= self.now_impl() {
            let notifier = fail!(from self, when create_notifier::<E>(listener_name, &self.event_config, self, msg),
                        with ClockListenerScheduleError::NotificationFailed,
                        "{msg} since the timeout has already expired and the notifier could not be created.");
            fail!(from self, when notifier.notify(EventId::new(0)),
                        with ClockListenerScheduleError::NotificationFailed,
                        "{msg} since the timeout has already expired but notifying the listener failed.");
            return Ok(ScheduleOnHandle::TranslatedTimeout(Timeout::By(
                Duration::ZERO,
            )));
        }

        let counter = state
            .global_generation_counter
            .fetch_add(1, Ordering::SeqCst);

        let index = fail!(from self, when self.write_state(timeout, listener_name, counter),
            with ClockListenerScheduleError::ClockScheduleCapacityExceeded,
            "Timeout cannot be scheduled because the capacity would be exceeded.");

        // Check `now` again because the timeout could have expired while writing the clock state. If so, notify
        // the listener.
        if timeout <= self.now_impl() {
            if state.generation_counters[index as usize]
                .compare_exchange(counter, 0, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                unsafe {
                    state
                        .index_list
                        .release_raw_index(index, ReleaseMode::Default);
                }
                let notifier = fail!(from self, when create_notifier::<E>(listener_name, &self.event_config, self, msg),
                            with ClockListenerScheduleError::NotificationFailed,
                            "{msg} since the timeout has already expired and the notifier could not be created.");
                fail!(from self, when notifier.notify(EventId::new(0)),
                            with ClockListenerScheduleError::NotificationFailed,
                            "{msg} since the timeout has already expired but notifying the listener failed.");
            }
            return Ok(ScheduleOnHandle::TranslatedTimeout(Timeout::By(
                Duration::ZERO,
            )));
        }

        let guard = Guard {
            generation_counter: counter,
            index,
            generation_counters: &state.generation_counters,
            index_list: &state.index_list,
        };
        Ok(ScheduleOnHandle::Guard(Box::new(guard)))
    }
}

impl<E: Event + 'static> ClockReactorSchedule for SimulationClock<E>
where
    E::Listener: SynchronousMultiplexing,
{
    fn schedule<'a>(
        &'a self,
        timeout: Timeout,
    ) -> Result<ScheduleHandle<'a>, ClockReactorScheduleError> {
        let msg = "Timeout cannot be scheduled";

        let state = self.state.get();
        let timeout = self.translate_timeout(timeout);

        let id = state.name_generator_counter.fetch_add(1, Ordering::Relaxed);
        let listener_name =
            unsafe { FileName::new_unchecked_const(format!("{}_{}", self.name, id).as_bytes()) };
        let listener = match create_reactor_attachment::<E>(&listener_name, &self.event_config) {
            Ok(l) => l,
            Err(e) => {
                fail!(from self, with ClockReactorScheduleError::ListenerAttachmentFailed,
                    "{msg} because attaching the listener to the reactor failed: {e:?}");
            }
        };

        if timeout <= self.now_impl() {
            return Ok(ScheduleHandle::TranslatedTimeout(Timeout::By(
                Duration::ZERO,
            )));
        }

        let counter = state
            .global_generation_counter
            .fetch_add(1, Ordering::SeqCst);

        let index = fail!(from self, when self.write_state(timeout, &listener_name, counter),
            with ClockReactorScheduleError::ClockScheduleCapacityExceeded,
            "Timeout cannot be scheduled because the capacity would be exceeded.");

        // Check `now` again because the timeout could have expired while writing the clock state.
        if timeout <= self.now_impl() {
            if state.generation_counters[index as usize]
                .compare_exchange(counter, 0, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                unsafe {
                    state
                        .index_list
                        .release_raw_index(index, ReleaseMode::Default);
                };
            }
            return Ok(ScheduleHandle::TranslatedTimeout(Timeout::By(
                Duration::ZERO,
            )));
        }

        let guard = Guard {
            generation_counter: counter,
            index,
            generation_counters: &state.generation_counters,
            index_list: &state.index_list,
        };

        Ok(ScheduleHandle::GuardAndListener((
            Box::new(guard),
            listener,
        )))
    }
}

impl<E: Event> ClockNow for SimulationClock<E> {
    fn now(&self) -> Result<TimePoint, ClockNowError> {
        Ok(self.now_impl())
    }
}

impl<E: Event> SimulationClock<E> {
    fn translate_timeout(&self, timeout: Timeout) -> TimePoint {
        match timeout {
            Timeout::At(timepoint) => timepoint,
            Timeout::By(duration) => self.now_impl() + duration,
        }
    }

    fn write_state(
        &self,
        timeout: TimePoint,
        listener_name: &FileName,
        generation_counter: u64,
    ) -> Result<u32, ()> {
        let timepoint = ScheduledTimepoint {
            trigger_name: *listener_name,
            timeout_in_ns_after_creation: timeout.as_nanos() as u64,
        };

        let index = match unsafe { self.state.get().index_list.acquire_raw_index() } {
            Ok(idx) => idx,
            Err(_) => {
                fail!(from self, with (),
                    "Trigger {} cannot be scheduled since it would exceed the max. amount of triggers.",
                    listener_name);
            }
        };

        self.state.get().scheduled_timepoints[index as usize].write(timepoint);
        // SYNC POINT: write scheduled_timepoints
        self.state.get().generation_counters[index as usize]
            .store(generation_counter, Ordering::SeqCst);
        Ok(index)
    }

    fn now_impl(&self) -> TimePoint {
        let state = self.state.get();
        state.creation_time
            + Duration::from_nanos(state.nanoseconds_since_creation.load(Ordering::SeqCst))
    }
}

impl<E: Event + 'static> Clock<E> for SimulationClock<E>
where
    E::Listener: SynchronousMultiplexing,
{
    type Configuration = TimeControllerConfig;
    type Builder = SimulationClockBuilder<E>;
    const HAS_TIME_CONTROLLER_SUPPORT: bool = true;

    fn blocking_sleep(&self, timeout: Timeout) -> Result<(), ClockBlockingSleepError> {
        let msg = "Blocking sleep failed";
        let id = self
            .state
            .get()
            .name_generator_counter
            .fetch_add(1, Ordering::Relaxed);
        let listener_name =
            unsafe { FileName::new_unchecked_const(format!("{}_{}", self.name, id).as_bytes()) };
        let listener = match create_waiter::<E>(&listener_name, &self.event_config) {
            Ok(l) => l,
            Err(e) => {
                fail!(from self, with ClockBlockingSleepError::ScheduleTimeoutFailed,
                    "{msg} because the listener could not be created: {e:?}");
            }
        };
        let guard = fail!(from self, when self.schedule_on(timeout, &listener_name),
            with ClockBlockingSleepError::ScheduleTimeoutFailed,
            "{msg} because the timeout could not be scheduled.");
        match guard {
            ScheduleOnHandle::TranslatedTimeout(_) => {
                // timeout has already expired
                Ok(())
            }
            ScheduleOnHandle::Guard(_g) => {
                match listener.blocking_wait(&mut |_| {}) {
                    Ok(_) => Ok(()),
                    Err(_) => {
                        // TODO iox2-2100: more fine-grained error handling
                        fail!(from self, with ClockBlockingSleepError::InternalError, "");
                    }
                }
            }
        }
    }
}

pub const TIME_CONTROLLER_STATE_NAME: FileName =
    unsafe { FileName::new_unchecked_const(b"time_controller") };

impl<E: Event> TimeControllerBuilder for SimulationClockBuilder<E> {
    type TimeController = SimulationTimeController<E>;

    fn create_time_controller(
        self,
    ) -> Result<SimulationTimeController<E>, TimeControllerBuilderCreateError> {
        let origin = "SimulationTimeControllerBuilder::create_time_controller()";
        let msg = "Unable to create the time controller for the simulation clock";

        match self.open_or_create_time_controller() {
            Ok(time_controller) => Ok(time_controller),
            Err(TimeControllerBuilderOpenOrCreateError::AlreadyOpenedByAnotherInstance) => {
                fail!(from origin, with TimeControllerBuilderCreateError::AlreadyCreatedByAnotherInstance,
                    "{msg} because it already exists.");
            }
            Err(TimeControllerBuilderOpenOrCreateError::InternalError) => {
                fail!(from origin, with TimeControllerBuilderCreateError::InternalError,
                    "{msg} due to an internal error.");
            }
        }
    }

    fn open_time_controller(
        self,
    ) -> Result<SimulationTimeController<E>, TimeControllerBuilderOpenError> {
        let origin = "SimulationTimeControllerBuilder::open_time_controller()";
        let msg = "Unable to open the time controller for the simulation clock";

        let storage =
            <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(
                &self.name,
            )
            .config(&self.config.storage_config)
            .open(AccessMode::ReadWrite);
        match storage {
            Err(DynamicStorageOpenError::DoesNotExist) => {
                fail!(from origin, with TimeControllerBuilderOpenError::TimeControllerDoesNotExist,
                    "{msg} because it does not exist.");
            }
            Err(e) => {
                fail!(from origin, with TimeControllerBuilderOpenError::InternalError,
                    "{msg} because the clock state storage could not be opened: {e:?}");
            }
            Ok(s) => {
                match s.get().has_active_time_controller.compare_exchange(
                    false,
                    true,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => Ok(SimulationTimeController {
                        clock_state: s,
                        event_config: self.event_config,
                    }),
                    Err(_) => {
                        fail!(from origin, with TimeControllerBuilderOpenError::AlreadyOpenedByAnotherInstance,
                            "{msg} because it is already opened.");
                    }
                }
            }
        }
    }

    fn open_or_create_time_controller(
        self,
    ) -> Result<SimulationTimeController<E>, TimeControllerBuilderOpenOrCreateError> {
        let origin = "SimulationTimeControllerBuilder::open_or_create_time_controller()";
        let msg = "Unable to open/create the time controller for the simulation clock";

        // Create and initialize clock storage
        let storage = fail!(from origin, when self.open_or_create_clock_state_storage(),
            "{msg} because the clock state could not be opened/created.");
        match storage.get().has_active_time_controller.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => Ok(SimulationTimeController {
                clock_state: storage,
                event_config: self.event_config,
            }),
            Err(_) => {
                fail!(from origin, with TimeControllerBuilderOpenOrCreateError::AlreadyOpenedByAnotherInstance,
                            "{msg} because it already exists.");
            }
        }
    }
}

#[derive(Debug)]
pub struct SimulationTimeController<E: Event> {
    clock_state: Storage<SimulationClockState>,
    event_config: E::Configuration,
}

impl<E: Event> Drop for SimulationTimeController<E> {
    fn drop(&mut self) {
        self.clock_state
            .get()
            .has_active_time_controller
            .store(false, Ordering::SeqCst);
    }
}

impl<E: Event> Abandonable for SimulationTimeController<E> {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            <Storage<SimulationClockState> as Abandonable>::abandon_in_place(NonNull::from_mut(
                &mut this.clock_state,
            ))
        }
    }
}

impl<E: Event> TimeController for SimulationTimeController<E> {
    type E = E;

    fn advance(&self, duration: Duration) -> Result<TimePoint, TimeControllerAdvanceError> {
        let msg = "Failed to advance time";

        let state = self.clock_state.get();
        let old_simulated_time = state
            .nanoseconds_since_creation
            .fetch_add(duration.as_nanos() as u64, Ordering::SeqCst);

        // iterate scheduled time points and notify
        let capacity = state.scheduled_timepoints.capacity();
        for i in 0..capacity {
            // SYNC POINT: read scheduled_timepoints
            let counter = state.generation_counters[i].load(Ordering::SeqCst);
            if counter == 0 {
                continue;
            }

            let timepoint = state.scheduled_timepoints[i].read();
            if state.generation_counters[i].load(Ordering::SeqCst) != counter {
                // guard deregistered trigger
                continue;
            }
            let timepoint = unsafe { timepoint.assume_consistent() };
            let trigger_name = timepoint.trigger_name;
            let timeout = timepoint.timeout_in_ns_after_creation;

            if old_simulated_time + duration.as_nanos().min(u64::MAX as u128) as u64 >= timeout {
                let notifier = fail!(from self,
                    when create_notifier::<E>(&trigger_name, &self.event_config, self, msg),
                    with TimeControllerAdvanceError::NotificationFailed,
                    "{msg} because a notifier could not be created.");
                fail!(from self, when notifier.notify(EventId::new(0)),
                    with TimeControllerAdvanceError::NotificationFailed,
                    "{msg} because a scheduled notification failed.");
                if state.generation_counters[i]
                    .compare_exchange(counter, 0, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    unsafe {
                        state
                            .index_list
                            .release_raw_index(i as u32, ReleaseMode::Default)
                    };
                }
            }
        }

        Ok(TimePoint::from_nanos(old_simulated_time as u128))
    }
}

impl<E: Event> NamedConceptMgmt for SimulationTimeController<E> {
    type Configuration = TimeControllerConfig;

    unsafe fn remove_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        let config = crate::dynamic_storage::posix_shared_memory::Configuration::<
            SimulationClockState,
        >::default();
        let config = config.suffix(cfg.get_suffix());
        unsafe { Storage::remove_cfg(name, &config) }
    }

    fn list_cfg(
        cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        let config = crate::dynamic_storage::posix_shared_memory::Configuration::<
            SimulationClockState,
        >::default();
        let config = config.suffix(cfg.get_suffix());
        Storage::list_cfg(&config)
    }

    fn does_exist_cfg(
        name: &FileName,
        cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        let config = crate::dynamic_storage::posix_shared_memory::Configuration::<
            SimulationClockState,
        >::default();
        let config = config.suffix(cfg.get_suffix());
        Storage::does_exist_cfg(name, &config)
    }

    fn remove_path_hint(
        _value: &Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        Ok(())
    }
}
