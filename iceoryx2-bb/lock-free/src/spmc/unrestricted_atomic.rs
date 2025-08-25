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

use iceoryx2_bb_elementary::math::{align, max};
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

    #[doc(hidden)]
    // # Safety
    //
    //   * the memory position must not be modified after
    //     [`Producer::__internal_update_write_cell()`] has been called
    pub unsafe fn __internal_get_ptr_to_write_cell(&self) -> *mut T {
        let write_cell = self.atomic.mgmt.write_cell.load(Ordering::Relaxed);
        unsafe { (*self.atomic.data[write_cell as usize % NUMBER_OF_CELLS].get()).as_mut_ptr() }
    }

    #[doc(hidden)]
    // # Safety
    //
    //   * the method must not be called without first writing to the memory position returned by
    //     [`Producer::__internal_get_ptr_to_write_cell()`]
    pub unsafe fn __internal_update_write_cell(&self) {
        /////////////////////////
        // SYNC POINT - write
        // After writing the content of the write_cell, the content needs to be synced with the
        // reader.
        /////////////////////////
        self.atomic.mgmt.write_cell.fetch_add(1, Ordering::Release);
    }
}

impl<T: Copy> Drop for Producer<'_, T> {
    fn drop(&mut self) {
        self.atomic.mgmt.has_producer.store(true, Ordering::Relaxed);
    }
}

unsafe impl<T: Copy> Send for Producer<'_, T> {}
unsafe impl<T: Copy> Sync for Producer<'_, T> {}

#[doc(hidden)]
#[repr(C)]
pub struct UnrestrictedAtomicMgmt {
    write_cell: IoxAtomicU32,
    has_producer: IoxAtomicBool,
}

impl Default for UnrestrictedAtomicMgmt {
    fn default() -> Self {
        Self {
            write_cell: IoxAtomicU32::new(1),
            has_producer: IoxAtomicBool::new(true),
        }
    }
}

impl UnrestrictedAtomicMgmt {
    pub fn new() -> Self {
        Self::default()
    }

    #[doc(hidden)]
    /// # Safety
    ///
    ///   * store operations are only allowed when this method returns Ok
    ///   * [`UnrestrictedAtomicMgmt::__internal_release_producer()`] must be called when the
    ///     [`UnrestrictedAtomicMgmt`] (used without [`UnrestrictedAtomic`]) is dropped
    pub unsafe fn __internal_acquire_producer(&self) -> Result<bool, bool> {
        self.has_producer
            .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed)
    }

    #[doc(hidden)]
    /// # Safety
    ///
    ///   * store operations are not allowed after this method was called
    ///   * [`UnrestrictedAtomicMgmt::__internal_acquire_producer()`] must have been
    ///     successfully called before
    pub unsafe fn __internal_release_producer(&self) {
        self.has_producer.store(true, Ordering::Relaxed);
    }

    #[doc(hidden)]
    /// # Safety
    ///
    ///   * [`UnrestrictedAtomicMgmt::__internal_acquire_producer()`] must have been
    ///     successfully called before
    ///   * the memory position must not be modified after
    ///     [`UnrestrictedAtomicMgmt::__internal_update_write_cell()`] has been called
    ///   * the memory position must not be read before
    ///     [`UnrestrictedAtomicMgmt::__internal_update_write_cell()`] has been called
    pub unsafe fn __internal_get_ptr_to_write_cell(
        &self,
        value_size: usize,
        value_alignment: usize,
        data_ptr: *mut u8,
    ) -> *mut u8 {
        let write_cell = self.write_cell.load(Ordering::Relaxed);
        unsafe {
            let data_cell_ptr =
                Self::__internal_get_data_cell(value_size, value_alignment, data_ptr, write_cell);
            data_cell_ptr as *mut u8
        }
    }

    #[doc(hidden)]
    /// # Safety
    ///
    ///   * the method must not be called without first writing to the memory position returned by
    ///     [`UnrestrictedAtomicMgmt::__internal_get_ptr_to_write_cell()`]
    pub unsafe fn __internal_update_write_cell(&self) {
        /////////////////////////
        // SYNC POINT - write
        // After writing the content of the write_cell, the content needs to be synced with the
        // reader.
        /////////////////////////
        self.write_cell.fetch_add(1, Ordering::Release);
    }

    /// # Safety
    ///
    ///   * see Safety section of core::ptr::copy_nonoverlapping
    pub unsafe fn load(
        &self,
        value_ptr: *mut u8,
        value_size: usize,
        value_alignment: usize,
        data_ptr: *const u8,
    ) {
        /////////////////////////
        // SYNC POINT - read
        /////////////////////////
        let mut read_cell = self.write_cell.load(Ordering::Acquire) - 1;

        loop {
            unsafe {
                let data_cell_ptr = Self::__internal_get_data_cell(
                    value_size,
                    value_alignment,
                    data_ptr,
                    read_cell,
                );
                core::ptr::copy_nonoverlapping(data_cell_ptr as *const u8, value_ptr, value_size);
            }

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
    }

    /// # Safety
    ///
    ///   * see Safety section of core::ptr::add
    pub unsafe fn __internal_get_data_cell(
        value_size: usize,
        value_alignment: usize,
        data_ptr: *const u8,
        cell: u32,
    ) -> usize {
        align(
            unsafe { data_ptr.add(value_size * (cell as usize % NUMBER_OF_CELLS)) } as usize,
            value_alignment,
        )
    }

    #[doc(hidden)]
    /// Returns the size of an UnrestrictedAtomic<T> when value_size = size_of::<T>() and
    /// value_alignment = align_of::<T>()
    pub fn __internal_get_unrestricted_atomic_size(
        value_size: usize,
        value_alignment: usize,
    ) -> usize {
        let atomic_alignment =
            UnrestrictedAtomicMgmt::__internal_get_unrestricted_atomic_alignment(value_alignment);
        // atomic_size = aligned size of data + aligned size of UnrestrictedAtomicMgmt
        let atomic_size = 2 * align(value_size, value_alignment)
            + align(
                core::mem::size_of::<UnrestrictedAtomicMgmt>(),
                atomic_alignment,
            );
        // Align atomic_size to a multiple of atomic_alignment because the size of UnrestrictedAtomic must
        // be a multiple of its alignment.
        align(atomic_size, atomic_alignment)
    }

    #[doc(hidden)]
    /// Returns the alignment of an UnrestrictedAtomic<T> when
    /// value_alignment = align_of::<T>()
    pub fn __internal_get_unrestricted_atomic_alignment(value_alignment: usize) -> usize {
        max(
            core::mem::align_of::<UnrestrictedAtomicMgmt>(),
            value_alignment,
        )
    }
}

/// An atomic implementation where the underlying type has to be copyable but is otherwise
/// unrestricted.
#[repr(C)]
pub struct UnrestrictedAtomic<T: Copy> {
    mgmt: UnrestrictedAtomicMgmt,
    data: [UnsafeCell<MaybeUninit<T>>; NUMBER_OF_CELLS],
}

impl<T: Copy + Debug> Debug for UnrestrictedAtomic<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UnrestrictedAtomic<{}> {{ write_cell: {}, data: {:?}, has_producer: {} }}",
            core::any::type_name::<T>(),
            self.mgmt.write_cell.load(Ordering::Relaxed),
            self.load(),
            self.mgmt.has_producer.load(Ordering::Relaxed)
        )
    }
}

unsafe impl<T: Copy> Send for UnrestrictedAtomic<T> {}
unsafe impl<T: Copy> Sync for UnrestrictedAtomic<T> {}

impl<T: Copy> UnrestrictedAtomic<T> {
    /// Creates a new atomic containing the provided value.
    pub fn new(value: T) -> Self {
        Self {
            mgmt: UnrestrictedAtomicMgmt {
                has_producer: IoxAtomicBool::new(true),
                write_cell: IoxAtomicU32::new(1),
            },
            data: [
                UnsafeCell::new(MaybeUninit::new(value)),
                UnsafeCell::new(MaybeUninit::uninit()),
            ],
        }
    }

    /// Returns a producer if one is available otherwise [`None`].
    pub fn acquire_producer(&self) -> Option<Producer<'_, T>> {
        match unsafe { self.mgmt.__internal_acquire_producer() } {
            Ok(_) => Some(Producer { atomic: self }),
            Err(_) => None,
        }
    }

    fn store(&self, new_value: T) {
        let write_cell = self.mgmt.write_cell.load(Ordering::Relaxed);
        unsafe {
            (*self.data[write_cell as usize % NUMBER_OF_CELLS].get())
                .as_mut_ptr()
                .write(new_value);
        }

        /////////////////////////
        // SYNC POINT - write
        // prevent reordering of `data` after advancing `write_cell` which would signal
        // the completion of the store operation and would result in a data race when
        // `data` would be written after the `write_cell` operation
        /////////////////////////
        self.mgmt.write_cell.fetch_add(1, Ordering::Release);
    }

    /// Loads the underlying value and returns a copy of it.
    pub fn load(&self) -> T {
        let mut return_value: MaybeUninit<T> = MaybeUninit::uninit();
        unsafe {
            self.mgmt.load(
                return_value.as_mut_ptr().cast(),
                core::mem::size_of::<T>(),
                core::mem::align_of::<T>(),
                self.data.as_ptr().cast(),
            );
            return_value.assume_init()
        }
    }

    #[doc(hidden)]
    pub fn __internal_get_mgmt(&self) -> &UnrestrictedAtomicMgmt {
        &self.mgmt
    }

    #[doc(hidden)]
    pub fn __internal_get_data_ptr(&self) -> *mut u8 {
        self.data.as_ptr() as *mut u8
    }
}

/// Used for the language bindings where the type to store in the [`UnrestrictedAtomic`]
/// cannot be passed as generic.
#[doc(hidden)]
pub struct __InternalPtrs {
    pub atomic_mgmt_ptr: *mut u8,
    pub atomic_payload_ptr: *mut u8,
}

/// Used for the language bindings where the type to store in the [`UnrestrictedAtomic`]
/// cannot be passed as generic.
#[doc(hidden)]
pub unsafe fn __internal_calculate_atomic_mgmt_and_payload_ptr(
    raw_memory_ptr: *mut u8,
    value_alignment: usize,
) -> __InternalPtrs {
    let atomic_mgmt_alignment_offset =
        raw_memory_ptr.align_offset(align_of::<UnrestrictedAtomicMgmt>().max(value_alignment));
    let atomic_mgmt_ptr: *mut UnrestrictedAtomicMgmt =
        unsafe { raw_memory_ptr.add(atomic_mgmt_alignment_offset).cast() };
    unsafe { atomic_mgmt_ptr.write(UnrestrictedAtomicMgmt::new()) };

    let payload_ptr = atomic_mgmt_ptr as usize + core::mem::size_of::<UnrestrictedAtomicMgmt>();
    let payload_ptr = align(payload_ptr, value_alignment);

    __InternalPtrs {
        atomic_mgmt_ptr: atomic_mgmt_ptr as *mut u8,
        atomic_payload_ptr: payload_ptr as *mut u8,
    }
}
