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

extern crate alloc;

use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::ptr::NonNull;

use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator, Dealloc};

pub struct Allocator {}

impl Default for Allocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator {
    pub fn new() -> Self {
        Self {}
    }
}

impl BaseAllocator<NonNull<u8>> for Allocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<u8>, AllocationError> {
        let ptr = unsafe { alloc(layout) };
        NonNull::new(ptr).ok_or(AllocationError::OutOfMemory)
    }
}

impl Dealloc<NonNull<u8>> for Allocator {
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { dealloc(ptr.as_ptr(), layout) };
    }
}
