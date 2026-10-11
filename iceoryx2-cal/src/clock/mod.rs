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

use core::fmt::Display;

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_posix::clock::TimeSpan;
use serde::{Deserialize, Serialize};

enum_gen! {
    TimePointFromDurationError
  entry:
    Overflow
}

const MILLI_SECOND_IN_SECONDS: u64 = 1_000;
const MICRO_SECOND_IN_SECONDS: u64 = 1_000_000;
const NANO_SECOND_IN_SECONDS: u64 = 1_000_000_000;

#[repr(C)]
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TimePoint {
    seconds: u64,
    nanoseconds: u32,
}

unsafe impl ZeroCopySend for TimePoint {}

impl Display for TimePoint {
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

impl TimePoint {
    pub const fn new() -> Self {
        Self {
            seconds: 0,
            nanoseconds: 0,
        }
    }

    pub fn from_time_span(value: TimeSpan) -> Self {
        Self {
            seconds: value.as_secs(),
            nanoseconds: value.subsec_nanos(),
        }
    }

    pub fn from_duration(value: core::time::Duration) -> Self {
        Self {
            seconds: value.as_secs(),
            nanoseconds: value.subsec_nanos(),
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
}
