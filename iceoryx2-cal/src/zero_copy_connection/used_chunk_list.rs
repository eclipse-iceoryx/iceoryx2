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
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

use iceoryx2_bb_elementary::{
    math::align_to,
    relocatable_container::RelocatableContainer,
    relocatable_ptr::{PointerTrait, RelocatablePointer},
};
use iceoryx2_bb_log::{fail, fatal_panic};

use crate::shm_allocator::PointerOffset;

#[derive(Debug)]
#[repr(C)]
pub struct RelocatableUsedChunkList {
    data_ptr: RelocatablePointer<UnsafeCell<PointerOffset>>,
    capacity: usize,
    size: UnsafeCell<usize>,
    is_memory_initialized: AtomicBool,
}

impl RelocatableContainer for RelocatableUsedChunkList {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            capacity,
            size: UnsafeCell::new(0),
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
                    std::mem::size_of::<PointerOffset>() * self.capacity,
                    std::mem::align_of::<PointerOffset>())),
            "Failed to initialize since the allocation of the data memory failed.");

        self.data_ptr.init(memory);

        for _ in 0..self.capacity {
            unsafe {
                (self.data_ptr.as_ptr() as *mut UnsafeCell<PointerOffset>)
                    .add(self.size())
                    .write(UnsafeCell::new(PointerOffset::new(0)))
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
            size: UnsafeCell::new(0),
            is_memory_initialized: AtomicBool::new(true),
        }
    }
}

impl RelocatableUsedChunkList {
    pub const fn const_memory_size(capacity: usize) -> usize {
        std::mem::size_of::<PointerOffset>() * capacity + std::mem::align_of::<PointerOffset>() - 1
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn size(&self) -> usize {
        unsafe { *self.size.get() }
    }

    fn verify_init(&self, source: &str) {
        if !self.is_memory_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from self, "Undefined behavior when calling \"{}\" and the object is not initialized.", source);
        }
    }

    pub fn insert(&self, value: PointerOffset) -> bool {
        self.verify_init("insert");

        if self.size() == self.capacity {
            return false;
        }

        unsafe { *(*self.data_ptr.as_ptr().add(self.size())).get() = value };
        unsafe { *self.size.get() += 1 };

        true
    }

    pub fn remove(&self, value: PointerOffset) -> bool {
        for i in 0..self.size() {
            let rhs = unsafe { *(*self.data_ptr.as_ptr().add(i)).get() };
            if rhs == value {
                unsafe { *self.size.get() -= 1 };
                if i + 1 != self.size() {
                    unsafe {
                        *(*self.data_ptr.as_ptr().add(i)).get() =
                            *(*self.data_ptr.as_ptr().add(self.size())).get()
                    };
                }

                return true;
            }
        }

        false
    }

    pub fn pop(&self) -> Option<PointerOffset> {
        self.verify_init("pop");

        if self.size() == 0 {
            return None;
        }

        let value = unsafe { *(*self.data_ptr.as_ptr().add(self.size() - 1)).get() };
        unsafe { *self.size.get() -= 1 };
        Some(value)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeUsedChunkList<const CAPACITY: usize> {
    list: RelocatableUsedChunkList,
    data: [UnsafeCell<PointerOffset>; CAPACITY],
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
            data: core::array::from_fn(|_| UnsafeCell::new(PointerOffset::new(0))),
        }
    }

    pub fn insert(&self, value: PointerOffset) -> bool {
        self.list.insert(value)
    }

    pub fn pop(&mut self) -> Option<PointerOffset> {
        self.list.pop()
    }

    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    pub fn size(&self) -> usize {
        self.list.size()
    }
}
