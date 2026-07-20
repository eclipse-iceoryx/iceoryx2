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

//! A **threadsafe** and **lock-free** bump allocator.
//! It can be allocated with `BumpAllocator::allocate()` but `BumpAllocator::deallocate`
//! deallocate all allocated chunks. See this: `https://os.phil-opp.com/allocator-designs/`
//! for more details.
//!
//! ```
//! use core::alloc::Layout;
//! use core::ptr::NonNull;
//!
//! use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
//! use crate::iceoryx2_bb_elementary::bump_allocator::BaseAllocator;
//! extern crate iceoryx2_bb_loggers;
//!
//! let mut memory = [0u8; 8192];
//! const MEM_SIZE: usize = 128;
//! const MEM_ALIGN: usize = 64;
//! let layout = Layout::from_size_align(MEM_SIZE, MEM_ALIGN).unwrap();
//!
//! let allocator = BumpAllocator::new(
//!     NonNull::<u8>::from_ref(&memory[0]),
//!     memory.len(),
//! );
//!
//! let mut memory = allocator.allocate(layout).unwrap();
//!
//! unsafe {
//!     allocator.deallocate(
//!         NonNull::new(memory.as_mut().as_mut_ptr().cast()).unwrap(),
//!         layout,
//!     )
//! };
//! ```

use core::{fmt::Display, ptr::NonNull};

use crate::math::align;
use crate::sync_ptr::SyncPtr;
use iceoryx2_bb_concurrency::atomic::AtomicUsize;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_elementary_traits::generic_pointer::NonNullFamily;
use iceoryx2_bb_elementary_traits::pointer::Pointer;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;

pub use iceoryx2_bb_elementary_traits::allocator::{AllocationError, BaseAllocator};

#[derive(Debug)]
#[repr(C)]
pub struct BumpAllocator {
    pub(crate) start: SyncPtr<u8>,
    addr_next_free_memory: AtomicUsize,
    full_memory_size: usize,
}

unsafe impl ZeroCopySend for BumpAllocator {}

impl Display for BumpAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "BumpAllocator {{ start: {:?}, current_position: {}, size: {} }}",
            self.start,
            self.addr_next_free_memory
                .load(core::sync::atomic::Ordering::Relaxed),
            self.full_memory_size,
        )
    }
}

impl BumpAllocator {
    pub fn new(start: NonNull<u8>, full_memory_size: usize) -> Self {
        Self {
            start: SyncPtr::new(start.as_ptr()),
            addr_next_free_memory: AtomicUsize::new(0),
            full_memory_size,
        }
    }

    pub fn start_address(&self) -> *const u8 {
        self.start.as_ptr()
    }

    pub fn used_space(&self) -> usize {
        self.addr_next_free_memory.load(Ordering::Relaxed)
    }

    pub fn free_space(&self) -> usize {
        self.full_memory_size - self.used_space()
    }

    pub fn total_space(&self) -> usize {
        self.full_memory_size
    }
}

impl BaseAllocator<NonNullFamily> for BumpAllocator {
    fn allocate(&self, layout: core::alloc::Layout) -> Result<NonNull<u8>, AllocationError> {
        let msg = "Unable to allocate chunk with";
        let mut next_aligned_free_address;

        if layout.size() == 0 {
            fail!(from self, with AllocationError::SizeIsZero,
                "{} {:?} since the requested size was zero.", msg, layout);
        }
        let start = self.start.as_ptr() as usize;
        let mut current_addr_next_free_memory = self
            .addr_next_free_memory
            .load(core::sync::atomic::Ordering::Relaxed);

        loop {
            next_aligned_free_address =
                align(start + current_addr_next_free_memory, layout.align()) - start;
            if next_aligned_free_address + layout.size() > self.full_memory_size {
                fail!(from self, with AllocationError::OutOfMemory,
                    "{} {:?} since there is not enough memory available.", msg, layout);
            }

            match self.addr_next_free_memory.compare_exchange_weak(
                current_addr_next_free_memory,
                next_aligned_free_address + layout.size(),
                core::sync::atomic::Ordering::Relaxed,
                core::sync::atomic::Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => current_addr_next_free_memory = v,
            }
        }

        Ok(unsafe {
            NonNull::new_unchecked(
                self.start
                    .as_ptr()
                    .cast_mut()
                    .add(next_aligned_free_address),
            )
        })
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: core::alloc::Layout) {
        self.addr_next_free_memory
            .store(0, core::sync::atomic::Ordering::Relaxed);
    }
}
