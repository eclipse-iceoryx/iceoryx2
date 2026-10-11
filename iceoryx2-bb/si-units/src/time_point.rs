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
};

use crate::{TimeSpan, internal};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use serde::{Deserialize, Serialize};

/// A time point, a specific timestamp in time (a duration since the epoch).
///
/// A [`TimePoint`] can be created from seconds, milliseconds, microseconds or
/// nanoseconds and converted back to any of these units. Arithmetic with
/// [`TimeSpan`]s is supported to shift a time point forward or backward in time.
///
/// # Example
///
/// ```rust
/// use iceoryx2_bb_si_units::{TimePoint, TimeSpan};
///
/// let mut time_point = TimePoint::from_secs(10);
/// time_point += TimeSpan::from_millis(500);
/// assert_eq!(time_point.as_secs(), 10);
/// assert_eq!(time_point.subsec_nanos(), 500_000_000);
///
/// let earlier = time_point - TimeSpan::from_secs(2);
/// assert_eq!(earlier.as_secs(), 8);
/// ```
#[repr(C)]
#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ZeroCopySend,
)]
pub struct TimePoint {
    #[serde(flatten)]
    time: internal::TimeSpec,
}

impl Add<TimeSpan> for TimePoint {
    type Output = TimePoint;

    fn add(self, rhs: TimeSpan) -> Self::Output {
        TimePoint {
            time: self.time + rhs.time,
        }
    }
}

impl AddAssign<TimeSpan> for TimePoint {
    fn add_assign(&mut self, rhs: TimeSpan) {
        self.time = self.time + rhs.time
    }
}

impl Sub<TimeSpan> for TimePoint {
    type Output = TimePoint;

    fn sub(self, rhs: TimeSpan) -> Self::Output {
        TimePoint {
            time: self.time - rhs.time,
        }
    }
}

impl Sub<TimePoint> for TimePoint {
    type Output = TimeSpan;

    fn sub(self, rhs: TimePoint) -> Self::Output {
        TimeSpan {
            time: self.time - rhs.time,
        }
    }
}

impl SubAssign<TimeSpan> for TimePoint {
    fn sub_assign(&mut self, rhs: TimeSpan) {
        self.time = self.time - rhs.time
    }
}

impl Display for TimePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.time)
    }
}

impl TimePoint {
    /// Creates a new [`TimePoint`] of `0` seconds.
    pub const fn new() -> Self {
        Self {
            time: internal::TimeSpec::new(),
        }
    }

    /// Returns the amount of time elapsed since the given [`TimePoint`].
    pub fn time_span_since(&self, value: TimePoint) -> TimeSpan {
        if self.time < value.time {
            value - *self
        } else {
            *self - value
        }
    }

    /// Creates a new [`TimePoint`] from the given number of seconds.
    pub const fn from_secs(value: u64) -> Self {
        Self {
            time: internal::TimeSpec::from_secs(value),
        }
    }

    /// Creates a new [`TimePoint`] from the given number of milliseconds.
    pub const fn from_millis(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_millis(value),
        }
    }

    /// Creates a new [`TimePoint`] from the given number of microseconds.
    pub const fn from_micros(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_micros(value),
        }
    }

    /// Creates a new [`TimePoint`] from the given number of nanoseconds.
    pub const fn from_nanos(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_nanos(value),
        }
    }

    /// Returns the [`TimePoint`] as a number of seconds.
    pub const fn as_secs(&self) -> u64 {
        self.time.as_secs()
    }

    /// Returns the [`TimePoint`] as a number of microseconds.
    pub const fn as_micros(&self) -> u64 {
        self.time.as_micros()
    }

    /// Returns the [`TimePoint`] as a number of milliseconds.
    pub const fn as_millis(&self) -> u64 {
        self.time.as_millis()
    }

    /// Returns the [`TimePoint`] as a number of nanoseconds.
    pub const fn as_nanos(&self) -> u128 {
        self.time.as_nanos()
    }

    /// Returns the sub-second component in nanoseconds
    pub const fn subsec_nanos(&self) -> u32 {
        self.time.subsec_nanos()
    }
}
