// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

//! A **threadsafe** queue which triggers a consumer when data arrived or triggers the producer
//! when the queue is no longer full.
//!
//! # Example
//!
//! ```
//! use std::thread;
//! use iceoryx2_bb_threadsafe::trigger_queue::*;
//!
//! const CAPACITY: usize = 16;
//!
//! let mtx_handle = MutexHandle::new();
//! let free_handle = UnnamedSemaphoreHandle::new();
//! let used_handle = UnnamedSemaphoreHandle::new();
//!
//! let queue = TriggerQueue::<u64, CAPACITY>::new(&mtx_handle, &free_handle, &used_handle);
//!
//! thread::scope(|s| {
//!     let consumer = s.spawn(|| {
//!         for i in 0..10 {
//!             println!("got: {}", queue.blocking_pop());
//!         }
//!     });
//!
//!     let producer = s.spawn(|| {
//!         for i in 0..10 {
//!             queue.blocking_push(i);
//!             println!("pushed data {}", i);
//!         }
//!     });
//! })
//! ```
pub use iceoryx2_bb_posix::mutex::*;
pub use iceoryx2_bb_posix::semaphore::*;

use core::{fmt::Debug, marker::PhantomData, time::Duration};
use iceoryx2_bb_container::queue::FixedSizeQueue;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_log::fatal_panic;

const INTER_PROCESS_SUPPORT: bool = true;

#[derive(Debug)]
pub struct TriggerQueue<'a, T: Debug, const CAPACITY: usize> {
    queue: Mutex<'a, 'a, FixedSizeQueue<T, CAPACITY>>,
    free_slots: UnnamedSemaphore<'a>,
    used_slots: UnnamedSemaphore<'a>,
    _phantom_data: PhantomData<T>,
}

unsafe impl<T: Debug + ZeroCopySend, const CAPACITY: usize> ZeroCopySend
    for TriggerQueue<'_, T, CAPACITY>
{
}

impl<'a, T: Debug, const CAPACITY: usize> TriggerQueue<'a, T, CAPACITY> {
    /// Creates a new [`TriggerQueue`] which uses the [`ClockType::default()`] in
    /// [`TriggerQueue::timed_push()`] and [`TriggerQueue::timed_pop()`].
    pub fn new(
        mtx_handle: &'a MutexHandle<FixedSizeQueue<T, CAPACITY>>,
        free_handle: &'a UnnamedSemaphoreHandle,
        used_handle: &'a UnnamedSemaphoreHandle,
    ) -> Self {
        Self::new_with_custom_clock(mtx_handle, free_handle, used_handle, ClockType::default())
    }

    /// Creates a new [`TriggerQueue`] which uses the user provided clock in
    /// [`TriggerQueue::timed_push()`] and [`TriggerQueue::timed_pop()`].
    pub fn new_with_custom_clock(
        mtx_handle: &'a MutexHandle<FixedSizeQueue<T, CAPACITY>>,
        free_handle: &'a UnnamedSemaphoreHandle,
        used_handle: &'a UnnamedSemaphoreHandle,
        clock_type: ClockType,
    ) -> Self {
        let msg = "Fatal failure while creating TriggerQueue";
        Self {
            queue: fatal_panic!(from "TriggerQueue::new", when MutexBuilder::new()
                    .is_interprocess_capable(INTER_PROCESS_SUPPORT)
                    .create(FixedSizeQueue::<T, CAPACITY>::new(), mtx_handle),
                    "{} since the mutex creation failed.", msg),
            free_slots: fatal_panic!(from "TriggerQueue::new", when UnnamedSemaphoreBuilder::new()
                    .initial_value(CAPACITY as u32)
                    .clock_type(clock_type).create(free_handle),
                    "{} since the free slots counting semaphore creation failed.", msg),
            used_slots: fatal_panic!(from "TriggerQueue::new", when UnnamedSemaphoreBuilder::new()
                    .initial_value(0)
                    .clock_type(clock_type).create(used_handle),
                    "{} since the free slots counting semaphore creation failed.", msg),
            _phantom_data: PhantomData,
        }
    }

    /// Tries to push a value into the queue. When the queue is full it returns false, otherwise
    /// true.
    pub fn try_push(&self, value: T) -> bool {
        match self.free_slots.try_wait().unwrap() {
            true => self.push(value),
            false => false,
        }
    }

    /// Tries to push a value into the queue until the timeout is reached. If the sample was
    /// pushed into the queue it returns true, otherwise false.
    pub fn timed_push(&self, value: T, timeout: Duration) -> bool {
        match self.free_slots.timed_wait(timeout).unwrap() {
            true => self.push(value),
            false => false,
        }
    }

    /// Blocks the process until the value could be pushed into the queue.
    pub fn blocking_push(&self, value: T) {
        self.free_slots.blocking_wait().unwrap();
        self.push(value);
    }

    /// Tries to pop a value out of the queue. If the queue was empty it returns [`None`]
    /// otherwise the value packed inside the Option.
    pub fn try_pop(&self) -> Option<T> {
        match self.used_slots.try_wait().unwrap() {
            true => self.pop(),
            false => None,
        }
    }

    /// Tries to pop a value out of the queue until the timeout is reached. If a value could not be
    /// acquired it returns [`None`].
    pub fn timed_pop(&self, timeout: Duration) -> Option<T> {
        match self.used_slots.timed_wait(timeout).unwrap() {
            true => self.pop(),
            false => None,
        }
    }

    /// Blocks until a value could be acquired from the queue.
    pub fn blocking_pop(&self) -> T {
        self.used_slots.blocking_wait().unwrap();
        self.pop().unwrap()
    }

    /// Empties the queue.
    pub fn clear(&self) {
        while self.try_pop().is_some() {}
    }

    /// Returns the capacity of the queue.
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns the amount of values stored inside the queue
    pub fn len(&self) -> usize {
        fatal_panic!(from self, when self.queue.lock(),
            "Failed to acquire mutex to acquire size")
        .len()
    }

    /// Returns true if the queue is full, otherwise false
    pub fn is_full(&self) -> bool {
        fatal_panic!(from self, when self.queue.lock(),
            "Failed to acquire mutex to acquire full state")
        .is_full()
    }

    /// Returns true if the queue is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        fatal_panic!(from self, when self.queue.lock(),
            "Failed to acquire mutex to acquire empty state")
        .is_empty()
    }

    fn push(&self, value: T) -> bool {
        fatal_panic!(from self, when self.queue.lock(),
            "Failed to acquire mutex to push")
        .push(value);
        self.used_slots.post().unwrap();
        true
    }

    fn pop(&self) -> Option<T> {
        let value = fatal_panic!(from self, when self.queue.lock(),
            "Failed to acquire mutex to pop")
        .pop();
        self.free_slots.post().unwrap();
        value
    }
}
