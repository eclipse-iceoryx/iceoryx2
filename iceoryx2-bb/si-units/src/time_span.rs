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

use core::time::Duration;

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use serde::{Deserialize, Serialize};

use crate::time_point::TimeSpec;

/// A intermediate [`Duration`] representation with a fixed memory layout which
/// is shared memory compatible.
#[repr(C)]
#[derive(
    Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize,
)]
pub struct TimeSpan {
    #[serde(flatten)]
    time: TimeSpec,
}

unsafe impl ZeroCopySend for TimeSpan {}

impl TimeSpan {
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
