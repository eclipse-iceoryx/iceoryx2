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

use crate::{allocator::BaseAllocator, math::align};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Simple BumpAllocator for testing purposes. Do not use this in production. If you are looking
/// for a production ready BumpAllocator use the one from iceoryx2_bb_memory::bump_allocator
#[doc(hidden)]
pub struct BumpAllocator {
    start: usize,
    pos: AtomicUsize,
}

impl BumpAllocator {
    pub fn new(start: usize) -> Self {
        Self {
            start,
            pos: AtomicUsize::new(start),
        }
    }
}

impl BaseAllocator for BumpAllocator {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, crate::allocator::AllocationError> {
        let mem = align(self.pos.load(Ordering::Relaxed), layout.align());
        self.pos.store(mem + layout.size(), Ordering::Relaxed);

        unsafe {
            Ok(std::ptr::NonNull::new_unchecked(
                std::slice::from_raw_parts_mut(mem as *mut u8, layout.size()),
            ))
        }
    }

    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        self.pos.store(self.start, Ordering::Relaxed);
    }
}
