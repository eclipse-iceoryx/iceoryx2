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
        ScheduleOnHandle, SteppedClock, TimeController, TimeControllerBuilder,
        TimeControllerBuilderCreateError, TimeControllerBuilderOpenError,
        TimeControllerBuilderOpenOrCreateError, Timeout,
        internal::{create_reactor_attachment, create_waiter},
    },
    dynamic_storage::{DynamicStorage, DynamicStorageBuilder, posix_shared_memory::Storage},
    event::{Event, EventId, Notifier, NotifierBuilder, UnixDatagramShmCountingBitSet},
    named_concept::{NamedConceptBuilder, NamedConceptConfiguration, NamedConceptMgmt},
};
use core::{marker::PhantomData, mem::MaybeUninit, ptr::NonNull, time::Duration};
use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64, Ordering};
use iceoryx2_bb_container::{
    queue::RelocatableContainer,
    vector::{RelocatableVec, Vector},
};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::{
    testing::abandonable::Abandonable, zero_copy_send::ZeroCopySend,
};
use iceoryx2_bb_lock_free::mpmc::{
    unique_index_set::UniqueIndexSet, unique_index_set_enums::ReleaseMode,
};
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::{file::AccessMode, file_descriptor_set::SynchronousMultiplexing};
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_system_types::{file_name::FileName, path::Path};
use iceoryx2_log::{error, fail, fatal_panic};

// TODO: refactor error handling

pub struct Guard<'a> {
    generation_counter: u64,
    index: u32,
    scheduled_timepoints: &'a RelocatableVec<ScheduledTimepoint>,
    index_list: &'a UniqueIndexSet,
}

impl<'a> ScheduleGuard<'a> for Guard<'a> {}

impl<'a> Drop for Guard<'a> {
    fn drop(&mut self) {
        if self.scheduled_timepoints[self.index as usize]
            .generation_counter
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
// TODO: generic for storage
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
            capacity: 100, // TODO: const
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
#[derive(Debug, ZeroCopySend)]
pub(crate) struct ScheduledTimepoint {
    pub(crate) trigger_name: FileName,
    pub(crate) timeout_in_ns_after_creation: u64,
    pub(crate) generation_counter: AtomicU64,
}

#[repr(C)]
#[derive(Debug, ZeroCopySend)]
pub struct SimulationClockState {
    creation_time: TimePoint,
    nanoseconds_since_creation: AtomicU64,
    scheduled_timepoints: RelocatableVec<ScheduledTimepoint>,
    index_list: UniqueIndexSet,
    global_generation_counter: AtomicU64,
    name_id: AtomicU64,
}

#[derive(Debug)]
pub struct SimulationClockBuilder<E: Event> {
    name: FileName,
    config: TimeControllerConfig,
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
            _event: PhantomData,
        }
    }

    fn config(mut self, config: &TimeControllerConfig) -> Self {
        self.config = config.clone();
        self
    }

    fn open_clock(self) -> Result<SimulationClock<E>, ClockBuilderOpenClockError> {
        match SimulationTimeControllerBuilder::new(&self.name, &self.config)
            .open_or_create_clock_state_storage()
        {
            Ok(storage) => Ok(SimulationClock {
                name: self.name,
                state: storage,
                _event: PhantomData,
            }),
            Err(_) => {
                fail!(from self, with ClockBuilderOpenClockError::InternalError,
                    "Unable to open the simulation clock due to an internal error while
                    opening/creating the clock state.");
            }
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SimulationClock<E: Event> {
    name: FileName,
    state: Storage<SimulationClockState>,
    _event: PhantomData<E>,
}

unsafe impl<E: Event> Send for SimulationClock<E> {}
unsafe impl<E: Event> Sync for SimulationClock<E> {}
impl<E: Event> SteppedClock for SimulationClock<E> {}

impl<E: Event> Abandonable for SimulationClock<E>
where
    E::Listener: SynchronousMultiplexing,
{
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
        let counter = self
            .state
            .get()
            .global_generation_counter
            .fetch_add(1, Ordering::SeqCst);

        let timeout = match timeout {
            Timeout::At(timepoint) => timepoint,
            Timeout::By(duration) => self.state.get().creation_time + duration,
        };
        let timepoint = ScheduledTimepoint {
            trigger_name: listener_name.clone(),
            timeout_in_ns_after_creation: timeout.as_nanos() as u64, // TODO: make u128?
            generation_counter: AtomicU64::new(counter),
        };

        match self.now() {
            Err(_) => fatal_panic!("This should never happen! Failed to obtain current time."),
            Ok(tp) => {
                if timeout < tp {
                    // TODO: trigger instead of error
                    fatal_panic!(
                        "Trigger {} cannot be scheduled since the timeout is already expired.",
                        listener_name
                    );
                }
            }
        }

        let index = match unsafe { self.state.get().index_list.acquire_raw_index() } {
            Ok(idx) => idx,
            Err(_) => fatal_panic!(
                "Trigger {} cannot be scheduled since it would exceed the max. amount of triggers.",
                listener_name
            ), // TODO: return error
        };

        unsafe {
            self.state.get_mut().scheduled_timepoints[index as usize] = timepoint;
        }

        match self.now() {
            Err(_) => {
                unsafe {
                    self.state.get().scheduled_timepoints[index as usize]
                        .generation_counter
                        .store(0, Ordering::SeqCst);
                    self.state
                        .get()
                        .index_list
                        .release_raw_index(index, ReleaseMode::Default);
                }
                fatal_panic!("This should never happen! Failed to obtain current time.");
            }
            Ok(tp) => {
                if timeout < tp {
                    match self.state.get().scheduled_timepoints[index as usize]
                        .generation_counter
                        .compare_exchange(counter, 0, Ordering::SeqCst, Ordering::SeqCst)
                    {
                        Ok(_) => unsafe {
                            self.state
                                .get()
                                .index_list
                                .release_raw_index(index, ReleaseMode::Default);
                        },
                        Err(_) => {
                            // advance() released index
                        }
                    };
                    // TODO: no error, trigger when schedule() set counter to 0
                    fatal_panic!(
                        "Trigger {} cannot be scheduled since the timeout is already expired.",
                        listener_name
                    );
                }
            }
        }

        let guard = Guard {
            generation_counter: counter,
            index,
            scheduled_timepoints: &self.state.get().scheduled_timepoints,
            index_list: &self.state.get().index_list,
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
        let id = self.state.get().name_id.fetch_add(1, Ordering::Relaxed);
        let listener_name =
            unsafe { FileName::new_unchecked_const(format!("{}_{}", self.name, id).as_bytes()) };
        let listener =
            create_reactor_attachment::<E>(&listener_name, &E::Configuration::default()).unwrap();
        let guard = self.schedule_on(timeout, &listener_name).unwrap();
        match guard {
            ScheduleOnHandle::TranslatedTimeout(_) => fatal_panic!(""),
            ScheduleOnHandle::Guard(g) => Ok(ScheduleHandle::GuardAndListener((g, listener))),
        }
    }
}

impl<E: Event> ClockNow for SimulationClock<E> {
    fn now(&self) -> Result<TimePoint, ClockNowError> {
        let state = self.state.get();
        Ok(state.creation_time
            + Duration::from_nanos(state.nanoseconds_since_creation.load(Ordering::Relaxed)))
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
        // TODO: unique name
        let listener_name = unsafe { FileName::new_unchecked_const(b"blub") };
        let listener = create_waiter::<E>(&listener_name, &E::Configuration::default()).unwrap();
        let guard = self.schedule_on(timeout, &listener_name).unwrap();
        match guard {
            ScheduleOnHandle::TranslatedTimeout(_) => {
                fail!(from self, with ClockBlockingSleepError::InternalError, "");
            }
            ScheduleOnHandle::Guard(_) => match listener.blocking_wait(&mut |_| {}) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    fail!(from self, with ClockBlockingSleepError::InternalError, "");
                }
            },
        }
    }
}

pub const TIME_CONTROLLER_STATE_NAME: FileName =
    unsafe { FileName::new_unchecked_const(b"timer_controller") };

#[derive(Debug)]
pub struct SimulationTimeControllerBuilder {
    name: FileName,
    config: TimeControllerConfig,
}

impl SimulationTimeControllerBuilder {
    pub fn new(name: &FileName, config: &TimeControllerConfig) -> Self {
        Self {
            name: *name,
            config: config.clone(),
        }
    }
}

impl TimeControllerBuilder<SimulationTimeController> for SimulationTimeControllerBuilder {
    fn create_time_controller(
        self,
    ) -> Result<SimulationTimeController, TimeControllerBuilderCreateError> {
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
    ) -> Result<SimulationTimeController, TimeControllerBuilderOpenError> {
        let origin = "SimulationTimeControllerBuilder::open_time_controller()";
        let msg = "Unable to open the time controller for the simulation clock";

        let time_controller_state =
            match <Storage<AtomicBool> as DynamicStorage<AtomicBool>>::Builder::new(
                &TIME_CONTROLLER_STATE_NAME,
            )
            .config(&self.state_config())
            .open(AccessMode::ReadWrite)
            {
                Ok(storage) => storage,
                Err(_) => {
                    fail!(from origin, with TimeControllerBuilderOpenError::TimeControllerDoesNotExist,
                        "{msg} because its state does not exist.");
                }
            };
        if time_controller_state.get().load(Ordering::SeqCst) {
            fail!(from origin, with TimeControllerBuilderOpenError::AlreadyOpenedByAnotherInstance,
                "{msg} because it is already opened.");
        }

        let storage =
            <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(
                &self.name,
            )
            .config(&self.config.storage_config)
            .open(AccessMode::ReadWrite);
        match storage {
            Err(e) => {
                fail!(from origin, with TimeControllerBuilderOpenError::InternalError,
                    "{msg} because the clock state storage could not be opened: {e:?}");
            }
            Ok(s) => {
                match time_controller_state.get().compare_exchange(
                    false,
                    true,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => Ok(SimulationTimeController {
                        clock_state: s,
                        config: self.config,
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
    ) -> Result<SimulationTimeController, TimeControllerBuilderOpenOrCreateError> {
        let origin = "SimulationTimeControllerBuilder::open_or_create_time_controller()";
        let msg = "Unable to open/create the time controller for the simulation clock";

        // Check if a time controller already exists
        let time_controller_state =
            match <Storage<AtomicBool> as DynamicStorage<AtomicBool>>::Builder::new(
                &TIME_CONTROLLER_STATE_NAME,
            )
            .config(&self.state_config())
            .has_ownership(false)
            .initializer(|_, _| {
                AtomicBool::new(false);
                true
            })
            .open_or_create()
            {
                Ok(storage) => storage,
                Err(e) => {
                    fail!(from origin, with TimeControllerBuilderOpenOrCreateError::InternalError,
                        "{msg} because its state could not be opened/created: {e:?}");
                }
            };
        if time_controller_state.get().load(Ordering::SeqCst) {
            return Err(TimeControllerBuilderOpenOrCreateError::AlreadyOpenedByAnotherInstance);
        }

        // Create and initialize clock storage
        let storage = fail!(from origin, when self.open_or_create_clock_state_storage(),
            "{msg} because the clock state could not be opened/created.");
        match time_controller_state.get().compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(_) => Ok(SimulationTimeController {
                clock_state: storage,
                config: self.config,
            }),
            Err(_) => {
                fail!(from origin, with TimeControllerBuilderOpenOrCreateError::AlreadyOpenedByAnotherInstance,
                            "{msg} because it already exists.");
            }
        }
    }
}

impl SimulationTimeControllerBuilder {
    fn state_config(&self) -> <Storage<AtomicBool> as NamedConceptMgmt>::Configuration {
        <Storage<AtomicBool> as NamedConceptMgmt>::Configuration::default()
            .prefix(self.config.get_prefix())
            .path_hint(self.config.get_path_hint())
    }

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
                RelocatableVec::<ScheduledTimepoint>::const_memory_size(capacity)
                    + UniqueIndexSet::const_memory_size(capacity),
            )
            .initializer(
                |state: &mut MaybeUninit<SimulationClockState>, allocator: &mut BumpAllocator| {
                    state.write(SimulationClockState {
                        creation_time: self.config.creation_time,
                        nanoseconds_since_creation: AtomicU64::new(0),
                        scheduled_timepoints: unsafe {
                            RelocatableVec::<ScheduledTimepoint>::new_uninit(capacity)
                        },
                        index_list: unsafe { UniqueIndexSet::new_uninit(capacity) },
                        global_generation_counter: AtomicU64::new(0),
                        name_id: AtomicU64::new(0),
                    });
                    let state = unsafe { state.assume_init_mut() };

                    if unsafe { state.scheduled_timepoints.init(allocator) }.is_err()
                        || unsafe { state.index_list.init(allocator).is_err() }
                    {
                        error!(from self, "Unable to initialize the simulation clock state.");
                        return false;
                    }

                    for _i in 0..capacity {
                        unsafe {
                            state
                                .scheduled_timepoints
                                .push_unchecked(ScheduledTimepoint {
                                    trigger_name: FileName::new_unchecked_const(b"empty"),
                                    timeout_in_ns_after_creation: 0,
                                    generation_counter: AtomicU64::new(0),
                                });
                        }
                    }

                    true
                },
            )
            .open_or_create(),
            with TimeControllerBuilderOpenOrCreateError::InternalError,
            "Unable to open/create the simulation clock state."))
    }
}

#[derive(Debug)]
pub struct SimulationTimeController {
    clock_state: Storage<SimulationClockState>,
    config: TimeControllerConfig,
}

impl Drop for SimulationTimeController {
    fn drop(&mut self) {
        let state_config = <Storage<AtomicBool> as NamedConceptMgmt>::Configuration::default()
            .prefix(self.config.get_prefix())
            .path_hint(self.config.get_path_hint());
        match <Storage<AtomicBool> as DynamicStorage<AtomicBool>>::Builder::new(
            &TIME_CONTROLLER_STATE_NAME,
        )
        .config(&state_config)
        .open(AccessMode::ReadWrite)
        {
            Ok(storage) => storage.get().store(false, Ordering::SeqCst),
            Err(_) => {}
        };
    }
}

impl Abandonable for SimulationTimeController {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe {
            <Storage<SimulationClockState> as Abandonable>::abandon_in_place(NonNull::from_mut(
                &mut this.clock_state,
            ))
        }
    }
}

impl TimeController for SimulationTimeController {
    type E = UnixDatagramShmCountingBitSet;

    fn advance(&self, duration: Duration) -> TimePoint {
        let state = unsafe { self.clock_state.get_mut() };
        let old_simulated_time = state
            .nanoseconds_since_creation
            .fetch_add(duration.as_nanos() as u64, Ordering::SeqCst);

        // iterate scheduled time points and notify
        let capacity = state.scheduled_timepoints.capacity();
        for i in 0..capacity {
            let counter = state.scheduled_timepoints[i]
                .generation_counter
                .load(Ordering::SeqCst);
            if counter == 0 {
                continue;
            }

            let trigger_name = state.scheduled_timepoints[i].trigger_name.clone();
            let timeout = state.scheduled_timepoints[i].timeout_in_ns_after_creation;
            if state.scheduled_timepoints[i]
                .generation_counter
                .load(Ordering::SeqCst)
                != counter
            {
                // guard deregistered trigger
                continue;
            }

            if old_simulated_time + duration.as_nanos() as u64 >= timeout {
                let notifier = <Self::E as Event>::NotifierBuilder::new(&trigger_name)
                    .open()
                    .unwrap();
                notifier.notify(EventId::new(0)).expect("notify successful");
                if state.scheduled_timepoints[i]
                    .generation_counter
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

        TimePoint::from_nanos(old_simulated_time as u128)
    }
}

impl NamedConceptMgmt for SimulationTimeController {
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
