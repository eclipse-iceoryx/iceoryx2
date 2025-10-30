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

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::{null_mut, NonNull};

use iceoryx2_bb_memory::bump_allocator::BaseAllocator;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
}

pub struct IceoryxBumpAllocator {
    inner: UnsafeCell<Option<BumpAllocator>>,
}

unsafe impl Sync for IceoryxBumpAllocator {}

impl IceoryxBumpAllocator {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub unsafe fn init(&self) {
        let start = &_heap_start as *const u8 as usize;
        let end = &_heap_end as *const u8 as usize;
        let size = end - start;

        let ptr = NonNull::new_unchecked(start as *mut u8);
        let allocator = BumpAllocator::new(ptr, size);

        *self.inner.get() = Some(allocator);
    }
}

unsafe impl GlobalAlloc for IceoryxBumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let inner = &*self.inner.get();

        if let Some(allocator) = inner {
            match allocator.allocate(layout) {
                Ok(ptr) => ptr.as_ptr() as *mut u8,
                Err(_) => null_mut(),
            }
        } else {
            null_mut()
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[global_allocator]
static GLOBAL_ALLOCATOR: IceoryxBumpAllocator = IceoryxBumpAllocator::new();

pub fn initialize() {
    unsafe {
        GLOBAL_ALLOCATOR.init();
    }
}

#[allow(dead_code)]
pub fn heap_info() -> (usize, usize) {
    unsafe {
        let start = &_heap_start as *const u8 as usize;
        let end = &_heap_end as *const u8 as usize;
        (start, end - start)
    }
}
