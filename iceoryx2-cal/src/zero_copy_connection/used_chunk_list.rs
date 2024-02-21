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
    relocatable_container::RelocatableContainer,
    relocatable_ptr::{PointerTrait, RelocatablePointer},
};
use iceoryx2_bb_log::{fail, fatal_panic};

use crate::shm_allocator::PointerOffset;

#[derive(Debug)]
#[repr(C)]
pub struct RelocatableUsedChunkList {
    data_ptr: RelocatablePointer<PointerOffset>,
    capacity: usize,
    size: usize,
    is_memory_initialized: AtomicBool,
}

impl RelocatableContainer for RelocatableUsedChunkList {
    unsafe fn new_uninit(capacity: usize) -> Self {
        Self {
            data_ptr: RelocatablePointer::new_uninit(),
            capacity,
            size: 0,
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
            size: 0,
            is_memory_initialized: AtomicBool::new(true),
        }
    }
}

impl RelocatableUsedChunkList {
    pub fn const_memory_size(capacity: usize) -> usize {
        std::mem::size_of::<PointerOffset>() * capacity + std::mem::align_of::<PointerOffset>() - 1
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn size(&self) -> usize {
        self.size
    }

    fn verify_init(&self, source: &str) {
        if !self.is_memory_initialized.load(Ordering::Relaxed) {
            fatal_panic!(from self, "Undefined behavior when calling \"{}\" and the object is not initialized.", source);
        }
    }

    pub fn insert(&mut self, value: PointerOffset) -> bool {
        self.verify_init("insert");

        if self.size == self.capacity {
            return false;
        }

        unsafe { self.data_ptr.as_mut_ptr().add(self.size).write(value) };
        self.size += 1;

        true
    }

    pub fn remove(&mut self) -> Option<PointerOffset> {
        self.verify_init("remove");

        if self.size == 0 {
            return None;
        }

        let value = unsafe { *self.data_ptr.as_mut_ptr().add(self.size - 1) };
        self.size -= 1;
        Some(value)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FixedSizeUsedChunkList<const CAPACITY: usize> {
    list: RelocatableUsedChunkList,
    data: [PointerOffset; CAPACITY],
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
            data: [PointerOffset::new(0); CAPACITY],
        }
    }

    pub fn insert(&mut self, value: PointerOffset) -> bool {
        self.list.insert(value)
    }

    pub fn remove(&mut self) -> Option<PointerOffset> {
        self.list.remove()
    }

    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    pub fn size(&self) -> usize {
        self.list.size()
    }
}
