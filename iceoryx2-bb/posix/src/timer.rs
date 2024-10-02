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
//! use iceoryx2_bb_posix::timer::*;
//! use core::time::Duration;
//!
//! let timer = TimerBuilder::new().create().unwrap();
//!
//! // the timer waits on the following time points
//! // 4 5 8 9 10 12 15 16 18
//!
//! timer.cyclic(Duration::from_secs(4));
//! timer.cyclic(Duration::from_secs(5));
//! timer.cyclic(Duration::from_secs(9));
//!
//! std::thread::sleep(timer.duration_until_next_timeout().unwrap());
//!
//! // contains all the timers where the timeout was hit
//! let mut missed_timeouts = vec![];
//! timer
//!     .missed_timeouts(|timer_index| missed_timeouts.push(timer_index));
//! ```

use std::{cell::RefCell, sync::atomic::Ordering, time::Duration};

use iceoryx2_bb_log::fail;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

use crate::{
    clock::ClockType,
    clock::{Time, TimeError},
};

/// Represents an index to identify an added timer with [`Timer::cyclic()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerIndex(u64);

/// Represents the RAII guard of [`Timer`] and is returned by [`Timer::cyclic()`].
/// As soon as it goes out of scope it removes the attached cyclic timeout from [`Timer`].
pub struct TimerGuard<'timer> {
    timer: &'timer Timer,
    index: u64,
}

impl<'timer> TimerGuard<'timer> {
    /// Returns the underlying [`TimerIndex`] of the attachment.
    pub fn index(&self) -> TimerIndex {
        TimerIndex(self.index)
    }

    /// Resets the attached timer and wait again the full time.
    pub fn reset(&self) -> Result<(), TimeError> {
        self.timer.reset(self.index)
    }
}

impl<'timer> Drop for TimerGuard<'timer> {
    fn drop(&mut self) {
        self.timer.remove(self.index);
    }
}

/// Builder to create a [`Timer`].
pub struct TimerBuilder {
    clock_type: ClockType,
}

impl Default for TimerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerBuilder {
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

    /// Creates a new [`Timer`]
    pub fn create(self) -> Result<Timer, TimeError> {
        let start_time = fail!(from "Timer::new()", when Time::now_with_clock(self.clock_type),
                                "Failed to create Timer since the current time could not be acquired.");
        let start_time = start_time.as_duration().as_nanos();

        Ok(Timer {
            attachments: RefCell::new(vec![]),
            id_count: IoxAtomicU64::new(0),
            clock_type: self.clock_type,
            previous_iteration: RefCell::new(start_time),
        })
    }
}

#[derive(Debug)]
struct Attachment {
    index: u64,
    period: u128,
    start_time: u128,
}

impl Attachment {
    fn new(index: u64, period: u128, clock_type: ClockType) -> Result<Self, TimeError> {
        let start_time = fail!(from "Attachment::new()", when Time::now_with_clock(clock_type),
                                "Failed to create Timer attachment since the current time could not be acquired.");
        let start_time = start_time.as_duration().as_nanos();

        Ok(Self {
            index,
            period,
            start_time,
        })
    }

    fn reset(&mut self, clock_type: ClockType) -> Result<(), TimeError> {
        let start_time = fail!(from "Attachment::new()", when Time::now_with_clock(clock_type),
                                "Failed to reset Timer attachment since the current time could not be acquired.");
        self.start_time = start_time.as_duration().as_nanos();
        Ok(())
    }
}

/// The [`Timer`] allows the user to attach multiple periodic timers with
/// [`Timer::cyclic()`], to wait on them by acquiring the waiting time to the next timer
/// with [`Timer::duration_until_next_timeout()`] and to acquire all missed timers via
/// [`Timer::missed_timeouts()`].
#[derive(Debug)]
pub struct Timer {
    attachments: RefCell<Vec<Attachment>>,
    id_count: IoxAtomicU64,
    previous_iteration: RefCell<u128>,

    clock_type: ClockType,
}

impl Timer {
    /// Adds a cyclic timeout to the [`Timer`] and returns an [`TimerGuard`] to
    /// identify the attachment uniquely.
    /// [`Timer::duration_until_next_timeout()`] will schedule the timings in a way that the attached
    /// timeout is considered cyclicly.
    pub fn cyclic(&self, timeout: Duration) -> Result<TimerGuard, TimeError> {
        let current_idx = self.id_count.load(Ordering::Relaxed);
        self.attachments.borrow_mut().push(Attachment::new(
            current_idx,
            timeout.as_nanos(),
            self.clock_type,
        )?);
        self.id_count.fetch_add(1, Ordering::Relaxed);

        Ok(TimerGuard {
            timer: self,
            index: current_idx,
        })
    }

    fn remove(&self, index: u64) {
        let mut index_to_remove = None;
        for (n, attachment) in self.attachments.borrow().iter().enumerate() {
            if attachment.index == index {
                index_to_remove = Some(n);
                break;
            }
        }

        if let Some(n) = index_to_remove {
            self.attachments.borrow_mut().remove(n);
        }
    }

    fn reset(&self, index: u64) -> Result<(), TimeError> {
        for attachment in &mut *self.attachments.borrow_mut() {
            if attachment.index == index {
                attachment.reset(self.clock_type)?;
                break;
            }
        }

        Ok(())
    }

    /// Returns the waiting duration until the next added timeout is reached.
    pub fn duration_until_next_timeout(&self) -> Result<Duration, TimeError> {
        let now = fail!(from self, when Time::now_with_clock(self.clock_type),
                        "Unable to return next duration since the current time could not be acquired.");
        let now = now.as_duration().as_nanos();
        *self.previous_iteration.borrow_mut() = now;

        let mut min_time = u128::MAX;
        for attachment in &*self.attachments.borrow() {
            min_time =
                min_time.min(attachment.period - (now - attachment.start_time) % attachment.period);
        }

        Ok(Duration::from_nanos(min_time as _))
    }

    /// Iterates over all missed timeouts and calls the provided callback for each of them
    /// and provide the [`TimerIndex`] to identify them.
    pub fn missed_timeouts<F: FnMut(TimerIndex)>(&self, mut call: F) -> Result<(), TimeError> {
        let now = fail!(from self, when Time::now_with_clock(self.clock_type),
                        "Unable to return next duration since the current time could not be acquired.");

        let now = now.as_duration().as_nanos();
        let last = *self.previous_iteration.borrow();

        for attachment in &*self.attachments.borrow() {
            let duration_until_last = last.max(attachment.start_time) - attachment.start_time;
            let duration_until_now = now - attachment.start_time;
            if (duration_until_last / attachment.period) < (duration_until_now / attachment.period)
            {
                call(TimerIndex(attachment.index));
            }
        }

        Ok(())
    }
}
