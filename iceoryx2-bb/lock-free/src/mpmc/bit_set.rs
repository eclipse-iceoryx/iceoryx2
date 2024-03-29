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

//! A **threadsafe** and **lock-free** bitset. Bits can be set and read from multiple threads
//! without any restriction. It provides 3 versions.
//!  * [`BitSet`] - Stores data in the heap and has a fixed capacity that must be defined at
//!                 runtime.
//!  * [`RelocatableBitSet`] - Stores data in the memory the allocator provides in
//!                 [`RelocatableBitSet::init()`]. Is relocatable and can be used in shared memory,
//!                 when the data allocator provides shared memory.
//!  * [`FixedSizeBitSet`] - Bitset with a compile time fixed capacity.
//!
//!  # Example
//!
//!  ```
//!  use iceoryx2_bb_lock_free::mpmc::bit_set::*;
//!
//!  let capacity = 123;
//!  let bitset = BitSet::new(capacity);
//!
//!  // set bit number 5
//!  bitset.set(5);
//!
//!  // resets the bitset and calls the callback for every bit that was set
//!  bitset.reset(|id| {
//!     println!("bit {} was set", id );
//!  });
//!  ```

use std::{
    alloc::Layout,
    fmt::Debug,
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};

use iceoryx2_bb_elementary::{
    math::align_to,
    math::unaligned_mem_size,
    owning_pointer::OwningPointer,
    relocatable_container::RelocatableContainer,
    relocatable_ptr::{PointerTrait, RelocatablePointer},
};

use iceoryx2_bb_log::{fail, fatal_panic};

type BitsetElement = AtomicU8;
const BITSET_ELEMENT_BITSIZE: usize = core::mem::size_of::<BitsetElement>() * 8;

/// This BitSet variant's data is stored in the heap.
pub type BitSet = details::BitSet<OwningPointer<BitsetElement>>;
/// This BitSet variant can be stored inside shared memory.
pub type RelocatableBitSet = details::BitSet<RelocatablePointer<BitsetElement>>;

pub mod details {
    use super::*;

    #[derive(Debug)]
    #[repr(C)]
    pub struct BitSet<PointerType: PointerTrait<BitsetElement>> {
        data_ptr: PointerType,
        capacity: usize,
        array_capacity: usize,
        is_memory_initialized: AtomicBool,
    }

    unsafe impl<PointerType: PointerTrait<BitsetElement>> Send for BitSet<PointerType> {}
    unsafe impl<PointerType: PointerTrait<BitsetElement>> Sync for BitSet<PointerType> {}

    impl BitSet<OwningPointer<BitsetElement>> {
        /// Create a new [`BitSet`] with data located in the heap.
        ///
        /// ```
        /// use iceoryx2_bb_lock_free::mpmc::bit_set::*;
        /// let bitset = BitSet::new(123);
        /// ```
        pub fn new(capacity: usize) -> Self {
            let array_capacity = Self::array_capacity(capacity);
            let mut data_ptr = OwningPointer::<BitsetElement>::new_with_alloc(array_capacity);

            for i in 0..array_capacity {
                unsafe { data_ptr.as_mut_ptr().add(i).write(BitsetElement::new(0)) };
            }

            Self {
                data_ptr,
                capacity,
                array_capacity,
                is_memory_initialized: AtomicBool::new(true),
            }
        }
    }

    impl RelocatableContainer for BitSet<RelocatablePointer<BitsetElement>> {
        unsafe fn new_uninit(capacity: usize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new_uninit(),
                capacity,
                array_capacity: Self::array_capacity(capacity),
                is_memory_initialized: AtomicBool::new(false),
            }
        }

        unsafe fn init<T: iceoryx2_bb_elementary::allocator::BaseAllocator>(
            &self,
            allocator: &T,
        ) -> Result<(), iceoryx2_bb_elementary::allocator::AllocationError> {
            if self.is_memory_initialized.load(Ordering::Relaxed) {
                fatal_panic!(from self,
                "Memory already initialized. Initializing it twice may lead to undefined behavior.");
            }

            let memory = fail!(from self, when allocator
            .allocate(Layout::from_size_align_unchecked(
                    std::mem::size_of::<BitsetElement>() * self.array_capacity,
                    std::mem::align_of::<BitsetElement>())),
            "Failed to initialize since the allocation of the data memory failed.");

            self.data_ptr.init(memory);

            for i in 0..self.array_capacity {
                unsafe {
                    (self.data_ptr.as_ptr() as *mut BitsetElement)
                        .add(i)
                        .write(BitsetElement::new(0))
                };
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

        unsafe fn new(capacity: usize, distance_to_data: isize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new(distance_to_data),
                capacity,
                array_capacity: Self::array_capacity(capacity),
                is_memory_initialized: AtomicBool::new(true),
            }
        }
    }

    impl<PointerType: PointerTrait<BitsetElement> + Debug> BitSet<PointerType> {
        pub(super) const fn array_capacity(capacity: usize) -> usize {
            capacity.div_ceil(BITSET_ELEMENT_BITSIZE)
        }

        /// Returns the required memory size for a BitSet with a specified capacity.
        pub const fn const_memory_size(capacity: usize) -> usize {
            unaligned_mem_size::<BitsetElement>(Self::array_capacity(capacity))
        }

        /// Returns the capacity of the BitSet
        pub fn capacity(&self) -> usize {
            self.capacity
        }

        #[inline(always)]
        fn verify_init(&self, source: &str) {
            debug_assert!(
                self.is_memory_initialized.load(Ordering::Relaxed),
                "Undefined behavior when calling \"{}\" and the object is not initialized.",
                source
            );
        }

        fn set_bit(&self, index: usize, bit: usize) -> bool {
            let data_ref = unsafe { &(*self.data_ptr.as_ptr().add(index)) };
            let mut current = data_ref.load(Ordering::Relaxed);
            let bit = 1 << bit;

            loop {
                if current & bit != 0 {
                    return false;
                }

                let current_with_bit = current | bit;

                match data_ref.compare_exchange(
                    current,
                    current_with_bit,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return true,
                    Err(v) => current = v,
                }
            }
        }

        /// Sets a bit in the BitSet
        pub fn set(&self, id: usize) -> bool {
            self.verify_init("set");
            debug_assert!(
                id < self.capacity,
                "This should never happen. Out of bounds access with index {}.",
                id
            );

            let bitset_index = id / BITSET_ELEMENT_BITSIZE;
            let bit = id % BITSET_ELEMENT_BITSIZE;
            self.set_bit(bitset_index, bit)
        }

        /// Reset every set bit in the BitSet and call the provided callback for every bit that
        /// was set.
        pub fn reset<F: FnMut(usize)>(&self, mut callback: F) {
            self.verify_init("set");
            for i in 0..self.array_capacity {
                let value = unsafe { (*self.data_ptr.as_ptr().add(i)).swap(0, Ordering::Relaxed) };
                for b in 0..BITSET_ELEMENT_BITSIZE {
                    if value & (1 << b) != 0 {
                        let index = i * BITSET_ELEMENT_BITSIZE + b;
                        callback(index);
                    }
                }
            }
        }
    }
}

/// This BitSet variant owns all data it requires.
#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeBitSet<const CAPACITY: usize> {
    bitset: RelocatableBitSet,
    // TODO: we waste here some memory since rust does us not allow to perform const operations
    //       on generic parameters. Whenever this is supported, change this line into
    //       data: [BitsetElement; Self::array_capacity(CAPACITY)]
    //       For now we can live with it, since the bitsets are usually rather small
    data: [BitsetElement; CAPACITY],
}

unsafe impl<const CAPACITY: usize> Send for FixedSizeBitSet<CAPACITY> {}
unsafe impl<const CAPACITY: usize> Sync for FixedSizeBitSet<CAPACITY> {}

impl<const CAPACITY: usize> Default for FixedSizeBitSet<CAPACITY> {
    fn default() -> Self {
        Self {
            bitset: unsafe {
                RelocatableBitSet::new(
                    CAPACITY,
                    align_to::<BitsetElement>(std::mem::size_of::<RelocatableBitSet>()) as _,
                )
            },
            data: core::array::from_fn(|_| BitsetElement::new(0)),
        }
    }
}

impl<const CAPACITY: usize> FixedSizeBitSet<CAPACITY> {
    /// Creates a new FixedSizeBitSet
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the capacity
    pub fn capacity(&self) -> usize {
        self.bitset.capacity()
    }

    /// Sets a bit in the BitSet
    pub fn set(&self, id: usize) -> bool {
        self.bitset.set(id)
    }

    /// Reset every set bit in the BitSet and call the provided callback for every bit that
    /// was set.
    pub fn reset<F: FnMut(usize)>(&self, callback: F) {
        self.bitset.reset(callback)
    }
}
