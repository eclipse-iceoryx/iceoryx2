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

use core::{fmt::Display, time::Duration};

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use serde::{Deserialize, Serialize};

const MILLI_SECOND_IN_SECONDS: u64 = 1_000;
const MICRO_SECOND_IN_SECONDS: u64 = 1_000_000;
const NANO_SECOND_IN_SECONDS: u64 = 1_000_000_000;

// TODO:
//  * add add, sub for timespec
//    * also for TimeSpan and TimePoint
//  * add docs and tests for TimePoint and TimeSpan
//  * add Display and all the other nice things to TimeSpan

#[repr(C)]
#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub(crate) struct TimeSpec {
    pub(crate) seconds: u64,
    pub(crate) nanoseconds: u32,
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
    pub(crate) const fn new() -> Self {
        Self {
            seconds: 0,
            nanoseconds: 0,
        }
    }

    pub(crate) const fn as_secs(&self) -> u64 {
        self.seconds
    }

    pub(crate) const fn as_micros(&self) -> u64 {
        self.nanoseconds as u64 / MICRO_SECOND_IN_SECONDS + self.seconds * MICRO_SECOND_IN_SECONDS
    }

    pub(crate) const fn as_millis(&self) -> u64 {
        self.nanoseconds as u64 / MILLI_SECOND_IN_SECONDS + self.seconds * MILLI_SECOND_IN_SECONDS
    }

    pub(crate) const fn as_nanos(&self) -> u128 {
        self.nanoseconds as u128 + self.seconds as u128 * NANO_SECOND_IN_SECONDS as u128
    }

    pub(crate) const fn subsec_nanos(&self) -> u32 {
        self.nanoseconds
    }
}

#[repr(C)]
#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct TimePoint {
    #[serde(flatten)]
    time: TimeSpec,
}

unsafe impl ZeroCopySend for TimePoint {}

impl Display for TimePoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.time)
    }
}

impl TimePoint {
    pub const fn new() -> Self {
        Self {
            time: TimeSpec::new(),
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
