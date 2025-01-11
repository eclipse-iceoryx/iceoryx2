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
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;
use std::sync::atomic::Ordering;

/// A minimalistic [`BumpAllocator`].
pub struct BumpAllocator {
    start: usize,
    pos: IoxAtomicUsize,
}

impl BumpAllocator {
    /// Creates a new [`BumpAllocator`] that manages the memory starting at `start`.
    pub fn new(start: usize) -> Self {
        Self {
            start,
            pos: IoxAtomicUsize::new(start),
        }
    }
}

impl BaseAllocator for BumpAllocator {
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, crate::allocator::AllocationError> {
        let mem = align(self.pos.load(Ordering::Relaxed), layout.align());
        self.pos.store(mem + layout.size(), Ordering::Relaxed);

        unsafe {
            Ok(core::ptr::NonNull::new_unchecked(
                std::slice::from_raw_parts_mut(mem as *mut u8, layout.size()),
            ))
        }
    }

    unsafe fn deallocate(&self, _ptr: core::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        self.pos.store(self.start, Ordering::Relaxed);
    }
}
