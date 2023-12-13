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

//! A **threadsafe** **lock-free** more generic alternative of the atomic. Can hold any arbitrary
//! type but is restricted to single producer multi consumer.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::*;
//! use std::sync::atomic::Ordering;
//!
//! let atomic = UnrestrictedAtomic::<[u8; 1024]>::new([0u8; 1024]);
//!
//! // store data
//! match atomic.acquire_producer() {
//!     None => panic!("a producer has been already acquired."),
//!     Some(producer) => producer.store([255u8; 1024]),
//! };
//!
//! // load data
//! let my_data = atomic.load();
//! ```

use std::{
    cell::UnsafeCell,
    fmt::Debug,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

const NUMBER_OF_CELLS: usize = 2;

/// Can be acquired via [`UnrestrictedAtomic::acquire_producer()`] if not another thread has
/// acquired one. There can only be one at a time. When it goes out of scope it deregisters at the
/// [`UnrestrictedAtomic`].
pub struct Producer<'a, T: Copy> {
    atomic: &'a UnrestrictedAtomic<T>,
}

impl<'a, T: Copy> Producer<'a, T> {
    /// Stores a `new_value` inside the atomic.
    pub fn store(&self, new_value: T) {
        self.atomic.store(new_value);
    }
}

impl<'a, T: Copy> Drop for Producer<'a, T> {
    fn drop(&mut self) {
        self.atomic.has_producer.store(true, Ordering::Relaxed);
    }
}

unsafe impl<'a, T: Copy> Send for Producer<'a, T> {}
unsafe impl<'a, T: Copy> Sync for Producer<'a, T> {}

/// An atomic implementation where the underlying type has to by copyable but is otherwise
/// unrestricted.
#[repr(C)]
pub struct UnrestrictedAtomic<T: Copy> {
    write_cell: AtomicU64,
    data: [UnsafeCell<MaybeUninit<T>>; NUMBER_OF_CELLS],
    has_producer: AtomicBool,
}

impl<T: Copy + Debug> Debug for UnrestrictedAtomic<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "UnrestrictedAtomic<{}> {{ write_cell: {}, data: {:?}, has_producer: {} }}",
            std::any::type_name::<T>(),
            self.write_cell.load(Ordering::Relaxed),
            self.load(),
            self.has_producer.load(Ordering::Relaxed)
        )
    }
}

unsafe impl<T: Copy> Send for UnrestrictedAtomic<T> {}
unsafe impl<T: Copy> Sync for UnrestrictedAtomic<T> {}

impl<T: Copy> UnrestrictedAtomic<T> {
    /// Creates a new atomic containing the provided value.
    pub fn new(value: T) -> Self {
        Self {
            has_producer: AtomicBool::new(true),
            write_cell: AtomicU64::new(1),
            data: [
                UnsafeCell::new(MaybeUninit::new(value)),
                UnsafeCell::new(MaybeUninit::uninit()),
            ],
        }
    }

    /// Returns a producer if one is available otherwise [`None`].
    pub fn acquire_producer(&self) -> Option<Producer<'_, T>> {
        match self
            .has_producer
            .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => Some(Producer { atomic: self }),
            Err(_) => None,
        }
    }

    fn store(&self, new_value: T) {
        let write_cell = self.write_cell.load(Ordering::Relaxed);
        unsafe {
            (*self.data[write_cell as usize % NUMBER_OF_CELLS].get())
                .as_mut_ptr()
                .write(new_value);
        }

        /////////////////////////
        // SYNC POINT - write
        /////////////////////////
        self.write_cell.fetch_add(1, Ordering::Release);
    }

    /// Loads the underlying value and returns a copy of it.
    pub fn load(&self) -> T {
        /////////////////////////
        // SYNC POINT - read
        /////////////////////////
        let mut read_cell = self.write_cell.load(Ordering::Acquire) - 1;
        let mut read_cell_update;

        let mut return_value;

        loop {
            unsafe {
                return_value =
                    *(*self.data[read_cell as usize % NUMBER_OF_CELLS].get()).assume_init_ref()
            };

            /////////////////////////
            // SYNC POINT - read (for write while reading)
            /////////////////////////
            read_cell_update = self.write_cell.load(Ordering::Acquire) - 1;

            if read_cell_update == read_cell {
                break;
            }

            read_cell = read_cell_update;
        }

        return_value
    }
}
