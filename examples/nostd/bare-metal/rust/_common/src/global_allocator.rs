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

#![allow(clippy::missing_safety_doc)]

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{NonNull, from_ref, null_mut};

use iceoryx2_bb_concurrency::cell::UnsafeCell;
use iceoryx2_bb_elementary_traits::non_null::NonNullCompat;
use iceoryx2_bb_memory::bump_allocator::BaseAllocator;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;

unsafe extern "C" {
    static _heap_start: u8;
    static _heap_end: u8;
}

pub struct IceoryxBumpAllocator {
    inner: UnsafeCell<Option<BumpAllocator>>,
}

unsafe impl Sync for IceoryxBumpAllocator {}

impl Default for IceoryxBumpAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl IceoryxBumpAllocator {
    pub const fn new() -> Self {
        Self {
            inner: UnsafeCell::new(None),
        }
    }

    pub unsafe fn init(&self) {
        let start = unsafe { &_heap_start };
        let end = unsafe { &_heap_end };
        let size = from_ref(end).addr() - from_ref(start).addr();
        let ptr = NonNull::iox2_from_ref(start);
        let allocator = BumpAllocator::new(ptr, size);

        unsafe { *self.inner.get() = Some(allocator) };
    }
}

unsafe impl GlobalAlloc for IceoryxBumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let inner = unsafe { &*self.inner.get() };

        if let Some(allocator) = inner {
            match allocator.allocate(layout) {
                Ok(ptr) => ptr.as_ptr().cast(),
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
