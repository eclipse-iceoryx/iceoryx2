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

//! A **threadsafe** and **lock-free** [`Allocator`] which acquires the memory from the heap.

use core::{alloc::Layout, ptr::NonNull};

use iceoryx2_bb_elementary_traits::allocator::{AllocationGrowError, AllocationShrinkError};
use iceoryx2_bb_log::fail;
use iceoryx2_bb_posix::memory::heap;

pub use iceoryx2_bb_elementary_traits::allocator::{AllocationError, Allocator, BaseAllocator};

#[derive(Debug)]
pub struct HeapAllocator {}

impl Default for HeapAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl HeapAllocator {
    pub const fn new() -> HeapAllocator {
        HeapAllocator {}
    }
}

impl BaseAllocator for HeapAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocationError> {
        Ok(fail!(from self, when heap::allocate(layout),
                "Failed to allocate {} bytes with an alignment of {}.", layout.size(), layout.align()))
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        heap::deallocate(ptr, layout);
    }
}

impl Allocator for HeapAllocator {
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationGrowError> {
        if old_layout.size() >= new_layout.size() {
            fail!(from self, with AllocationGrowError::GrowWouldShrink,
                "Failed to grow memory from (size: {}, align: {}) to (size: {}, align: {}).", old_layout.size(),old_layout.align(), new_layout.size(), new_layout.align());
        }
        Ok(
            fail!(from self, when heap::resize(ptr, old_layout, new_layout),
                "Failed to grow memory from (size: {}, align: {}) to (size: {}, align: {}).", old_layout.size(),old_layout.align(), new_layout.size(), new_layout.align()),
        )
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationShrinkError> {
        if old_layout.size() <= new_layout.size() {
            fail!(from self, with AllocationShrinkError::ShrinkWouldGrow,
                "Failed to shrink memory from (size: {}, align: {}) to (size: {}, align: {}).", old_layout.size(),old_layout.align(), new_layout.size(), new_layout.align());
        }
        Ok(
            fail!(from self, when heap::resize(ptr, old_layout, new_layout),
                "Failed to shrink memory from (size: {}, align: {}) to (size: {}, align: {}).", old_layout.size(),old_layout.align(), new_layout.size(), new_layout.align()),
        )
    }
}
