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
        Clock, ClockBlockingSleepError, ClockBuilder, ClockBuilderAcquireError, ClockNowError,
        FreeRunningClock, ScheduleGuard, ScheduleHandle, Timeout,
    },
    event::{Event, InternalStubEvent},
    named_concept::NamedConceptConfiguration,
};
use core::marker::PhantomData;
use iceoryx2_bb_posix::clock::{ClockType, NanosleepError};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_log::fail;

pub struct Guard {}

impl ScheduleGuard for Guard {}

#[derive(Debug, Clone, Copy)]
pub struct PosixTimeControllerConfig {
    clock_type: ClockType,
    stub_suffix: FileName,
}

impl Default for PosixTimeControllerConfig {
    fn default() -> Self {
        Self {
            clock_type: ClockType::default(),
            stub_suffix: PosixClock::<InternalStubEvent>::default_suffix(),
        }
    }
}

impl PosixTimeControllerConfig {
    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self
    }
}

impl NamedConceptConfiguration for PosixTimeControllerConfig {
    fn prefix(self, _value: &FileName) -> Self {
        self
    }

    fn get_prefix(&self) -> &FileName {
        unimplemented!()
    }

    fn suffix(self, _value: &FileName) -> Self {
        self
    }

    fn get_suffix(&self) -> &FileName {
        &self.stub_suffix
    }

    fn path_hint(self, _value: &iceoryx2_bb_system_types::path::Path) -> Self {
        self
    }

    fn get_path_hint(&self) -> &iceoryx2_bb_system_types::path::Path {
        unimplemented!()
    }
}

pub struct Builder {
    config: PosixTimeControllerConfig,
}

impl<E: Event> ClockBuilder<E, PosixClock<E>> for Builder {
    fn new(_name: &FileName) -> Self {
        Self {
            config: PosixTimeControllerConfig::default(),
        }
    }

    fn config(mut self, config: &PosixTimeControllerConfig) -> Self {
        self.config = *config;
        self
    }

    fn open_clock(self) -> Result<PosixClock<E>, ClockBuilderAcquireError> {
        Ok(PosixClock {
            clock_type: self.config.clock_type,
            _event: PhantomData,
        })
    }
}

#[derive(Debug)]
pub struct PosixClock<E: Event> {
    clock_type: ClockType,
    _event: PhantomData<E>,
}

impl<E: Event> FreeRunningClock for PosixClock<E> {}

impl<E: Event> Clock<E> for PosixClock<E> {
    type Guard = Guard;
    type Configuration = PosixTimeControllerConfig;
    type ClockBuilder = Builder;
    const HAS_TIME_CONTROLLER_SUPPORT: bool = false;

    fn schedule(&self, timeout: super::Timeout) -> (Self::Guard, ScheduleHandle<E>) {
        (Guard {}, ScheduleHandle::TranslatedTimeout(timeout))
    }

    fn schedule_on(
        &self,
        _timeout: super::Timeout,
        _trigger_name: &iceoryx2_bb_system_types::file_name::FileName,
    ) -> Self::Guard {
        Guard {}
    }

    fn blocking_sleep(&self, timeout: Timeout) -> Result<(), ClockBlockingSleepError> {
        let msg = "Failed to blocking sleep with";

        let duration = match timeout {
            Timeout::By(duration) => duration,
            Timeout::At(v) => {
                let now = match iceoryx2_bb_posix::clock::Time::now_with_clock(self.clock_type) {
                    Ok(v) => v,
                    Err(e) => {
                        fail!(from self, with ClockBlockingSleepError::InternalError,
                            "{msg} {timeout:?} due to an internal error while acquiring the current time. [{e:?}]");
                    }
                };

                if v < now.time_point() {
                    return Ok(());
                }

                now.time_point().time_span_between(v).into()
            }
        };

        match iceoryx2_bb_posix::clock::nanosleep_with_clock(duration, self.clock_type) {
            Ok(()) => Ok(()),
            Err(NanosleepError::InterruptedBySignal(_)) => {
                fail!(from self, with ClockBlockingSleepError::InterruptedBySignal,
                            "{msg} {timeout:?} since the sleep function was interrupted by a signal.");
            }
            Err(NanosleepError::DurationOutOfRange) => {
                fail!(from self, with ClockBlockingSleepError::DurationOutOfRange,
                            "{msg} {timeout:?} since the duration is out of range.");
            }
            Err(e) => {
                fail!(from self, with ClockBlockingSleepError::InternalError,
                            "{msg} {timeout:?} due to an internal error. [{e:?}]");
            }
        }
    }

    fn now(&self) -> Result<iceoryx2_bb_si_units::TimePoint, ClockNowError> {
        let msg = "Failed to acquire the current time";
        match iceoryx2_bb_posix::clock::Time::now_with_clock(self.clock_type) {
            Ok(v) => Ok(v.time_point()),
            Err(e) => {
                fail!(from self, with ClockNowError::InternalError,
                "{msg} due to an internal error. [{e:?}]");
            }
        }
    }
}
