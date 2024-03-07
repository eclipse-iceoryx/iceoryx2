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

pub type BitSet = details::BitSet<OwningPointer<AtomicU8>>;
pub type RelocatableBitSet = details::BitSet<RelocatablePointer<AtomicU8>>;

pub mod details {
    use super::*;

    #[derive(Debug)]
    #[repr(C)]
    pub struct BitSet<PointerType: PointerTrait<AtomicU8>> {
        data_ptr: PointerType,
        capacity: usize,
        array_capacity: usize,
        is_memory_initialized: AtomicBool,
    }

    unsafe impl<PointerType: PointerTrait<AtomicU8>> Send for BitSet<PointerType> {}
    unsafe impl<PointerType: PointerTrait<AtomicU8>> Sync for BitSet<PointerType> {}

    impl BitSet<OwningPointer<AtomicU8>> {
        pub fn new(capacity: usize) -> Self {
            let array_capacity = Self::array_capacity(capacity);
            let mut data_ptr = OwningPointer::<AtomicU8>::new_with_alloc(array_capacity);

            for i in 0..array_capacity {
                unsafe { data_ptr.as_mut_ptr().add(i).write(AtomicU8::new(0u8)) };
            }

            Self {
                data_ptr,
                capacity,
                array_capacity,
                is_memory_initialized: AtomicBool::new(true),
            }
        }
    }

    impl RelocatableContainer for BitSet<RelocatablePointer<AtomicU8>> {
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
                    std::mem::size_of::<AtomicU8>() * self.array_capacity,
                    std::mem::align_of::<AtomicU8>())),
            "Failed to initialize since the allocation of the data memory failed.");

            self.data_ptr.init(memory);

            for i in 0..self.array_capacity {
                unsafe {
                    (self.data_ptr.as_ptr() as *mut AtomicU8)
                        .add(i)
                        .write(AtomicU8::new(0u8))
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

    impl<PointerType: PointerTrait<AtomicU8> + Debug> BitSet<PointerType> {
        pub(super) const fn array_capacity(capacity: usize) -> usize {
            capacity.div_ceil(8)
        }
        pub const fn const_memory_size(capacity: usize) -> usize {
            unaligned_mem_size::<AtomicU8>(Self::array_capacity(capacity))
        }

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

        fn set_bit(&self, index: usize, bit: usize) {
            let data_ref = unsafe { &(*self.data_ptr.as_ptr().add(index)) };
            let mut current = data_ref.load(Ordering::Relaxed);
            loop {
                let current_with_bit_set = current & (1 << bit);

                match data_ref.compare_exchange(
                    current,
                    current_with_bit_set,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(v) => current = v,
                }
            }
        }

        pub fn set(&self, id: usize) {
            self.verify_init("set");
            debug_assert!(
                id < self.capacity,
                "This should never happen. Out of bounds access with index {}.",
                id
            );

            let bitset_index = id / 8;
            let bit = id % 8;
            self.set_bit(bitset_index, bit);
        }

        pub fn reset<F: FnMut(usize)>(&self, mut callback: F) {
            self.verify_init("set");
            for i in 0..self.array_capacity {
                let value =
                    unsafe { (*self.data_ptr.as_ptr().add(i)).swap(0u8, Ordering::Relaxed) };
                for b in 0..8 {
                    if value & (1 << b) != 0 {
                        let index = i * 8 + b;
                        callback(index);
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeBitSet<const CAPACITY: usize> {
    bitset: RelocatableBitSet,
    // TODO: we waste here some memory since rust does us not allow to perform const operations
    //       on generic parameters. Whenever this is supported, change this line into
    //       data: [AtomicU8; Self::array_capacity(CAPACITY)]
    //       For now we can live with it, since the bitsets are usually rather small
    data: [AtomicU8; CAPACITY],
}

impl<const CAPACITY: usize> Default for FixedSizeBitSet<CAPACITY> {
    fn default() -> Self {
        Self {
            bitset: unsafe {
                RelocatableBitSet::new(
                    CAPACITY,
                    align_to::<AtomicU8>(std::mem::size_of::<RelocatableBitSet>()) as _,
                )
            },
            data: core::array::from_fn(|_| AtomicU8::new(0u8)),
        }
    }
}

impl<const CAPACITY: usize> FixedSizeBitSet<CAPACITY> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn capacity(&self) -> usize {
        self.bitset.capacity()
    }

    pub fn set(&self, id: usize) {
        self.bitset.set(id)
    }

    pub fn reset<F: FnMut(usize)>(&self, callback: F) {
        self.bitset.reset(callback)
    }
}
