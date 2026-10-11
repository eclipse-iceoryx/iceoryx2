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

const MILLI_SECOND_IN_SECONDS: u64 = 1_000;
const MICRO_SECOND_IN_SECONDS: u64 = 1_000_000;
const NANO_SECOND_IN_SECONDS: u64 = 1_000_000_000;

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
pub struct TimeSpec {
    pub seconds: u64,
    pub nanoseconds: u32,
}

impl From<Duration> for TimeSpec {
    fn from(value: Duration) -> Self {
        TimeSpec {
            seconds: value.as_secs(),
            nanoseconds: value.subsec_nanos(),
        }
    }
}

impl From<TimeSpec> for Duration {
    fn from(value: TimeSpec) -> Self {
        Duration::from_secs(value.seconds) + Duration::from_nanos(value.nanoseconds as u64)
    }
}

impl Add<TimeSpec> for TimeSpec {
    type Output = TimeSpec;

    fn add(self, rhs: TimeSpec) -> Self::Output {
        TimeSpec::from_nanos(self.as_nanos() + rhs.as_nanos())
    }
}

impl AddAssign<TimeSpec> for TimeSpec {
    fn add_assign(&mut self, rhs: TimeSpec) {
        *self = *self + rhs;
    }
}

impl Sub<TimeSpec> for TimeSpec {
    type Output = TimeSpec;

    fn sub(self, rhs: TimeSpec) -> Self::Output {
        let self_nanos = self.as_nanos();
        let rhs_nanos = rhs.as_nanos();

        if self_nanos < rhs_nanos {
            Self::ZERO
        } else {
            TimeSpec::from_nanos(self_nanos - rhs_nanos)
        }
    }
}

impl SubAssign<TimeSpec> for TimeSpec {
    fn sub_assign(&mut self, rhs: TimeSpec) {
        *self = *self - rhs;
    }
}

impl Display for TimeSpec {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.nanoseconds < 1_000 {
            write!(f, "{} ns", self.as_nanos())
        } else if self.nanoseconds < 1_000_000 {
            let frac = self.as_nanos() % 1_000;
            if frac != 0 {
                write!(f, "{}.{} us", self.as_micros(), frac)
            } else {
                write!(f, "{} us", self.as_micros(),)
            }
        } else if self.nanoseconds < 1_000_000_000 {
            let frac = self.as_micros() % 1_000;
            if frac != 0 {
                write!(f, "{}.{} ms", self.as_millis(), frac)
            } else {
                write!(f, "{} us", self.as_millis(),)
            }
        } else {
            let frac = self.as_millis() % 1_000;
            if frac != 0 {
                write!(f, "{}.{} s", self.as_secs(), frac)
            } else {
                write!(f, "{} s", self.as_secs(),)
            }
        }
    }
}

impl TimeSpec {
    pub const ZERO: TimeSpec = TimeSpec {
        seconds: 0,
        nanoseconds: 0,
    };

    pub const fn new() -> Self {
        Self::ZERO
    }

    pub const fn from_secs(value: u64) -> Self {
        Self {
            seconds: value,
            nanoseconds: 0,
        }
    }

    pub const fn from_millis(value: u128) -> Self {
        let seconds = value as u64 / MILLI_SECOND_IN_SECONDS;
        Self {
            seconds: value as u64 / MILLI_SECOND_IN_SECONDS,
            nanoseconds: (value * 1_000_000 - seconds as u128 * NANO_SECOND_IN_SECONDS as u128)
                as u32,
        }
    }

    pub const fn from_micros(value: u128) -> Self {
        let seconds = value as u64 / MICRO_SECOND_IN_SECONDS;
        Self {
            seconds: value as u64 / MICRO_SECOND_IN_SECONDS,
            nanoseconds: (value * 1_000 - seconds as u128 * NANO_SECOND_IN_SECONDS as u128) as u32,
        }
    }

    pub const fn from_nanos(value: u128) -> Self {
        let seconds = value as u64 / NANO_SECOND_IN_SECONDS;
        Self {
            seconds: value as u64 / NANO_SECOND_IN_SECONDS,
            nanoseconds: (value - seconds as u128 * NANO_SECOND_IN_SECONDS as u128) as u32,
        }
    }

    pub const fn as_secs(&self) -> u64 {
        self.seconds
    }

    pub const fn as_micros(&self) -> u64 {
        self.nanoseconds as u64 / MICRO_SECOND_IN_SECONDS + self.seconds * MICRO_SECOND_IN_SECONDS
    }

    pub const fn as_millis(&self) -> u64 {
        self.nanoseconds as u64 / MILLI_SECOND_IN_SECONDS + self.seconds * MILLI_SECOND_IN_SECONDS
    }

    pub const fn as_nanos(&self) -> u128 {
        self.nanoseconds as u128 + self.seconds as u128 * NANO_SECOND_IN_SECONDS as u128
    }

    pub const fn subsec_nanos(&self) -> u32 {
        self.nanoseconds
    }
}
