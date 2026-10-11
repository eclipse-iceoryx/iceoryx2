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

// TODO:
//  * add add, sub for timespec
//    * also for TimeSpan and TimePoint
//  * add docs and tests for TimePoint and TimeSpan
//  * add Display and all the other nice things to TimeSpan

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
    pub const fn new() -> Self {
        Self {
            time: internal::TimeSpec::new(),
        }
    }

    pub const fn from_secs(value: u64) -> Self {
        Self {
            time: internal::TimeSpec::from_secs(value),
        }
    }

    pub const fn from_millis(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_millis(value),
        }
    }

    pub const fn from_micros(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_micros(value),
        }
    }

    pub const fn from_nanos(value: u128) -> Self {
        Self {
            time: internal::TimeSpec::from_nanos(value),
        }
    }

    pub const fn as_secs(&self) -> u64 {
        self.time.as_secs()
    }

    pub const fn as_micros(&self) -> u64 {
        self.time.as_micros()
    }

    pub const fn as_millis(&self) -> u64 {
        self.time.as_millis()
    }

    pub const fn as_nanos(&self) -> u128 {
        self.time.as_nanos()
    }

    pub const fn subsec_nanos(&self) -> u32 {
        self.time.subsec_nanos()
    }
}
