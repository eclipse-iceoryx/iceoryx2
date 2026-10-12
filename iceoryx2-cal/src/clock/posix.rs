use crate::{
    clock::{
        Clock, ClockBlockingSleepError, ClockBuilder, ClockBuilderAcquireError,
        ClockBuilderTimeControllerError, ClockNowError, ScheduleGuard, ScheduleHandle,
        TimeController, TimeControllerBuilder, Timeout,
    },
    event::{InternalStubEvent, NamedConceptBuilder, NamedConceptMgmt},
    named_concept::NamedConceptConfiguration,
};
use core::time::Duration;
use iceoryx2_bb_posix::clock::{ClockType, NanosleepError};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_log::fail;

pub struct Guard {}

impl ScheduleGuard for Guard {}

pub struct Builder {}

impl ClockBuilder<PosixClock, PosixTimeController, PosixTimeControllerBuilder> for Builder {
    fn acquire(self) -> Result<PosixClock, ClockBuilderAcquireError> {
        Ok(PosixClock {
            clock_type: ClockType::Monotonic,
        })
    }

    fn time_controller(self) -> Result<PosixTimeController, ClockBuilderTimeControllerError> {
        fail!(from "clock::posix::Builder::time_controller()", 
            with ClockBuilderTimeControllerError::TimeControllerIsNotSupported,
            "The controlling time is not supported for the POSIX clock.");
    }
}

impl NamedConceptBuilder<PosixTimeController> for Builder {
    fn new(_name: &FileName) -> Self {
        Self {}
    }

    fn config(self, _config: &PosixTimeControllerConfig) -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct PosixClock {
    clock_type: ClockType,
}

impl Clock for PosixClock {
    type Guard = Guard;
    type E = InternalStubEvent;
    type TimeController = PosixTimeController;
    type TimeControllerBuilder = PosixTimeControllerBuilder;
    type ClockBuilder = Builder;
    const HAS_TIME_CONTROLLER_SUPPORT: bool = false;

    fn schedule(&self, timeout: super::Timeout) -> (Self::Guard, ScheduleHandle<Self::E>) {
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

#[derive(Debug, Clone, Default)]
pub struct PosixTimeControllerConfig {}

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
        unimplemented!()
    }

    fn path_hint(self, _value: &iceoryx2_bb_system_types::path::Path) -> Self {
        self
    }

    fn get_path_hint(&self) -> &iceoryx2_bb_system_types::path::Path {
        unimplemented!()
    }
}

pub struct PosixTimeControllerBuilder {}

impl TimeControllerBuilder<PosixTimeController> for PosixTimeControllerBuilder {
    fn create(self) -> Result<PosixTimeController, super::TimeControllerCreateError> {
        unimplemented!()
    }

    fn open(self) -> Result<PosixTimeController, super::TimeControllerOpenError> {
        unimplemented!()
    }

    fn open_or_create(self) -> Result<PosixTimeController, super::TimeControllerOpenOrCreateError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct PosixTimeController {}

impl TimeController for PosixTimeController {
    type E = InternalStubEvent;

    fn advance(&self, _duration: Duration) -> iceoryx2_bb_si_units::TimePoint {
        unimplemented!()
    }
}

impl NamedConceptMgmt for PosixTimeController {
    type Configuration = PosixTimeControllerConfig;

    unsafe fn remove_cfg(
        _name: &FileName,
        _cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptRemoveError> {
        unimplemented!()
    }

    fn does_exist_cfg(
        _name: &FileName,
        _cfg: &Self::Configuration,
    ) -> Result<bool, crate::named_concept::NamedConceptDoesExistError> {
        unimplemented!()
    }

    fn list_cfg(
        _cfg: &Self::Configuration,
    ) -> Result<Vec<FileName>, crate::named_concept::NamedConceptListError> {
        unimplemented!()
    }

    fn remove_path_hint(
        _value: &iceoryx2_bb_system_types::path::Path,
    ) -> Result<(), crate::named_concept::NamedConceptPathHintRemoveError> {
        unimplemented!()
    }
}
