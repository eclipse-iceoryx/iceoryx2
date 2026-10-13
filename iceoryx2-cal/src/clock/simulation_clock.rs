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
        Clock, ClockBlockingSleepError, ClockBuilder, ClockBuilderAcquireError,
        ClockBuilderTimeControllerError, ClockNowError, ScheduleGuard, ScheduleHandle,
        TimeController, TimeControllerBuilder, TimeControllerBuilderCreateError,
        TimeControllerBuilderOpenError, TimeControllerBuilderOpenOrCreateError, Timeout,
    },
    dynamic_storage::{DynamicStorage, DynamicStorageBuilder, posix_shared_memory::Storage},
    event::UnixDatagramShmCountingBitSet,
    named_concept::{
        NamedConcept, NamedConceptBuilder, NamedConceptConfiguration, NamedConceptMgmt,
    },
};
use core::{mem::MaybeUninit, time::Duration};
use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};
use iceoryx2_bb_container::{
    queue::RelocatableContainer,
    vector::{RelocatableVec, Vector},
};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::unique_index_set::UniqueIndexSet;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_posix::file::AccessMode;
use iceoryx2_bb_si_units::TimePoint;
use iceoryx2_bb_system_types::{
    file_name::FileName,
    path::{Path, SemanticString},
};
use iceoryx2_log::{error, fail, fatal_panic};

pub struct Guard {
    generation_counter: u64,
}

impl ScheduleGuard for Guard {}

#[derive(Debug, Clone)]
// TODO: generic for storage
pub struct TimeControllerConfig {
    storage_config: <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration,
    capacity: usize,
    creation_time: TimePoint,
    time_controller_exists: bool,
}

impl Default for TimeControllerConfig {
    fn default() -> Self {
        let storage_config =
            <Storage<SimulationClockState> as NamedConceptMgmt>::Configuration::default();
        let mut suffix = *storage_config.get_suffix(); // TODO: provide default_suffix() for clock in mod.rs
        fatal_panic!(
            from "Configuration::default()",
            when suffix.insert_bytes(0, format!(".clock_state").as_bytes()),
            "This should never happen! Failed to added suffix to simulation clock state.");
        let storage_config = storage_config.suffix(&suffix);
        Self {
            storage_config,
            capacity: 0,
            creation_time: TimePoint::new(),
            time_controller_exists: false,
        }
    }
}

impl TimeControllerConfig {
    fn capacity(mut self, value: usize) -> Self {
        self.capacity = value;
        self
    }

    fn get_capacity(&self) -> usize {
        self.capacity
    }

    fn creation_time(mut self, value: TimePoint) -> Self {
        self.creation_time = value;
        self
    }

    fn get_creation_time(&self) -> TimePoint {
        self.creation_time
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
struct SimulationClockState {
    creation_time: TimePoint,
    nanoseconds_since_creation: AtomicU64,
    scheduled_timepoints: RelocatableVec<ScheduledTimepoint>,
    index_list: UniqueIndexSet,
}

#[derive(Debug)]
pub struct SimulationClockBuilder {
    name: FileName,
    config: TimeControllerConfig,
}

impl ClockBuilder<SimulationClock, SimulationTimeController, SimulationTimeControllerBuilder>
    for SimulationClockBuilder
{
    fn acquire(self) -> Result<SimulationClock, ClockBuilderAcquireError> {
        Ok(
            fail!(from self, when SimulationClock::open(&self.name, &self.config),
                with ClockBuilderAcquireError::InternalError, "Unable to acquire the simulation clock."),
        )
    }

    fn time_controller(self) -> Result<SimulationTimeController, ClockBuilderTimeControllerError> {
        // TODO: other error codes
        let builder = SimulationTimeControllerBuilder {
            name: self.name,
            config: self.config.clone(),
        };
        // TODO: set config flag (make atomic?), confg cannot be cloned
        let time_controller = fail!(from self, when builder.create(),
            with ClockBuilderTimeControllerError::TimeControllerIsNotSupported,
            "Unable to create simulation time controller");
        Err(ClockBuilderTimeControllerError::TimeControllerIsNotSupported)
    }
}

impl NamedConceptBuilder<SimulationTimeController> for SimulationClockBuilder {
    fn new(name: &FileName) -> Self {
        Self {
            name: *name,
            config: TimeControllerConfig::default(),
        }
    }

    fn config(mut self, config: &TimeControllerConfig) -> Self {
        self.config = config.clone();
        self
    }
}

#[repr(C)]
#[derive(Debug)]
struct SimulationClock {
    name: FileName,
    state: Storage<SimulationClockState>,
}

impl Clock for SimulationClock {
    type Guard = Guard;
    type E = UnixDatagramShmCountingBitSet;
    type TimeController = SimulationTimeController;
    type TimeControllerBuilder = SimulationTimeControllerBuilder;
    type ClockBuilder = SimulationClockBuilder;
    const HAS_TIME_CONTROLLER_SUPPORT: bool = true;

    // TODO: check current simulated time before and after registering timeout -> error
    fn schedule(&self, timeout: Timeout) -> (Self::Guard, ScheduleHandle<Self::E>) {
        // create name + trigger
        // call schedule_on
        unimplemented!()
    }

    // TODO: check current simulated time before and after registering timeout -> error
    fn schedule_on(&self, timeout: Timeout, trigger_name: &FileName) -> Self::Guard {
        let timeout = match timeout {
            Timeout::At(timepoint) => timepoint,
            Timeout::By(duration) => self.state.get().creation_time + duration,
        };
        let timepoint = ScheduledTimepoint {
            trigger_name: trigger_name.clone(),
            timeout_in_ns_after_creation: timeout.as_nanos() as u64, // TODO: make u128?
            generation_counter: AtomicU64::new(1),
        };

        // TODO: release index by Guard/TimeController
        let index = match unsafe { self.state.get().index_list.acquire_raw_index() } {
            Ok(idx) => idx,
            Err(_) => fatal_panic!(""), // TODO: error handling
        };

        unsafe {
            self.state.get_mut().scheduled_timepoints[index as usize] = timepoint;
        }

        Guard {
            generation_counter: 1,
        }
    }

    fn blocking_sleep(&self, timeout: Timeout) -> Result<(), ClockBlockingSleepError> {
        Err(ClockBlockingSleepError::InternalError)
    }

    fn now(&self) -> Result<TimePoint, ClockNowError> {
        Ok(TimePoint::from_nanos(
            self.state
                .get()
                .nanoseconds_since_creation
                .load(Ordering::Relaxed) as u128,
        ))
    }
}

impl SimulationClock {
    // TODO: error handling
    fn new(name: &FileName, config: &TimeControllerConfig) -> Result<Self, ()> {
        let origin = "SimulationClock::new()";
        let msg = "Unable to create the SimulationClock";

        let capacity = config.get_capacity();
        let storage =
            <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(
                name,
            )
            .config(&config.storage_config)
            .supplementary_size(
                RelocatableVec::<ScheduledTimepoint>::const_memory_size(capacity)
                    + UniqueIndexSet::const_memory_size(capacity),
            )
            .initializer(
                |state: &mut MaybeUninit<SimulationClockState>, allocator: &mut BumpAllocator| {
                    state.write(SimulationClockState {
                        creation_time: config.get_creation_time(),
                        nanoseconds_since_creation: AtomicU64::new(0),
                        scheduled_timepoints: unsafe {
                            RelocatableVec::<ScheduledTimepoint>::new_uninit(capacity)
                        },
                        index_list: unsafe { UniqueIndexSet::new_uninit(capacity) },
                    });
                    let state = unsafe { state.assume_init_mut() };

                    if unsafe { state.scheduled_timepoints.init(allocator) }.is_err() || unsafe { state.index_list.init(allocator).is_err() } {
                        error!(from origin, "{msg} because the simulation clock state could not be initialized.");
                        return false;
                    }

                    for _i in 0..capacity {
                        unsafe { state.scheduled_timepoints.push_unchecked(ScheduledTimepoint { trigger_name: FileName::new_unchecked_const(b"empty"), timeout_in_ns_after_creation: 0, generation_counter: AtomicU64::new(0) }); }
                    }

                    true
                },
            )
            .create();

        match storage {
            Ok(s) => Ok(Self {
                name: *name,
                state: s,
            }),
            Err(_) => Err(()),
        }
    }

    fn open(name: &FileName, config: &TimeControllerConfig) -> Result<Self, ()> {
        match <Storage<SimulationClockState> as DynamicStorage<SimulationClockState>>::Builder::new(
            name,
        )
        .config(&config.storage_config)
        .open(AccessMode::ReadWrite)
        {
            Err(_) => Err(()),
            Ok(storage) => Ok(Self {
                name: *name,
                state: storage,
            }),
        }
    }
}

#[derive(Debug)]
pub struct SimulationTimeControllerBuilder {
    name: FileName,
    config: TimeControllerConfig,
}

// fail when flag is true
impl TimeControllerBuilder<SimulationTimeController> for SimulationTimeControllerBuilder {
    fn create(self) -> Result<SimulationTimeController, TimeControllerBuilderCreateError> {
        Ok(SimulationTimeController {
            clock: fail!(from self, when SimulationClock::new(&self.name, &self.config),
            with TimeControllerBuilderCreateError::InternalError, "Unable to create the simulation clock."),
        })
    }

    fn open(self) -> Result<SimulationTimeController, TimeControllerBuilderOpenError> {
        Err(TimeControllerBuilderOpenError::InternalError)
    }

    fn open_or_create(
        self,
    ) -> Result<SimulationTimeController, TimeControllerBuilderOpenOrCreateError> {
        Err(TimeControllerBuilderOpenOrCreateError::InternalError)
    }
}

#[derive(Debug)]
pub struct SimulationTimeController {
    clock: SimulationClock,
}
// TODO: impl drop to set config flag to false?

impl TimeController for SimulationTimeController {
    type E = UnixDatagramShmCountingBitSet;

    fn advance(&self, duration: Duration) -> TimePoint {
        let mut state = unsafe { self.clock.state.get_mut() };
        let old_simulated_time = state
            .nanoseconds_since_creation
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);

        // iterate scheduled time points and notify
        let capacity = state.scheduled_timepoints.capacity();
        for i in 0..capacity {
            let counter = state.scheduled_timepoints[i]
                .generation_counter
                .load(Ordering::Relaxed);
            if counter == 0 {
                continue;
            }

            let trigger_name = state.scheduled_timepoints[i].trigger_name.clone();
            let timeout = state.scheduled_timepoints[i].timeout_in_ns_after_creation;
            if state.scheduled_timepoints[i]
                .generation_counter
                .load(Ordering::Relaxed)
                != counter
            {
                continue;
            }

            // use EventId = 0 for now
            // when triggered, unset generation counter
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

// impl NamedConcept for SimulationClock {
//     fn name(&self) -> &FileName {
//         &self.name
//     }
// }
