// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::NonNull,
};

use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_memory::heap_allocator::HeapAllocator;

#[derive(Debug)]
pub struct GlobalHeapAllocator(HeapAllocator);

impl GlobalHeapAllocator {
    pub const fn new() -> Self {
        Self(HeapAllocator::new())
    }
}

unsafe impl GlobalAlloc for GlobalHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.0.allocate(layout) {
            Ok(ptr) => ptr.as_ptr() as *mut u8,
            Err(_) => core::ptr::null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(non_null) = NonNull::new(ptr) {
            self.0.deallocate(non_null, layout);
        }
    }
}

#[global_allocator]
static GLOBAL: GlobalHeapAllocator = GlobalHeapAllocator::new();
