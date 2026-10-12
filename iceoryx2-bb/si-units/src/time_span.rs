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

use core::{
    fmt::Display,
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration,
};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use serde::{Deserialize, Serialize};

use crate::internal;

/// A timespan, a duration of time (a difference between two [`TimePoint`](crate::TimePoint)s).
///
/// It is an intermediate [`Duration`] representation with a fixed memory layout
/// which is shared memory compatible.
///
/// A [`TimeSpan`] can be created from seconds, milliseconds, microseconds or
/// nanoseconds and converted back to any of these units. Time spans can be
/// added to or subtracted from each other, as well as from [`TimePoint`](crate::TimePoint)s, and
/// converted to/from [`core::time::Duration`].
///
/// # Example
///
/// ```rust
/// use iceoryx2_bb_si_units::{TimePoint, TimeSpan};
///
/// let mut span = TimeSpan::from_millis(1500);
/// span += TimeSpan::from_secs(2);
/// assert_eq!(span.as_secs(), 3);
/// assert_eq!(span.subsec_nanos(), 500_000_000);
///
/// let time_point = TimePoint::from_secs(10);
/// let later = time_point + span;
/// assert_eq!(later.as_secs(), 13);
///
/// let std_duration: std::time::Duration = span.into();
/// assert_eq!(std_duration.as_secs(), 3);
/// ```
#[repr(C)]
#[derive(
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Serialize,
    Deserialize,
    ZeroCopySend,
)]
pub struct TimeSpan {
    pub(crate) time: internal::TimeSpec,
}

impl Add<TimeSpan> for TimeSpan {
    type Output = TimeSpan;

    fn add(self, rhs: TimeSpan) -> Self::Output {
        Self {
            time: self.time + rhs.time,
        }
    }
}

impl Add<Duration> for TimeSpan {
    type Output = TimeSpan;

    fn add(self, rhs: Duration) -> Self::Output {
        let rhs: TimeSpan = rhs.into();
        self + rhs
    }
}

impl AddAssign<TimeSpan> for TimeSpan {
    fn add_assign(&mut self, rhs: TimeSpan) {
        self.time = self.time + rhs.time
    }
}

impl AddAssign<Duration> for TimeSpan {
    fn add_assign(&mut self, rhs: Duration) {
        let rhs: TimeSpan = rhs.into();
        *self += rhs;
    }
}

impl Sub<TimeSpan> for TimeSpan {
    type Output = TimeSpan;

    fn sub(self, rhs: TimeSpan) -> Self::Output {
        Self {
            time: self.time - rhs.time,
        }
    }
}

impl Sub<Duration> for TimeSpan {
    type Output = TimeSpan;

    fn sub(self, rhs: Duration) -> Self::Output {
        let rhs: TimeSpan = rhs.into();
        self - rhs
    }
}

impl SubAssign<TimeSpan> for TimeSpan {
    fn sub_assign(&mut self, rhs: TimeSpan) {
        self.time = self.time - rhs.time
    }
}

impl SubAssign<Duration> for TimeSpan {
    fn sub_assign(&mut self, rhs: Duration) {
        let rhs: TimeSpan = rhs.into();
        *self -= rhs;
    }
}

impl Display for TimeSpan {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.time)
    }
}

impl From<Duration> for TimeSpan {
    fn from(value: Duration) -> Self {
        Self { time: value.into() }
    }
}

impl From<TimeSpan> for Duration {
    fn from(value: TimeSpan) -> Self {
        value.time.into()
    }
}

impl TimeSpan {
    /// A [`TimeSpan`] with the size of 0.
    pub const ZERO: TimeSpan = TimeSpan {
        time: internal::TimeSpec::ZERO,
    };

    /// The maximum [`TimeSpan`].
    pub const MAX: TimeSpan = TimeSpan {
        time: internal::TimeSpec::MAX,
    };

    /// Creates a new [`TimeSpan`] of `0` seconds.
    pub const fn new() -> Self {
        Self {
            time: internal::TimeSpec::new(),
        }
    }

    /// Creates a new [`TimeSpan`] from the given number of seconds.
    pub const fn from_secs(value: u64) -> Self {
        Self {
            time: internal::TimeSpec::from_secs(value),
        }
    }

    /// Creates a new [`TimeSpan`] from the given number of milliseconds.
    pub const fn from_millis(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_millis(value),
        }
    }

    /// Creates a new [`TimeSpan`] from the given number of microseconds.
    pub const fn from_micros(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_micros(value),
        }
    }

    /// Creates a new [`TimeSpan`] from the given number of nanoseconds.
    pub const fn from_nanos(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_nanos(value),
        }
    }

    /// Creates a new [`TimeSpan`] from the given number of seconds.
    pub fn from_secs_f64(value: f64) -> Self {
        Self {
            time: internal::TimeSpec::from_secs_f64(value),
        }
    }

    /// Creates a new [`TimeSpan`] from the given number of seconds.
    pub fn from_secs_f32(value: f32) -> Self {
        Self {
            time: internal::TimeSpec::from_secs_f32(value),
        }
    }

    /// Returns the [`TimeSpan`] as a number of seconds.
    pub const fn as_secs(&self) -> u64 {
        self.time.as_secs()
    }

    /// Returns the [`TimeSpan`] as a number of seconds.
    pub const fn as_secs_f32(&self) -> f32 {
        self.time.as_secs_f32()
    }

    /// Returns the [`TimeSpan`] as a number of seconds.
    pub const fn as_secs_f64(&self) -> f64 {
        self.time.as_secs_f64()
    }

    /// Returns the [`TimeSpan`] as a number of microseconds.
    pub const fn as_micros(&self) -> u64 {
        self.time.as_micros()
    }

    /// Returns the [`TimeSpan`] as a number of milliseconds.
    pub const fn as_millis(&self) -> u64 {
        self.time.as_millis()
    }

    /// Returns the [`TimeSpan`] as a number of nanoseconds.
    pub const fn as_nanos(&self) -> u128 {
        self.time.as_nanos()
    }

    /// Returns the sub-second component in nanoseconds
    pub const fn subsec_nanos(&self) -> u32 {
        self.time.subsec_nanos()
    }
}
