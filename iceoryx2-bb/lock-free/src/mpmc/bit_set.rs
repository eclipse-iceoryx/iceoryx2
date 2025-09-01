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
//!    runtime.
//!  * [`RelocatableBitSet`] - Stores data in the memory the allocator provides in
//!    [`RelocatableBitSet::init()`]. Is relocatable and can be used in shared memory,
//!    when the data allocator provides shared memory.
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
//!  bitset.reset_all(|id| {
//!     println!("bit {} was set", id );
//!  });
//!  ```

use core::{alloc::Layout, fmt::Debug, sync::atomic::Ordering};
use iceoryx2_bb_elementary::{
    bump_allocator::BumpAllocator,
    math::unaligned_mem_size,
    relocatable_ptr::{PointerTrait, RelocatablePointer},
};
use iceoryx2_bb_elementary_traits::{
    owning_pointer::OwningPointer, relocatable_container::RelocatableContainer,
};
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicBool, IoxAtomicU8, IoxAtomicUsize};

use iceoryx2_bb_log::{fail, fatal_panic};

/// This BitSet variant's data is stored in the heap.
pub type BitSet = details::BitSet<OwningPointer<details::BitsetElement>>;
/// This BitSet variant can be stored inside shared memory.
pub type RelocatableBitSet = details::BitSet<RelocatablePointer<details::BitsetElement>>;

#[doc(hidden)]
pub mod details {

    use super::*;

    pub type BitsetElement = IoxAtomicU8;
    const BITSET_ELEMENT_BITSIZE: usize = core::mem::size_of::<BitsetElement>() * 8;

    struct Id {
        index: usize,
        bit: u8,
    }

    impl Id {
        fn new(value: usize) -> Id {
            Self {
                index: value / BITSET_ELEMENT_BITSIZE,
                bit: (value % BITSET_ELEMENT_BITSIZE) as u8,
            }
        }
    }

    #[derive(Debug)]
    #[repr(C)]
    pub struct BitSet<PointerType: PointerTrait<BitsetElement>> {
        data_ptr: PointerType,
        capacity: usize,
        array_capacity: usize,
        reset_position: IoxAtomicUsize,
        is_memory_initialized: IoxAtomicBool,
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
                is_memory_initialized: IoxAtomicBool::new(true),
                reset_position: IoxAtomicUsize::new(0),
            }
        }
    }

    impl RelocatableContainer for BitSet<RelocatablePointer<BitsetElement>> {
        unsafe fn new_uninit(capacity: usize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new_uninit(),
                capacity,
                array_capacity: Self::array_capacity(capacity),
                is_memory_initialized: IoxAtomicBool::new(false),
                reset_position: IoxAtomicUsize::new(0),
            }
        }

        unsafe fn init<T: iceoryx2_bb_elementary_traits::allocator::BaseAllocator>(
            &mut self,
            allocator: &T,
        ) -> Result<(), iceoryx2_bb_elementary_traits::allocator::AllocationError> {
            if self.is_memory_initialized.load(Ordering::Relaxed) {
                fatal_panic!(from self,
                "Memory already initialized. Initializing it twice may lead to undefined behavior.");
            }

            let memory = fail!(from self, when allocator
            .allocate(Layout::from_size_align_unchecked(
                    core::mem::size_of::<BitsetElement>() * self.array_capacity,
                    core::mem::align_of::<BitsetElement>())),
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
                "Undefined behavior when calling BitSet::{source} and the object is not initialized."
            );
        }

        fn set_bit(&self, id: Id) -> bool {
            let data_ref = unsafe { &(*self.data_ptr.as_ptr().add(id.index)) };
            let mut current = data_ref.load(Ordering::Relaxed);
            let mask = 1 << id.bit;

            loop {
                if current & mask != 0 {
                    return false;
                }

                let current_with_bit = current | mask;

                match data_ref.compare_exchange(
                    current,
                    current_with_bit,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return true;
                    }
                    Err(v) => current = v,
                }
            }
        }

        fn clear_bit(&self, id: Id) -> bool {
            let data_ref = unsafe { &(*self.data_ptr.as_ptr().add(id.index)) };
            let mut current = data_ref.load(Ordering::Relaxed);
            let mask = 1 << id.bit;

            loop {
                if current & mask == 0 {
                    return false;
                }

                let current_with_cleared_bit = current & !mask;

                match data_ref.compare_exchange(
                    current,
                    current_with_cleared_bit,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return true;
                    }
                    Err(v) => current = v,
                }
            }
        }

        /// Sets a bit in the BitSet
        /// If the bit was successfully set it returns true, if the bit was already set it
        /// returns false.
        pub fn set(&self, id: usize) -> bool {
            self.verify_init("set()");
            debug_assert!(
                id < self.capacity,
                "This should never happen. Out of bounds access with index {id}."
            );

            self.set_bit(Id::new(id))
        }

        /// Resets the next set bit and returns the bit index. If no bit was set it returns
        /// [`None`].
        pub fn reset_next(&self) -> Option<usize> {
            self.verify_init("reset_next()");

            let current_position = self.reset_position.load(Ordering::Relaxed);
            for pos in (current_position..self.capacity).chain(0..current_position) {
                if self.clear_bit(Id::new(pos)) {
                    self.reset_position.store(pos + 1, Ordering::Relaxed);
                    return Some(pos);
                }
            }

            None
        }

        /// Reset every set bit in the BitSet and call the provided callback for every bit that
        /// was set. This is the most efficient way to acquire all bits that were set.
        pub fn reset_all<F: FnMut(usize)>(&self, mut callback: F) {
            self.verify_init("reset_all()");

            for i in 0..self.array_capacity {
                let value = unsafe { (*self.data_ptr.as_ptr().add(i)).swap(0, Ordering::Relaxed) };
                let main_index = i * BITSET_ELEMENT_BITSIZE;
                for b in 0..BITSET_ELEMENT_BITSIZE {
                    if value & (1 << b) != 0 {
                        callback(main_index + b);
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
    //       data: [`details::BitsetElement; Self::array_capacity(CAPACITY)`]
    //       For now we can live with it, since the bitsets are usually rather small
    data: [details::BitsetElement; CAPACITY],
}

unsafe impl<const CAPACITY: usize> Send for FixedSizeBitSet<CAPACITY> {}
unsafe impl<const CAPACITY: usize> Sync for FixedSizeBitSet<CAPACITY> {}

impl<const CAPACITY: usize> Default for FixedSizeBitSet<CAPACITY> {
    fn default() -> Self {
        let mut new_self = Self {
            bitset: unsafe { RelocatableBitSet::new_uninit(CAPACITY) },
            data: core::array::from_fn(|_| details::BitsetElement::new(0)),
        };

        let allocator = BumpAllocator::new(new_self.data.as_mut_ptr().cast());
        unsafe {
            new_self
                .bitset
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };

        new_self
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
    pub fn reset_next(&self) -> Option<usize> {
        self.bitset.reset_next()
    }

    /// Reset every set bit in the BitSet and call the provided callback for every bit that
    /// was set.
    pub fn reset_all<F: FnMut(usize)>(&self, callback: F) {
        self.bitset.reset_all(callback)
    }
}
