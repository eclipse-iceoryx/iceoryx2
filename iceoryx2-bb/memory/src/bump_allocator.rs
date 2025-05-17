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

//! A **threadsafe** and **lock-free** bump allocator which implements the [`BaseAllocator`].
//! It can be allocated with [`BumpAllocator::allocate()`] but [`BumpAllocator::deallocate`]
//! deallocate all allocated chunks. See this: `https://os.phil-opp.com/allocator-designs/`
//! for more details.

use core::{fmt::Display, ptr::NonNull, sync::atomic::Ordering};

use iceoryx2_bb_elementary::math::align;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

pub use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};

#[derive(Debug)]
pub struct BumpAllocator {
    pub(crate) start: usize,
    size: usize,
    current_position: IoxAtomicUsize,
}

impl Display for BumpAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BumpAllocator {{ start: {}, size: {}, current_position: {} }}",
            self.start,
            self.size,
            self.current_position
                .load(core::sync::atomic::Ordering::Relaxed)
        )
    }
}

impl BumpAllocator {
    pub fn new(ptr: NonNull<u8>, size: usize) -> Self {
        Self {
            start: ptr.as_ptr() as usize,
            size,
            current_position: IoxAtomicUsize::new(0),
        }
    }

    pub fn start_address(&self) -> usize {
        self.start
    }

    pub fn used_space(&self) -> usize {
        self.current_position.load(Ordering::Relaxed)
    }

    pub fn free_space(&self) -> usize {
        self.size - self.used_space()
    }

    pub fn total_space(&self) -> usize {
        self.size
    }
}

impl BaseAllocator for BumpAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<[u8]>, AllocationError> {
        let msg = "Unable to allocate chunk with";
        let mut aligned_position;

        if layout.size() == 0 {
            fail!(from self, with AllocationError::SizeIsZero,
                "{} {:?} since the requested size was zero.", msg, layout);
        }

        let mut current_position = self
            .current_position
            .load(core::sync::atomic::Ordering::Relaxed);
        loop {
            aligned_position = align(self.start + current_position, layout.align()) - self.start;
            if aligned_position + layout.size() > self.size {
                fail!(from self, with AllocationError::OutOfMemory,
                    "{} {:?} since there is not enough memory available.", msg, layout);
            }

            match self.current_position.compare_exchange_weak(
                current_position,
                aligned_position + layout.size(),
                core::sync::atomic::Ordering::Relaxed,
                core::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => current_position = v,
            }
        }

        Ok(unsafe {
            NonNull::new_unchecked(core::slice::from_raw_parts_mut(
                (self.start + aligned_position) as *mut u8,
                layout.size(),
            ))
        })
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        self.current_position
            .store(0, core::sync::atomic::Ordering::Relaxed);
    }
}
