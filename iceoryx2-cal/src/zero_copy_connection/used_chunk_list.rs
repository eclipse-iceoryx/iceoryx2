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
    sync::atomic::{AtomicBool, Ordering},
};

use iceoryx2_bb_elementary::{
    math::align_to,
    owning_pointer::OwningPointer,
    relocatable_container::RelocatableContainer,
    relocatable_ptr::{PointerTrait, RelocatablePointer},
};
use iceoryx2_bb_log::{fail, fatal_panic};

use crate::shm_allocator::PointerOffset;

pub type UsedChunkList = details::UsedChunkList<OwningPointer<AtomicBool>>;
pub type RelocatableUsedChunkList = details::UsedChunkList<RelocatablePointer<AtomicBool>>;

pub mod details {
    use std::fmt::Debug;

    use iceoryx2_bb_elementary::owning_pointer::OwningPointer;

    use super::*;

    #[derive(Debug)]
    #[repr(C)]
    pub struct UsedChunkList<PointerType: PointerTrait<AtomicBool>> {
        data_ptr: PointerType,
        capacity: usize,
        is_memory_initialized: AtomicBool,
    }

    unsafe impl<PointerType: PointerTrait<AtomicBool>> Send for UsedChunkList<PointerType> {}
    unsafe impl<PointerType: PointerTrait<AtomicBool>> Sync for UsedChunkList<PointerType> {}

    impl UsedChunkList<OwningPointer<AtomicBool>> {
        pub fn new(capacity: usize) -> Self {
            let mut data_ptr = OwningPointer::<AtomicBool>::new_with_alloc(capacity);

            for i in 0..capacity {
                unsafe { data_ptr.as_mut_ptr().add(i).write(AtomicBool::new(false)) };
            }

            Self {
                data_ptr,
                capacity,
                is_memory_initialized: AtomicBool::new(true),
            }
        }
    }

    impl RelocatableContainer for UsedChunkList<RelocatablePointer<AtomicBool>> {
        unsafe fn new_uninit(capacity: usize) -> Self {
            Self {
                data_ptr: RelocatablePointer::new_uninit(),
                capacity,
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
                    std::mem::size_of::<AtomicBool>() * self.capacity,
                    std::mem::align_of::<AtomicBool>())),
            "Failed to initialize since the allocation of the data memory failed.");

            self.data_ptr.init(memory);

            for i in 0..self.capacity {
                unsafe {
                    (self.data_ptr.as_ptr() as *mut AtomicBool)
                        .add(i)
                        .write(AtomicBool::new(false))
                };
            }

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
                is_memory_initialized: AtomicBool::new(true),
            }
        }
    }

    impl<PointerType: PointerTrait<AtomicBool> + Debug> UsedChunkList<PointerType> {
        pub const fn const_memory_size(capacity: usize) -> usize {
            std::mem::size_of::<AtomicBool>() * capacity + std::mem::align_of::<AtomicBool>() - 1
        }

        pub fn capacity(&self) -> usize {
            self.capacity
        }

        fn verify_init(&self, source: &str) {
            if !self.is_memory_initialized.load(Ordering::Relaxed) {
                fatal_panic!(from self, "Undefined behavior when calling \"{}\" and the object is not initialized.", source);
            }
        }

        fn set(&self, idx: usize, value: bool) -> bool {
            self.verify_init("set");

            if self.capacity <= idx {
                fatal_panic!(from self,
                "This should never happen. Out of bounds access with index {}.", idx);
            }

            unsafe { (*self.data_ptr.as_ptr().add(idx)).swap(value, Ordering::Relaxed) }
        }

        pub fn insert(&self, value: usize) -> bool {
            !self.set(value, true)
        }

        pub fn remove(&self, value: usize) -> bool {
            self.set(value, false)
        }

        pub fn pop(&self) -> Option<usize> {
            self.verify_init("pop");

            for i in 0..self.capacity {
                if unsafe { (*self.data_ptr.as_ptr().add(i)).swap(false, Ordering::Relaxed) } {
                    return Some(i);
                }
            }

            None
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeUsedChunkList<const CAPACITY: usize> {
    list: RelocatableUsedChunkList,
    data: [AtomicBool; CAPACITY],
}

impl<const CAPACITY: usize> FixedSizeUsedChunkList<CAPACITY> {
    pub fn new() -> Self {
        Self {
            list: unsafe {
                RelocatableUsedChunkList::new(
                    CAPACITY,
                    align_to::<PointerOffset>(std::mem::size_of::<RelocatableUsedChunkList>()) as _,
                )
            },
            data: core::array::from_fn(|_| AtomicBool::new(false)),
        }
    }

    pub fn insert(&self, value: usize) -> bool {
        self.list.insert(value)
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.list.pop()
    }

    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    pub fn remove(&self, value: usize) -> bool {
        self.list.remove(value)
    }
}
