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

//! A **threadsafe** **lock-free** single produce single consumer queue.
//! **IMPORTANT** Can only be used with trivially copyable types which are also trivially dropable.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_lock_free::spsc::queue::*;
//!
//! const QUEUE_CAPACITY: usize = 128;
//! let queue = Queue::<u64, QUEUE_CAPACITY>::new();
//!
//! let mut producer = match queue.acquire_producer() {
//!     None => panic!("a producer has been already acquired."),
//!     Some(p) => p,
//! };
//!
//! if !producer.push(&1234) {
//!     println!("queue is full");
//! }
//!
//!
//! let mut consumer = match queue.acquire_consumer() {
//!     None => panic!("a consumer has been already acquired."),
//!     Some(p) => p,
//! };
//!
//! match consumer.pop() {
//!     None => println!("queue is empty"),
//!     Some(v) => println!("got {}", v)
//! }
//! ```

use core::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::Ordering};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU64};

/// The [`Producer`] of the [`Queue`] which can add values to it via [`Producer::push()`].
pub struct Producer<'a, T: Copy, const CAPACITY: usize> {
    queue: &'a Queue<T, CAPACITY>,
}

impl<T: Copy, const CAPACITY: usize> Producer<'_, T, CAPACITY> {
    /// Adds a new value to the queue, if the queue is full it returns false otherwise true
    pub fn push(&mut self, t: &T) -> bool {
        unsafe { self.queue.push(t) }
    }
}

impl<T: Copy, const CAPACITY: usize> Drop for Producer<'_, T, CAPACITY> {
    fn drop(&mut self) {
        self.queue.has_producer.store(true, Ordering::Relaxed);
    }
}

/// The [`Consumer`] of the [`Queue`] which can acquire values from it via [`Consumer::pop()`].
pub struct Consumer<'a, T: Copy, const CAPACITY: usize> {
    queue: &'a Queue<T, CAPACITY>,
}

impl<T: Copy, const CAPACITY: usize> Consumer<'_, T, CAPACITY> {
    /// Removes the oldest element from the queue. If the queue is empty it returns [`None`]
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.queue.pop() }
    }
}

impl<T: Copy, const CAPACITY: usize> Drop for Consumer<'_, T, CAPACITY> {
    fn drop(&mut self) {
        self.queue.has_consumer.store(true, Ordering::Relaxed);
    }
}

/// The threadsafe lock-free with a compile time fixed capacity.
pub struct Queue<T: Copy, const CAPACITY: usize> {
    data: [UnsafeCell<MaybeUninit<T>>; CAPACITY],
    write_position: IoxAtomicU64,
    read_position: IoxAtomicU64,
    has_producer: IoxAtomicBool,
    has_consumer: IoxAtomicBool,
}

unsafe impl<T: Copy + Sync, const CAPACITY: usize> Sync for Queue<T, CAPACITY> {}

impl<T: Copy, const CAPACITY: usize> Queue<T, CAPACITY> {
    /// Creates a new empty queue
    pub fn new() -> Self {
        Self {
            data: core::array::from_fn(|_| UnsafeCell::new(MaybeUninit::uninit())),
            write_position: IoxAtomicU64::new(0),
            read_position: IoxAtomicU64::new(0),
            has_producer: IoxAtomicBool::new(true),
            has_consumer: IoxAtomicBool::new(true),
        }
    }

    /// Returns a [`Producer`] to add data to the queue. If a producer was already
    /// acquired it returns [`None`].
    /// ```
    /// use iceoryx2_bb_lock_free::spsc::queue::*;
    ///
    /// const QUEUE_CAPACITY: usize = 128;
    /// let queue = Queue::<u64, QUEUE_CAPACITY>::new();
    ///
    /// let mut producer = match queue.acquire_producer() {
    ///     None => panic!("a producer has been already acquired."),
    ///     Some(p) => p,
    /// };
    ///
    /// if !producer.push(&1234) {
    ///     println!("queue is full");
    /// }
    /// ```
    pub fn acquire_producer(&self) -> Option<Producer<'_, T, CAPACITY>> {
        match self
            .has_producer
            .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => Some(Producer { queue: self }),
            Err(_) => None,
        }
    }

    /// Returns a [`Consumer`] to acquire data from the queue. If a consumer was already
    /// acquired it returns [`None`].
    /// ```
    /// use iceoryx2_bb_lock_free::spsc::queue::*;
    ///
    /// const QUEUE_CAPACITY: usize = 128;
    /// let queue = Queue::<u64, QUEUE_CAPACITY>::new();
    ///
    /// let mut consumer = match queue.acquire_consumer() {
    ///     None => panic!("a consumer has been already acquired."),
    ///     Some(p) => p,
    /// };
    ///
    /// match consumer.pop() {
    ///     None => println!("queue is empty"),
    ///     Some(v) => println!("got {}", v)
    /// }
    /// ```
    pub fn acquire_consumer(&self) -> Option<Consumer<'_, T, CAPACITY>> {
        match self
            .has_consumer
            .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => Some(Consumer { queue: self }),
            Err(_) => None,
        }
    }
    /// Push an index into the [`Queue`]. If the queue is full the oldest
    /// index is returned and replaced with the new value.
    ///
    /// # Safety
    ///
    ///  * [`Queue::push()`] cannot be called concurrently. The user has
    ///    to ensure that at most one thread access this method.
    pub unsafe fn push(&self, t: &T) -> bool {
        let current_write_pos = self.write_position.load(Ordering::Relaxed);
        ////////////////
        // SYNC POINT with `read_position` store in `pop`
        ////////////////
        let current_read_pos = self.read_position.load(Ordering::Acquire);
        let is_full = Self::check_is_full(current_write_pos, current_read_pos);

        if is_full {
            return false;
        }

        unsafe {
            self.data[(current_write_pos % (CAPACITY as u64)) as usize]
                .get()
                .write(MaybeUninit::new(*t));
        }
        ////////////////
        // SYNC POINT with `write_position` load in `pop`
        // prevent that writing to `data` is reordered after advancing of
        // `write_position` which would signal that the data is ready although
        // the data is not yet written and create a data race.
        ////////////////
        self.write_position
            .store(current_write_pos + 1, Ordering::Release);
        true
    }

    /// Acquires an index from the [`Queue`]. If the queue is empty
    /// [`None`] is returned.
    ///
    /// # Safety
    ///
    ///  * [`Queue::pop()`] cannot be called concurrently. The user has
    ///    to ensure that at most one thread access this method.
    pub unsafe fn pop(&self) -> Option<T> {
        let current_read_pos = self.read_position.load(Ordering::Relaxed);
        ////////////////
        // SYNC POINT with `write_position` store in `push`
        ////////////////
        let is_empty = current_read_pos == self.write_position.load(Ordering::Acquire);

        if is_empty {
            return None;
        }

        let out: T = unsafe {
            *self.data[(current_read_pos % (CAPACITY as u64)) as usize]
                .get()
                .as_ref()
                .unwrap()
                .as_ptr()
        };

        ////////////////
        // SYNC POINT with `read_position` load in `push`
        // prevent that reading from `data` is reordered after advancing of
        // `read_position` which would signal a free slot although the data
        // is not yet read and create a data race.
        ////////////////
        self.read_position
            .store(current_read_pos + 1, Ordering::Release);

        Some(out)
    }

    fn acquire_read_and_write_position(&self) -> (u64, u64) {
        loop {
            let write_position = self.write_position.load(Ordering::Relaxed);
            let read_position = self.read_position.load(Ordering::Relaxed);

            if write_position == self.write_position.load(Ordering::Relaxed)
                && read_position == self.read_position.load(Ordering::Relaxed)
            {
                return (write_position, read_position);
            }
        }
    }

    /// Returns true if the queue is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        let (write_position, read_position) = self.acquire_read_and_write_position();
        write_position == read_position
    }

    /// Returns the number of elements stored in the queue
    pub fn len(&self) -> usize {
        let (write_position, read_position) = self.acquire_read_and_write_position();
        (write_position - read_position) as usize
    }

    /// Returns the overall capacity of the queue
    pub fn capacity(&self) -> usize {
        CAPACITY
    }

    /// Returns true if the queue is full, otherwise false
    pub fn is_full(&self) -> bool {
        let (write_position, read_position) = self.acquire_read_and_write_position();
        Self::check_is_full(write_position, read_position)
    }

    fn check_is_full(write_pos: u64, read_pos: u64) -> bool {
        write_pos == read_pos + CAPACITY as u64
    }
}

impl<T: Copy, const CAPACITY: usize> Default for Queue<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}
