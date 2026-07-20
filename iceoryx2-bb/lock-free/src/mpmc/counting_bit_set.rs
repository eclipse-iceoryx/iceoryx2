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

//! A **threadsafe** and **lock-free** counting bitset. Each bit holds an atomic counter
//! instead of a boolean value. Bits can be incremented and read from multiple threads
//! without any restriction. It provides 3 versions.
//!  * [`CountingBitSet`] - Stores data in the heap and has a fixed capacity that must be defined at
//!    runtime.
//!  * [`RelocatableCountingBitSet`] - Stores data in the memory the allocator provides in
//!    [`RelocatableCountingBitSet::init()`]. Is relocatable and can be used in shared memory,
//!    when the data allocator provides shared memory.
//!  * [`FixedSizeCountingBitSet`] - Bitset with a compile time fixed capacity.
//!
//!  # Example
//!
//!  ```
//!  # extern crate iceoryx2_bb_loggers;
//!
//!  use iceoryx2_bb_lock_free::mpmc::counting_bit_set::*;
//!
//!  let capacity = 123;
//!  let bitset = CountingBitSet::new(capacity);
//!
//!  // increment counter for bit number 5
//!  let old_count = bitset.set(5);
//!  println!("old count: {}", old_count);
//!
//!  // resets the bitset and calls the callback for every bit that was set
//!  bitset.reset_all(|bit_state| {
//!     println!("bit {} was set {} times", bit_state.bit(), bit_state.count());
//!  });
//!  ```

use core::alloc::Layout;
use core::mem::MaybeUninit;
use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU64};
use iceoryx2_bb_concurrency::atomic::{AtomicU8, Ordering};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::{
    math::unaligned_mem_size,
    relocatable_ptr::{Pointer, RelocatablePointer},
};
use iceoryx2_bb_elementary_traits::{
    owning_pointer::OwningPointer, relocatable_container::RelocatableContainer,
    zero_copy_send::ZeroCopySend,
};
use iceoryx2_log::fail;
use iceoryx2_log::fatal_panic;

type AtomicBaseType = AtomicU64;

trait AtomicMax {
    fn max_value() -> u64;
}

impl AtomicMax for AtomicU8 {
    fn max_value() -> u64 {
        u8::MAX as u64
    }
}

impl AtomicMax for AtomicU16 {
    fn max_value() -> u64 {
        u16::MAX as u64
    }
}

impl AtomicMax for AtomicU32 {
    fn max_value() -> u64 {
        u32::MAX as u64
    }
}

impl AtomicMax for AtomicU64 {
    fn max_value() -> u64 {
        u64::MAX
    }
}

pub struct BitState {
    bit: usize,
    count: u64,
}

impl BitState {
    pub fn bit(&self) -> usize {
        self.bit
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

pub type CountingBitSet = details::CountingBitSet<OwningPointer<AtomicBaseType>>;
pub type RelocatableCountingBitSet = details::CountingBitSet<RelocatablePointer<AtomicBaseType>>;

#[doc(hidden)]
pub mod details {
    use iceoryx2_bb_elementary_traits::generic_pointer::NonNullFamily;

    use super::*;

    #[derive(Debug)]
    #[repr(C)]
    pub struct CountingBitSet<PointerType: Pointer<AtomicBaseType>> {
        data_ptr: PointerType,
        capacity: usize,
        is_memory_initialized: AtomicBool,
    }

    unsafe impl<PointerType: Pointer<AtomicBaseType> + ZeroCopySend> ZeroCopySend
        for CountingBitSet<PointerType>
    {
    }
    unsafe impl<PointerType: Pointer<AtomicBaseType>> Send for CountingBitSet<PointerType> {}
    unsafe impl<PointerType: Pointer<AtomicBaseType>> Sync for CountingBitSet<PointerType> {}

    impl CountingBitSet<OwningPointer<AtomicBaseType>> {
        /// Create a new [`CountingBitSet`] with data located in the heap.
        ///
        /// ```
        /// # extern crate iceoryx2_bb_loggers;
        ///
        /// use iceoryx2_bb_lock_free::mpmc::counting_bit_set::*;
        /// let bitset = CountingBitSet::new(123);
        /// ```
        pub fn new(capacity: usize) -> Self {
            let mut data_ptr = OwningPointer::<AtomicBaseType>::new_with_alloc(capacity);

            for i in 0..capacity {
                unsafe { data_ptr.as_mut_ptr().add(i).write(AtomicBaseType::new(0)) };
            }

            Self {
                data_ptr,
                capacity,
                is_memory_initialized: AtomicBool::new(true),
            }
        }
    }

    impl RelocatableContainer for CountingBitSet<RelocatablePointer<AtomicBaseType>> {
        unsafe fn new_uninit(capacity: usize) -> Self {
            unsafe {
                Self {
                    data_ptr: RelocatablePointer::new_uninit(),
                    capacity,
                    is_memory_initialized: AtomicBool::new(false),
                }
            }
        }

        unsafe fn init<T: iceoryx2_bb_elementary::bump_allocator::BaseAllocator<NonNullFamily>>(
            &mut self,
            allocator: &T,
        ) -> Result<(), iceoryx2_bb_elementary::bump_allocator::AllocationError> {
            if self.is_memory_initialized.load(Ordering::Relaxed) {
                fatal_panic!(from self,
                    "Memory already initialized. Initializing it twice may lead to undefined behavior.");
            }

            let layout = Layout::array::<AtomicBaseType>(self.capacity)
                .expect("The capacity always results in a valid layout.");
            let memory = fail!(from self, when allocator.allocate(layout),
                "Failed to initialize since the allocation of the data memory failed.");
            unsafe { self.data_ptr.init(memory) };
            for i in 0..self.capacity {
                unsafe {
                    (self.data_ptr.as_mut_ptr())
                        .add(i)
                        .write(AtomicBaseType::new(0))
                }
            }

            // relaxed is sufficient since no relocatable container can be used
            // before init was called. Meaning, it is not allowed to send or share
            // the container with other threads when it is in an uninitialized state.
            self.is_memory_initialized.store(true, Ordering::Relaxed);

            Ok(())
        }

        fn memory_size(capacity: usize) -> usize {
            Self::const_memory_size(capacity)
        }
    }

    impl<PointerType: Pointer<AtomicBaseType>> CountingBitSet<PointerType> {
        /// Returns the required memory size for a [`CountingBitSet`] with a specified capacity.
        pub const fn const_memory_size(capacity: usize) -> usize {
            unaligned_mem_size::<AtomicBaseType>(capacity)
        }

        /// Returns the capacity of the [`CountingBitSet`].
        pub fn capacity(&self) -> usize {
            self.capacity
        }

        #[inline(always)]
        fn verify_init(&self, source: &str) {
            debug_assert!(
                self.is_memory_initialized.load(Ordering::Relaxed),
                "Undefined behavior when calling CountingBitSet::{source} and the object is not initialized."
            );
        }

        /// Returns the maximum value a bit's counter can hold.
        pub fn max_count() -> u64 {
            AtomicBaseType::max_value()
        }

        /// Increments the counter for the bit at the given index and returns the previous count.
        pub fn set(&self, id: usize) -> u64 {
            self.verify_init("set()");
            debug_assert!(
                id < self.capacity(),
                "This should never happen. Out of bounds access with index {id}."
            );

            unsafe { &(*self.data_ptr.as_ptr().add(id)) }.fetch_add(1, Ordering::Relaxed) as _
        }

        /// Reset every bit in the [`CountingBitSet`] and call the provided callback for every bit
        /// that had a non-zero count.
        pub fn reset_all<F: FnMut(BitState)>(&self, mut callback: F) {
            self.verify_init("reset_all()");

            for bit in 0..self.capacity {
                let count =
                    unsafe { (*self.data_ptr.as_ptr().add(bit)).swap(0, Ordering::Relaxed) };
                if count == 0 {
                    continue;
                }

                callback(BitState {
                    bit,
                    count: count as _,
                })
            }
        }
    }
}

#[derive(Debug, ZeroCopySend)]
#[repr(C)]
pub struct FixedSizeCountingBitSet<const CAPACITY: usize> {
    bitset: RelocatableCountingBitSet,
    data: [MaybeUninit<AtomicU64>; CAPACITY],
}

impl<const CAPACITY: usize> Default for FixedSizeCountingBitSet<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAPACITY: usize> FixedSizeCountingBitSet<CAPACITY> {
    /// Creates a new FixedSizeCountingBitSet.
    pub fn new() -> Self {
        let mut new_self = Self {
            bitset: unsafe { RelocatableCountingBitSet::new_uninit(CAPACITY) },
            data: [const { MaybeUninit::uninit() }; CAPACITY],
        };

        let data_ptr =
            unsafe { core::ptr::NonNull::<u8>::new_unchecked(new_self.data.as_mut_ptr().cast()) };
        let allocator =
            BumpAllocator::new(data_ptr, core::mem::size_of_val(new_self.data.as_ref()));

        unsafe {
            new_self
                .bitset
                .init(&allocator)
                .expect("All required memory is preallocated")
        };

        new_self
    }

    /// Returns the capacity.
    pub fn capacity(&self) -> usize {
        self.bitset.capacity()
    }

    /// Increments the counter for the bit at the given index and returns the previous count.
    pub fn set(&self, id: usize) -> u64 {
        self.bitset.set(id)
    }

    /// Reset every bit in the CountingBitSet and call the provided callback for every bit
    /// that had a non-zero count.
    pub fn reset_all<F: FnMut(BitState)>(&self, callback: F) {
        self.bitset.reset_all(callback);
    }
}
