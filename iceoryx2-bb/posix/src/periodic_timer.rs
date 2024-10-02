// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use std::time::Duration;

use iceoryx2_bb_log::fail;

use crate::{
    clock::ClockType,
    clock::{Time, TimeError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodicTimerIndex(usize);

pub struct PeriodicTimerBuilder {
    clock_type: ClockType,
}

impl PeriodicTimerBuilder {
    pub fn new() -> Self {
        Self {
            clock_type: ClockType::default(),
        }
    }

    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self
    }

    pub fn create(self) -> Result<PeriodicTimer, TimeError> {
        let start_time = fail!(from "PeriodicTimer::new()", when Time::now_with_clock(self.clock_type),
                                "Failed to create PeriodicTimer since the current time could not be acquired.");

        Ok(PeriodicTimer {
            timeouts: vec![],
            id_count: PeriodicTimerIndex(0),
            clock_type: self.clock_type,
            last_iteration: Duration::ZERO,
            start_time,
        })
    }
}

#[derive(Debug)]
pub struct PeriodicTimer {
    timeouts: Vec<(PeriodicTimerIndex, u64)>,
    id_count: PeriodicTimerIndex,
    clock_type: ClockType,
    start_time: Time,
    last_iteration: Duration,
}

impl PeriodicTimer {
    pub fn add(&mut self, timeout: Duration) -> PeriodicTimerIndex {
        let current_idx = self.id_count;
        self.timeouts.push((current_idx, timeout.as_nanos() as _));
        self.id_count.0 += 1;
        current_idx
    }

    pub fn remove(&mut self, index: PeriodicTimerIndex) {
        for (n, (idx, _)) in self.timeouts.iter().enumerate() {
            if *idx == index {
                self.timeouts.remove(n);
                break;
            }
        }
    }

    pub fn start(&mut self) -> Result<(), TimeError> {
        self.start_time = fail!(from self, when Time::now_with_clock(self.clock_type),
                                "Failed to start PeriodicTimer since the current time could not be acquired.");

        Ok(())
    }

    pub fn next_iteration(&mut self) -> Result<Duration, TimeError> {
        let elapsed = fail!(from self, when self.start_time.elapsed(),
                        "Unable to return next duration since the elapsed time could not be acquired.");
        self.last_iteration = elapsed;
        let elapsed = elapsed.as_nanos();

        let mut min_time = u128::MAX;
        for (_, timeout) in &self.timeouts {
            min_time = min_time.min(*timeout as u128 - elapsed % *timeout as u128);
        }

        Ok(Duration::from_nanos(min_time as _))
    }

    pub fn missed_timers<F: FnMut(PeriodicTimerIndex)>(
        &self,
        mut call: F,
    ) -> Result<(), TimeError> {
        let elapsed = fail!(from self, when self.start_time.elapsed(),
                        "Unable to return next duration since the elapsed time could not be acquired.");

        let last = self.last_iteration.as_nanos();
        let elapsed = elapsed.as_nanos();

        for (index, timeout) in &self.timeouts {
            if (last / *timeout as u128) < (elapsed / *timeout as u128) {
                call(*index);
            }
        }

        Ok(())
    }
}
