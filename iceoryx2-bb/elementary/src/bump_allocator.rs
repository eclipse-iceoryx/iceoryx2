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

use crate::math::align;
use iceoryx2_bb_concurrency::atomic::AtomicUsize;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};

/// A minimalistic [`BumpAllocator`].
pub struct BumpAllocator {
    start: *mut u8,
    pos: AtomicUsize,
}

impl BumpAllocator {
    /// Creates a new [`BumpAllocator`] that manages the memory starting at `start`.
    pub fn new(start: *mut u8) -> Self {
        Self {
            start,
            pos: AtomicUsize::new(start as usize),
        }
    }
}

impl BaseAllocator for BumpAllocator {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, AllocationError> {
        let mem = align(self.pos.load(Ordering::Relaxed), layout.align());
        self.pos.store(mem + layout.size(), Ordering::Relaxed);

        unsafe {
            Ok(core::ptr::NonNull::new_unchecked(
                core::ptr::slice_from_raw_parts_mut(
                    self.start.add(mem - self.start as usize),
                    layout.size(),
                ),
            ))
        }
    }

    unsafe fn deallocate(&self, _ptr: core::ptr::NonNull<u8>, _layout: core::alloc::Layout) {
        self.pos.store(self.start as usize, Ordering::Relaxed);
    }
}
