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
//! use core::sync::atomic::Ordering;
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

use core::{cell::UnsafeCell, fmt::Debug, mem::MaybeUninit, sync::atomic::Ordering};

use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU32};

// ATTENTION: To ensure the functionality also in the case of an overflow with the 'write_cell'
// value, the value of `NUMBER_OF_CELLS` must be a power of two
const NUMBER_OF_CELLS: usize = 2;

/// Can be acquired via [`UnrestrictedAtomic::acquire_producer()`] if not another thread has
/// acquired one. There can only be one at a time. When it goes out of scope it deregisters at the
/// [`UnrestrictedAtomic`].
pub struct Producer<'a, T: Copy> {
    atomic: &'a UnrestrictedAtomic<T>,
}

impl<T: Copy> Producer<'_, T> {
    /// Stores a `new_value` inside the atomic.
    pub fn store(&self, new_value: T) {
        self.atomic.store(new_value);
    }
}

impl<T: Copy> Drop for Producer<'_, T> {
    fn drop(&mut self) {
        self.atomic.has_producer.store(true, Ordering::Relaxed);
    }
}

unsafe impl<T: Copy> Send for Producer<'_, T> {}
unsafe impl<T: Copy> Sync for Producer<'_, T> {}

/// An atomic implementation where the underlying type has to by copyable but is otherwise
/// unrestricted.
#[repr(C)]
pub struct UnrestrictedAtomic<T: Copy> {
    write_cell: IoxAtomicU32,
    data: [UnsafeCell<MaybeUninit<T>>; NUMBER_OF_CELLS],
    has_producer: IoxAtomicBool,
}

impl<T: Copy + Debug> Debug for UnrestrictedAtomic<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UnrestrictedAtomic<{}> {{ write_cell: {}, data: {:?}, has_producer: {} }}",
            core::any::type_name::<T>(),
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
            has_producer: IoxAtomicBool::new(true),
            write_cell: IoxAtomicU32::new(1),
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
        // prevent reordering of `data` after advancing `write_cell` which would signal
        // the completion of the store operation an would result in a data race when
        // `data` would be written after the `write_cell` operation
        /////////////////////////
        self.write_cell.fetch_add(1, Ordering::Release);
    }

    /// Loads the underlying value and returns a copy of it.
    pub fn load(&self) -> T {
        /////////////////////////
        // SYNC POINT - read
        /////////////////////////
        let mut read_cell = self.write_cell.load(Ordering::Acquire) - 1;

        let mut return_value;

        loop {
            unsafe {
                return_value =
                    *(*self.data[read_cell as usize % NUMBER_OF_CELLS].get()).assume_init_ref()
            };

            /////////////////////////
            // SYNC POINT - read (for write while reading)
            // prevent reordering of reading from `data` after checking for a change
            // of the `write_cell` position which would result in a data race
            /////////////////////////
            let expected_write_cell = read_cell + 1;
            let write_cell_result = self.write_cell.compare_exchange(
                expected_write_cell,
                expected_write_cell,
                Ordering::Release,
                Ordering::Acquire,
            );
            if let Err(write_cell) = write_cell_result {
                read_cell = write_cell - 1;
            } else {
                break;
            }
        }

        return_value
    }
}
