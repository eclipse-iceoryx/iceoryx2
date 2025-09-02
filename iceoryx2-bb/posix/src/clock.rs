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

//! Contains POSIX timing related abstractions.
//!
//! * [`Time`] - acquires the current system time and measures the elapsed time
//! * [`ClockType`] - describes certain types of clocks
//! * [`nanosleep()`] & [`nanosleep_with_clock()`] - wait a defined amount of time on a custom
//!   clock
//! * [`AsTimeval`] - trait for easy [`posix::timeval`] conversion, required for low level posix
//!   calls
//! * [`AsTimespec`] - trait for easy [`posix::timespec`] conversion, required for low level posix
//!   calls

use crate::handle_errno;
use crate::system_configuration::Feature;
use core::time::Duration;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum TimeError {
    ClockTypeIsNotSupported,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum NanosleepError {
    InterruptedBySignal(Duration),
    DurationOutOfRange,
    ClockTypeIsNotSupported,
    UnknownError(i32),
}

enum_gen! {
/// Use this more generic error enum when you do not require the fine grained error handling.
/// Every error enum is convertable into this one and on a higher level this is convertable into
/// [`crate::Error`].
    ClockError
  generalization:
    TimeError <= TimeError,
    NanosleepError <= NanosleepError
}

impl From<TimeError> for NanosleepError {
    fn from(t: TimeError) -> Self {
        match t {
            TimeError::ClockTypeIsNotSupported => NanosleepError::ClockTypeIsNotSupported,
            TimeError::UnknownError(v) => NanosleepError::UnknownError(v),
        }
    }
}

/// Represents different low level clocks.
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, ZeroCopySend, Serialize, Deserialize)]
#[repr(C)]
pub enum ClockType {
    /// represents a steady clock which does not change when the system time
    /// is adjusted
    Monotonic,
    /// Clock which represents the current system time. Can change when the  system time is
    /// adjusted.
    Realtime,
}

impl Default for ClockType {
    fn default() -> Self {
        if Feature::MonotonicClock.is_available() {
            ClockType::Monotonic
        } else {
            ClockType::Realtime
        }
    }
}

impl ClockType {
    /// Returns a slice containing all supported [`ClockType`]s
    pub fn all_supported_clocks() -> &'static [ClockType] {
        if Feature::MonotonicClock.is_available() {
            &[ClockType::Monotonic, ClockType::Realtime]
        } else {
            &[ClockType::Realtime]
        }
    }

    fn as_i32(&self) -> i32 {
        match self {
            ClockType::Monotonic => posix::CLOCK_MONOTONIC as _,
            ClockType::Realtime => posix::CLOCK_REALTIME as _,
        }
    }
}

/// Trait to convert constructs which represent time into the c pendant [`posix::timespec`]
pub trait AsTimespec {
    fn as_timespec(&self) -> posix::timespec;
}

impl AsTimespec for Duration {
    fn as_timespec(&self) -> posix::timespec {
        posix::timespec {
            tv_sec: self.as_secs() as posix::time_t,
            tv_nsec: self.subsec_nanos() as posix::long,
        }
    }
}

/// Trait to convert constructs which represent time into the c pendant [`posix::timeval`]
pub trait AsTimeval {
    fn as_timeval(&self) -> posix::timeval;
}

impl AsTimeval for Duration {
    fn as_timeval(&self) -> posix::timeval {
        posix::timeval {
            tv_sec: self.as_secs() as _,
            tv_usec: self.subsec_micros() as _,
        }
    }
}

/// Builder for [`Time`].
///
/// # Examples
/// ```
/// use iceoryx2_bb_posix::clock::*;
/// let my_time = TimeBuilder::new().clock_type(ClockType::Realtime)
///                                 .seconds(123)
///                                 .nanoseconds(456).create();
/// ```
pub struct TimeBuilder {
    time: Time,
}

impl Default for TimeBuilder {
    fn default() -> Self {
        TimeBuilder {
            time: Time {
                clock_type: ClockType::default(),
                seconds: 0,
                nanoseconds: 0,
            },
        }
    }
}

impl TimeBuilder {
    pub fn new() -> TimeBuilder {
        Self::default()
    }

    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.time.clock_type = value;
        self
    }

    pub fn seconds(mut self, value: u64) -> Self {
        self.time.seconds = value;
        self
    }

    pub fn nanoseconds(mut self, value: u32) -> Self {
        self.time.nanoseconds = value;
        self
    }

    pub fn create(self) -> Time {
        self.time
    }
}

/// Represents time under a specified [`ClockType`]
#[repr(C)]
#[derive(
    Default, Clone, Copy, Eq, PartialEq, Hash, Debug, ZeroCopySend, Serialize, Deserialize,
)]
pub struct Time {
    pub(crate) clock_type: ClockType,
    pub(crate) seconds: u64,
    pub(crate) nanoseconds: u32,
}

impl Time {
    /// Returns the current time.
    ///
    /// # Examples
    /// ```ignore
    /// use iceoryx2_bb_posix::clock::*;
    ///
    /// let now: Time = Time::now_with_clock(ClockType::Monotonic).unwrap();
    /// ```
    pub fn now_with_clock(clock_type: ClockType) -> Result<Self, TimeError> {
        let mut current_time = posix::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };

        handle_errno!(TimeError, from "Time::now",
            errno_source unsafe { posix::clock_gettime(clock_type.as_i32() as _, &mut current_time).into() },
            success Errno::ESUCCES => Time { clock_type,
                                             seconds: current_time.tv_sec as u64,
                                             nanoseconds: current_time.tv_nsec as u32},
            Errno::ENOSYS => (ClockTypeIsNotSupported, "Failed to get time {{ clock_type: {:?} }} since the clock is not supported.", clock_type),
            v => (UnknownError(v as i32), "Failed to get time {{ clock_type: {:?} }} since an unknown error occurred ({}).", clock_type, v)
        );
    }

    /// Returns the current time with [`ClockType::default()`]
    pub fn now() -> Result<Self, TimeError> {
        Self::now_with_clock(ClockType::default())
    }

    /// Returns the elapsed time which has passed between Time and now as [`Duration`].
    ///
    /// # Examples
    /// ```
    /// use iceoryx2_bb_posix::clock::*;
    /// use core::time::Duration;
    ///
    /// let now: Time = Time::now().unwrap();
    /// // do something
    /// let elapsed_time: Duration = now.elapsed().unwrap();
    /// ```
    pub fn elapsed(&self) -> Result<Duration, TimeError> {
        let now =
            fail!(from self, when Time::now_with_clock(self.clock_type), "Failed to acquire elapsed time")
                .as_duration();

        Ok(now - self.as_duration())
    }

    /// Returns the number of seconds
    pub fn seconds(&self) -> u64 {
        self.seconds
    }

    /// Returns the fractional part in nanoseconds
    pub fn nanoseconds(&self) -> u32 {
        self.nanoseconds
    }

    pub fn clock_type(&self) -> ClockType {
        self.clock_type
    }

    /// Converts Time into duration
    pub fn as_duration(&self) -> Duration {
        Duration::from_secs(self.seconds) + Duration::from_nanos(self.nanoseconds as u64)
    }
}

impl AsTimespec for Time {
    fn as_timespec(&self) -> posix::timespec {
        posix::timespec {
            tv_sec: self.seconds as _,
            tv_nsec: self.nanoseconds as _,
        }
    }
}

/// Suspends the current thread for a provided duration.
///
/// # Examples
/// ```
/// use iceoryx2_bb_posix::clock::*;
/// use core::time::Duration;
///
/// // sleep for 100 milliseconds
/// nanosleep(Duration::from_millis(100)).unwrap();
/// ```
pub fn nanosleep(duration: Duration) -> Result<(), NanosleepError> {
    nanosleep_with_clock(duration, ClockType::default())
}

/// Suspends the current thread for a provided duration in a user provided [`ClockType`].
///
/// # Attention
///
/// Be aware
/// that the sleep can be extended or non existing when using [`ClockType::Realtime`] and the
/// system time changes.
/// Assume you would like to wait 1 second and while being in the function the system time is
///  * moved one hour into the future, then you will not wait at all.
///  * moved one hour into the past, then you will wait for 3601 seconds
///
/// # Examples
/// ```
/// use iceoryx2_bb_posix::clock::*;
/// use core::time::Duration;
///
/// // sleep for 100 milliseconds
/// nanosleep_with_clock(Duration::from_millis(100), ClockType::Realtime).unwrap();
/// ```
pub fn nanosleep_with_clock(
    duration: Duration,
    clock_type: ClockType,
) -> Result<(), NanosleepError> {
    if duration.is_zero() {
        return Ok(());
    }

    let wait_time = Time::now_with_clock(clock_type)?.as_duration() + duration;

    let timeout = posix::timespec {
        tv_sec: wait_time.as_secs() as posix::time_t,
        tv_nsec: wait_time.subsec_nanos() as posix::long,
    };

    let mut time_left = posix::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let mut remaining_sleeping_time = Duration::ZERO;
    handle_errno!(NanosleepError, from "nanosleep_with_clock",
        errno_source unsafe {
            let e = posix::clock_nanosleep(
                clock_type.as_i32() as _,
                posix::CLOCK_TIMER_ABSTIME,
                &timeout,
                &mut time_left,
            ).into();

            if e == Errno::EINTR {
                remaining_sleeping_time = Duration::from_secs(time_left.tv_sec as u64)
                    + Duration::from_nanos(time_left.tv_nsec as u64);
            }
            e
        },
        success Errno::ESUCCES => (),
        Errno::EINTR => (InterruptedBySignal(remaining_sleeping_time),
            "Interrupted \"nanosleep\": {{ duration: {:?}, clock_type: {:?} }}, remaining sleeping time: {:?}", duration, clock_type, remaining_sleeping_time),
        Errno::EINVAL => (DurationOutOfRange, "Invalid argument in \"nanosleep\". Either the duration: {:?} is out of range or the clock type {:?} is invalid.", duration, clock_type),
        Errno::ENOTSUP => (ClockTypeIsNotSupported, "Clock not supported in \"nanosleep\": {{ duration: {:?}, clock_type: {:?} }}", duration, clock_type),
        v => (UnknownError(v as i32), "Unknown error occurred in \"nanosleep\": {{ duration: {:?}, clock_type: {:?} }}, ({})", duration, clock_type, v)
    );
}
