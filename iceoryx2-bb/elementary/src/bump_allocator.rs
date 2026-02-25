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

use core::ptr::NonNull;
use iceoryx2_bb_concurrency::cell::RefCell;

use crate::math::align;
use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};

/// A minimalistic [`BumpAllocator`].
pub struct BumpAllocator {
    start: NonNull<u8>,
    addr_next_free_memory: RefCell<u64>,
    start_size: usize,
}

impl BumpAllocator {
    /// Creates a new [`BumpAllocator`] that manages the memory starting at `start`.
    pub fn new(start: NonNull<u8>, size: usize) -> Self {
        Self {
            start,
            addr_next_free_memory: RefCell::<u64>::new(start.as_ptr().cast::<u64>() as u64),
            start_size: size,
        }
    }
}

impl BaseAllocator for BumpAllocator {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, AllocationError> {
        let mem = align(
            *self.addr_next_free_memory.borrow() as usize,
            layout.align(),
        ) as u64;
        let allocated_memory_size =
            *self.addr_next_free_memory.borrow() as usize - self.start.as_ptr() as usize;

        if allocated_memory_size + layout.size() > self.start_size {
            return Err(AllocationError::SizeTooLarge);
        }

        self.addr_next_free_memory
            .replace(mem + layout.size() as u64);

        unsafe {
            Ok(core::ptr::NonNull::new_unchecked(
                core::ptr::slice_from_raw_parts_mut(
                    self.start
                        .add(mem as usize - self.start.as_ptr() as usize)
                        .as_ptr(),
                    layout.size(),
                ),
            ))
        }
    }

    unsafe fn deallocate(&self, _ptr: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        self.addr_next_free_memory
            .replace(self.start.as_ptr() as u64);
    }
}
