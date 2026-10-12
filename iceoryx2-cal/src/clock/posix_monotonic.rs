use iceoryx2_bb_posix::clock::{ClockType, NanosleepError};
use iceoryx2_log::fail;

use crate::{
    clock::{Clock, ClockBlockingSleepError, ScheduleGuard, ScheduleHandle, Timeout},
    event::{InternalStubEvent, NamedConceptMgmt},
};

pub struct Guard {}

impl ScheduleGuard for Guard {}

pub struct Builder {}

#[derive(Debug)]
pub struct PosixMonotonicClock {}

impl Clock for PosixMonotonicClock {
    type Guard = Guard;
    type E = InternalStubEvent;

    fn schedule(&self, timeout: super::Timeout) -> (Self::Guard, ScheduleHandle<Self::E>) {
        (Guard {}, ScheduleHandle::TranslatedTimeout(timeout))
    }

    fn schedule_on(
        &self,
        timeout: super::Timeout,
        _trigger_name: &iceoryx2_bb_system_types::file_name::FileName,
    ) -> (Self::Guard, super::Timeout) {
        (Guard {}, timeout)
    }

    fn blocking_sleep(&self, timeout: Timeout) -> Result<(), ClockBlockingSleepError> {
        let msg = "Failed to blocking sleep with";
        match timeout {
            Timeout::By(duration) => {
                match iceoryx2_bb_posix::clock::nanosleep_with_clock(duration, ClockType::Monotonic)
                {
                    Ok(()) => (),
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
                Ok(())
            }
            Timeout::At(v) => {
                todo!()
            }
        }
    }

    fn now(&self) -> Result<iceoryx2_bb_si_units::TimePoint, ()> {
        iceoryx2_bb_posix::clock::Time::now_with_clock(ClockType::Monotonic);
        todo!()
    }
}

pub struct TimeControllerBuilder {}

pub struct TimeController {}
