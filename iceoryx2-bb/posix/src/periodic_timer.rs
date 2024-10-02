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

//! # Example
//!
//! ```no_run
//! use iceoryx2_bb_periodic_timer::*;
//!
//! let periodic_timer = PeriodicTimerBuilder::new().create().unwrap();
//!
//! // the timer waits on the following time points
//! // 4 5 8 9 10 12 15 16 18
//!
//! periodic_timer.add(Duration::from_secs(4));
//! periodic_timer.add(Duration::from_secs(5));
//! periodic_timer.add(Duration::from_secs(9));
//!
//! std::thread::sleep(periodic_timer.next_iteration().unwrap());
//!
//! ```

use std::time::Duration;

use iceoryx2_bb_log::fail;

use crate::{
    clock::ClockType,
    clock::{Time, TimeError},
};

/// Represents an index to identify an added timer with [`PeriodicTimer::add()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodicTimerIndex(usize);

/// Builder to create a [`PeriodicTimer`].
pub struct PeriodicTimerBuilder {
    clock_type: ClockType,
}

impl PeriodicTimerBuilder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            clock_type: ClockType::default(),
        }
    }

    /// Defines the [`ClockType`] that is used for time measurements. By default it is
    /// [`ClockType::default()`].
    pub fn clock_type(mut self, value: ClockType) -> Self {
        self.clock_type = value;
        self
    }

    /// Creates a new [`PeriodicTimer`]
    pub fn create(self) -> Result<PeriodicTimer, TimeError> {
        let start_time = fail!(from "PeriodicTimer::new()", when Time::now_with_clock(self.clock_type),
                                "Failed to create PeriodicTimer since the current time could not be acquired.");

        Ok(PeriodicTimer {
            timeouts: vec![],
            id_count: PeriodicTimerIndex(0),
            clock_type: self.clock_type,
            last_iteration: 0,
            start_time,
        })
    }
}

#[derive(Debug)]
struct Attachment {
    index: PeriodicTimerIndex,
    period: u128,
    start_time: u128,
}

impl Attachment {
    fn new(
        index: PeriodicTimerIndex,
        period: u128,
        clock_type: ClockType,
    ) -> Result<Self, TimeError> {
        let start_time = fail!(from "Attachment::new()", when Time::now_with_clock(clock_type),
                                "Failed to create PeriodicTimer attachment since the current time could not be acquired.");
        let start_time = start_time.as_duration().as_nanos();

        Ok(Self {
            index,
            period,
            start_time,
        })
    }

    fn reset(&mut self, clock_type: ClockType) -> Result<(), TimeError> {
        let start_time = fail!(from "Attachment::new()", when Time::now_with_clock(clock_type),
                                "Failed to reset PeriodicTimer attachment since the current time could not be acquired.");
        self.start_time = start_time.as_duration().as_nanos();
        Ok(())
    }
}

/// The [`PeriodicTimer`] allows the user to attach multiple periodic timers with
/// [`PeriodicTimer::add()`], to wait on them by acquiring the waiting time to the next timer
/// with [`PeriodicTimer::next_iteration()`] and to acquire all missed timers via
/// [`PeriodicTimer::missed_timers()`].
#[derive(Debug)]
pub struct PeriodicTimer {
    timeouts: Vec<Attachment>,
    id_count: PeriodicTimerIndex,
    clock_type: ClockType,
    start_time: Time,
    last_iteration: u128,
}

impl PeriodicTimer {
    /// Adds a new timeout to the [`PeriodicTimer`] and returns an [`PeriodicTimerIndex`] to
    /// identify the attachment uniquely.
    pub fn add(&mut self, timeout: Duration) -> Result<PeriodicTimerIndex, TimeError> {
        let current_idx = self.id_count;
        self.timeouts.push(Attachment::new(
            current_idx,
            timeout.as_nanos(),
            self.clock_type,
        )?);
        self.id_count.0 += 1;

        Ok(current_idx)
    }

    /// Removes the given [`PeriodicTimerIndex`] from the [`PeriodicTimer`].
    pub fn remove(&mut self, index: PeriodicTimerIndex) {
        for (n, attachment) in self.timeouts.iter().enumerate() {
            if attachment.index == index {
                self.timeouts.remove(n);
                break;
            }
        }
    }

    /// Resets the attached timer and wait again the full time.
    pub fn reset(&mut self, index: PeriodicTimerIndex) -> Result<(), TimeError> {
        for attachment in &mut self.timeouts {
            if attachment.index == index {
                attachment.reset(self.clock_type)?;
                break;
            }
        }

        Ok(())
    }

    /// Returns the waiting duration until the next added timeout is reached.
    pub fn next_iteration(&mut self) -> Result<Duration, TimeError> {
        let now = fail!(from self, when Time::now_with_clock(self.clock_type),
                        "Unable to return next duration since the current time could not be acquired.");
        let now = now.as_duration().as_nanos();
        self.last_iteration = now;

        let mut min_time = u128::MAX;
        for attachment in &self.timeouts {
            min_time =
                min_time.min(attachment.period - (now - attachment.start_time) % attachment.period);
        }

        Ok(Duration::from_nanos(min_time as _))
    }

    /// Iterates over all missed timeouts and calls the provided callback for each of them
    /// and provide the [`PeriodicTimerIndex`] to identify them.
    pub fn missed_timers<F: FnMut(PeriodicTimerIndex)>(
        &self,
        mut call: F,
    ) -> Result<(), TimeError> {
        let elapsed = fail!(from self, when self.start_time.elapsed(),
                        "Unable to return next duration since the elapsed time could not be acquired.");

        let last = self.last_iteration;
        let elapsed = elapsed.as_nanos();

        for attachment in &self.timeouts {
            if ((last - attachment.start_time) / attachment.period)
                < ((elapsed - attachment.start_time) / attachment.period)
            {
                call(attachment.index);
            }
        }

        Ok(())
    }
}
