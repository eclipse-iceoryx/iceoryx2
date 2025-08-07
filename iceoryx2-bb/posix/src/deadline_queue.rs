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
//! use iceoryx2_bb_posix::deadline_queue::*;
//! use core::time::Duration;
//!
//! let deadline_queue = DeadlineQueueBuilder::new().create().unwrap();
//!
//! // the DeadlineQueue waits on the following time points
//! // 4 5 8 9 10 12 15 16 18
//!
//! let guard_1 = deadline_queue.add_deadline_interval(Duration::from_secs(4));
//! let guard_2 = deadline_queue.add_deadline_interval(Duration::from_secs(5));
//! let guard_3 = deadline_queue.add_deadline_interval(Duration::from_secs(9));
//!
//! std::thread::sleep(deadline_queue.duration_until_next_deadline().unwrap());
//!
//! // contains all the deadlines where the deadline was hit
//! let mut missed_deadlines = vec![];
//! deadline_queue
//!     .missed_deadlines(|deadline_queue_index| {
//!         missed_deadlines.push(deadline_queue_index);
//!         CallbackProgression::Continue
//!     });
//! ```

pub use iceoryx2_bb_elementary::CallbackProgression;

use core::{cell::RefCell, fmt::Debug, sync::atomic::Ordering, time::Duration};
use iceoryx2_bb_log::fail;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

use crate::{
    clock::ClockType,
    clock::{Time, TimeError},
};

/// Represents an index to identify an added deadline_queue with [`DeadlineQueue::add_deadline_interval()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DeadlineQueueIndex(u64);

pub trait DeadlineQueueGuardable: Debug {}

/// Represents the RAII guard of [`DeadlineQueue`] and is returned by [`DeadlineQueue::add_deadline_interval()`].
/// As soon as it goes out of scope it removes the attached cyclic deadline from [`DeadlineQueue`].
#[derive(Debug)]
pub struct DeadlineQueueGuard<'deadline_queue> {
    deadline_queue: &'deadline_queue DeadlineQueue,
    index: DeadlineQueueIndex,
}

impl DeadlineQueueGuardable for DeadlineQueueGuard<'_> {}

impl DeadlineQueueGuard<'_> {
    /// Returns the underlying [`DeadlineQueueIndex`] of the attachment.
    pub fn index(&self) -> DeadlineQueueIndex {
        self.index
    }

    /// Resets the attached deadline_queue and wait again the full time.
    pub fn reset(&self) -> Result<(), TimeError> {
        self.deadline_queue.reset(self.index)
    }
}

impl Drop for DeadlineQueueGuard<'_> {
    fn drop(&mut self) {
        self.deadline_queue.remove(self.index.0);
    }
}

/// Builder to create a [`DeadlineQueue`].
pub struct DeadlineQueueBuilder {
    clock_type: ClockType,
}

impl Default for DeadlineQueueBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DeadlineQueueBuilder {
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

    /// Creates a new [`DeadlineQueue`]
    pub fn create(self) -> Result<DeadlineQueue, TimeError> {
        let start_time = fail!(from "DeadlineQueue::new()", when Time::now_with_clock(self.clock_type),
                                "Failed to create DeadlineQueue since the current time could not be acquired.");
        let start_time = start_time.as_duration().as_nanos();

        Ok(DeadlineQueue {
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
                                "Failed to create DeadlineQueue attachment since the current time could not be acquired.");
        let start_time = start_time.as_duration().as_nanos();

        Ok(Self {
            index,
            period,
            start_time,
        })
    }

    fn reset(&mut self, clock_type: ClockType) -> Result<(), TimeError> {
        let start_time = fail!(from "Attachment::new()", when Time::now_with_clock(clock_type),
                                "Failed to reset DeadlineQueue attachment since the current time could not be acquired.");
        self.start_time = start_time.as_duration().as_nanos();
        Ok(())
    }
}

/// The [`DeadlineQueue`] allows the user to attach multiple periodic deadline_queues with
/// [`DeadlineQueue::add_deadline_interval()`], to wait on them by acquiring the waiting time to the next deadline_queue
/// with [`DeadlineQueue::duration_until_next_deadline()`] and to acquire all missed deadline_queues via
/// [`DeadlineQueue::missed_deadlines()`].
#[derive(Debug)]
pub struct DeadlineQueue {
    attachments: RefCell<Vec<Attachment>>,
    id_count: IoxAtomicU64,
    previous_iteration: RefCell<u128>,

    clock_type: ClockType,
}

impl DeadlineQueue {
    /// Returns the number of attachments.
    pub fn len(&self) -> usize {
        self.attachments.borrow().len()
    }

    /// Returns true if the deadline queue does not contain any attachments.
    pub fn is_empty(&self) -> bool {
        self.attachments.borrow().is_empty()
    }

    /// Adds a cyclic deadline to the [`DeadlineQueue`] and returns an [`DeadlineQueueGuard`] to
    /// identify the attachment uniquely.
    /// [`DeadlineQueue::duration_until_next_deadline()`] will schedule the timings in a way that the
    /// attached deadline is considered cyclicly.
    pub fn add_deadline_interval(
        &self,
        deadline: Duration,
    ) -> Result<DeadlineQueueGuard<'_>, TimeError> {
        let current_idx = self.id_count.load(Ordering::Relaxed);
        self.attachments.borrow_mut().push(Attachment::new(
            current_idx,
            deadline.as_nanos(),
            self.clock_type,
        )?);
        self.id_count.fetch_add(1, Ordering::Relaxed);

        Ok(DeadlineQueueGuard {
            deadline_queue: self,
            index: DeadlineQueueIndex(current_idx),
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

    /// Resets the attached deadline_queue and wait again the full time.
    pub fn reset(&self, index: DeadlineQueueIndex) -> Result<(), TimeError> {
        for attachment in &mut *self.attachments.borrow_mut() {
            if attachment.index == index.0 {
                attachment.reset(self.clock_type)?;
                break;
            }
        }

        Ok(())
    }

    /// Returns the waiting duration until the next deadline is reached. If there have been
    /// already deadlines missed it returns a duration of zero.
    pub fn duration_until_next_deadline(&self) -> Result<Duration, TimeError> {
        if self.is_empty() {
            return Ok(Duration::MAX);
        }

        let now = fail!(from self, when Time::now_with_clock(self.clock_type),
                        "Unable to return next duration since the current time could not be acquired.");
        let now = now.as_duration().as_nanos();
        let mut has_missed_deadline = false;
        self.handle_missed_deadlines(now, |_| {
            has_missed_deadline = true;
            CallbackProgression::Stop
        });

        if has_missed_deadline {
            return Ok(Duration::ZERO);
        }

        // must be set after the return caused by a missed deadline, otherwise the user is unable
        // to acquire the missed deadline
        *self.previous_iteration.borrow_mut() = now;

        let mut min_time = u128::MAX;
        for attachment in &*self.attachments.borrow() {
            min_time =
                min_time.min(attachment.period - (now - attachment.start_time) % attachment.period);
        }

        Ok(Duration::from_nanos(min_time as _))
    }

    fn handle_missed_deadlines<F: FnMut(DeadlineQueueIndex) -> CallbackProgression>(
        &self,
        now: u128,
        mut call: F,
    ) {
        let last = *self.previous_iteration.borrow();

        for attachment in &*self.attachments.borrow() {
            let duration_until_last = last.max(attachment.start_time) - attachment.start_time;
            let duration_until_now = now - attachment.start_time;
            match attachment.period {
                0 => {
                    if matches!(
                        call(DeadlineQueueIndex(attachment.index)),
                        CallbackProgression::Stop
                    ) {
                        return;
                    }
                }
                _ => {
                    let last = duration_until_last / attachment.period;
                    let current = duration_until_now / attachment.period;

                    if last < current
                        && matches!(
                            call(DeadlineQueueIndex(attachment.index)),
                            CallbackProgression::Stop
                        )
                    {
                        return;
                    }
                }
            }
        }
    }

    /// Iterates over all missed deadlines and calls the provided callback for each of them
    /// and provide the [`DeadlineQueueIndex`] to identify them.
    pub fn missed_deadlines<F: FnMut(DeadlineQueueIndex) -> CallbackProgression>(
        &self,
        mut call: F,
    ) -> Result<(), TimeError> {
        let now = fail!(from self, when Time::now_with_clock(self.clock_type),
                        "Unable to return next duration since the current time could not be acquired.");

        let now = now.as_duration().as_nanos();
        self.handle_missed_deadlines(now, |idx| -> CallbackProgression { call(idx) });
        *self.previous_iteration.borrow_mut() = now;

        Ok(())
    }
}
