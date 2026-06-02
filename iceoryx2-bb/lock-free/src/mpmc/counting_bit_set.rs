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

use core::alloc::Layout;
use core::mem::MaybeUninit;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::{
    math::unaligned_mem_size,
    relocatable_ptr::{PointerTrait, RelocatablePointer},
};
use iceoryx2_bb_elementary_traits::{
    owning_pointer::OwningPointer, relocatable_container::RelocatableContainer,
    zero_copy_send::ZeroCopySend,
};
use iceoryx2_log::fail;
use iceoryx2_log::fatal_panic;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ZeroCopySend)]
#[repr(C)]
pub enum CounterBitSize {
    Bit8 = 1,
    #[default]
    Bit16 = 2,
    Bit32 = 4,
    Bit64 = 8,
}

impl CounterBitSize {
    pub const fn const_default() -> Self {
        CounterBitSize::Bit16
    }
}

struct Id {
    index: usize,
    bit_offset: u8,
}

impl Id {
    fn new(value: usize, bit_size: CounterBitSize) -> Self {
        let counter_size = 8 / (bit_size as usize);
        Self {
            index: value / counter_size,
            bit_offset: ((value % counter_size) * (bit_size as usize)) as u8,
        }
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

pub type CountingBitSet = details::CountingBitSet<OwningPointer<AtomicU64>>;
pub type RelocatableCountingBitSet = details::CountingBitSet<RelocatablePointer<AtomicU64>>;

#[doc(hidden)]
pub mod details {
    use super::*;

    #[derive(Debug)]
    #[repr(C)]
    pub struct CountingBitSet<PointerType: PointerTrait<AtomicU64>> {
        data_ptr: PointerType,
        capacity: usize,
        array_capacity: usize,
        counter_bit_size: CounterBitSize,
        is_memory_initialized: AtomicBool,
    }

    unsafe impl<PointerType: PointerTrait<AtomicU64> + ZeroCopySend> ZeroCopySend
        for CountingBitSet<PointerType>
    {
    }
    unsafe impl<PointerType: PointerTrait<AtomicU64>> Send for CountingBitSet<PointerType> {}
    unsafe impl<PointerType: PointerTrait<AtomicU64>> Sync for CountingBitSet<PointerType> {}

    impl CountingBitSet<OwningPointer<AtomicU64>> {
        pub fn new(capacity: usize, counter_bit_size: CounterBitSize) -> Self {
            let array_capacity = Self::array_capacity(capacity, counter_bit_size);
            let mut data_ptr = OwningPointer::<AtomicU64>::new_with_alloc(array_capacity);

            for i in 0..array_capacity {
                unsafe { data_ptr.as_mut_ptr().add(i).write(AtomicU64::new(0)) };
            }

            Self {
                data_ptr,
                capacity,
                array_capacity,
                counter_bit_size,
                is_memory_initialized: AtomicBool::new(true),
            }
        }
    }

    impl RelocatableContainer for CountingBitSet<RelocatablePointer<AtomicU64>> {
        unsafe fn new_uninit(capacity: usize) -> Self {
            unsafe { Self::new_uninit_with_counter_bit_size(capacity, CounterBitSize::default()) }
        }

        unsafe fn init<T: iceoryx2_bb_elementary::bump_allocator::BaseAllocator>(
            &mut self,
            allocator: &T,
        ) -> Result<(), iceoryx2_bb_elementary::bump_allocator::AllocationError> {
            if self.is_memory_initialized.load(Ordering::Relaxed) {
                fatal_panic!(from self,
                    "Memory already initialized. Initializing it twice may lead to undefined behavior.");
            }

            let layout = Layout::array::<AtomicU64>(self.array_capacity)
                .expect("The capacity always results in a valid layout.");
            let memory = fail!(from self, when allocator.allocate(layout),
                "Failed to initialize since the allocation of the data memory failed.");
            unsafe { self.data_ptr.init(memory) };
            for i in 0..self.array_capacity {
                unsafe { (self.data_ptr.as_mut_ptr()).add(i).write(AtomicU64::new(0)) }
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

    impl CountingBitSet<RelocatablePointer<AtomicU64>> {
        pub unsafe fn new_uninit_with_counter_bit_size(
            capacity: usize,
            counter_bit_size: CounterBitSize,
        ) -> Self {
            unsafe {
                Self {
                    data_ptr: RelocatablePointer::new_uninit(),
                    capacity,
                    counter_bit_size,
                    array_capacity: Self::array_capacity(capacity, counter_bit_size),
                    is_memory_initialized: AtomicBool::new(false),
                }
            }
        }
    }

    impl<PointerType: PointerTrait<AtomicU64>> CountingBitSet<PointerType> {
        pub const fn array_capacity(capacity: usize, counter_bit_size: CounterBitSize) -> usize {
            capacity.div_ceil(counter_bit_size as usize)
        }

        pub const fn const_memory_size(capacity: usize) -> usize {
            Self::const_memory_size_with_counter_bit_size(capacity, CounterBitSize::const_default())
        }

        pub const fn const_memory_size_with_counter_bit_size(
            capacity: usize,
            counter_bit_size: CounterBitSize,
        ) -> usize {
            unaligned_mem_size::<AtomicU64>(Self::array_capacity(capacity, counter_bit_size))
        }

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

        fn set_bit(&self, id: Id) -> u64 {
            let data_ref = unsafe { &(*self.data_ptr.as_ptr().add(id.index)) };
            let mut current = data_ref.load(Ordering::Relaxed);

            loop {
                let current_value = current >> id.bit_offset;

                let new_value = match self.counter_bit_size {
                    CounterBitSize::Bit8 => (current_value as u8).saturating_add(1) as u64,
                    CounterBitSize::Bit16 => (current_value as u16).saturating_add(1) as u64,
                    CounterBitSize::Bit32 => (current_value as u32).saturating_add(1) as u64,
                    CounterBitSize::Bit64 => (current_value).saturating_add(1),
                };

                let updated = current | (new_value << id.bit_offset);

                match data_ref.compare_exchange(
                    current,
                    updated,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return new_value,
                    Err(v) => current = v,
                };
            }
        }

        pub fn set(&self, id: usize) -> u64 {
            self.verify_init("set()");
            debug_assert!(
                id < self.capacity(),
                "This should never happen. Out of bounds access with index {id}."
            );

            self.set_bit(Id::new(id, self.counter_bit_size))
        }

        pub fn reset_all<F: FnMut(BitState)>(&self, mut callback: F) {
            self.verify_init("reset_all()");
            let number_of_elements = 8 / (self.counter_bit_size as usize);
            let element_size = self.counter_bit_size as usize;

            for index in 0..self.array_capacity {
                let value =
                    unsafe { (*self.data_ptr.as_ptr().add(index)).swap(0, Ordering::Relaxed) };
                if value == 0 {
                    continue;
                }

                for offset in 0..number_of_elements {
                    let count = (value << (element_size * (number_of_elements - offset - 1)))
                        >> (element_size * offset);

                    if count != 0 {
                        callback(BitState {
                            bit: index * number_of_elements + offset,
                            count,
                        })
                    }
                }
            }
        }
    }
}

#[derive(Debug, ZeroCopySend)]
#[repr(C)]
pub struct FixedSizeCountingBitSet<const ARRAY_CAPACITY: usize> {
    bitset: RelocatableCountingBitSet,
    data: [MaybeUninit<AtomicU64>; ARRAY_CAPACITY],
}

impl<const ARRAY_CAPACITY: usize> Default for FixedSizeCountingBitSet<ARRAY_CAPACITY> {
    fn default() -> Self {
        Self::new_with_counter_bit_size(CounterBitSize::default())
    }
}

impl<const ARRAY_CAPACITY: usize> FixedSizeCountingBitSet<ARRAY_CAPACITY> {
    pub fn new() -> Self {
        Self::new_with_counter_bit_size(CounterBitSize::default())
    }

    pub fn new_with_counter_bit_size(counter_bit_size: CounterBitSize) -> Self {
        let mut new_self = Self {
            bitset: unsafe {
                RelocatableCountingBitSet::new_uninit_with_counter_bit_size(
                    ARRAY_CAPACITY,
                    counter_bit_size,
                )
            },
            data: [const { MaybeUninit::uninit() }; ARRAY_CAPACITY],
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

    pub fn capacity(&self) -> usize {
        self.bitset.capacity()
    }

    pub fn set(&self, id: usize) -> u64 {
        self.bitset.set(id)
    }

    pub fn reset_all<F: FnMut(BitState)>(&self, callback: F) {
        self.bitset.reset_all(callback);
    }
}
