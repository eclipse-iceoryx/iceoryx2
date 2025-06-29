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

//! A **non-threadsafe** [`Allocator`] which manages only on chunk. When allocating memory always the
//! maximum amount of available aligned memory is provided.
//!
//! # Example
//! ```
//! use iceoryx2_bb_memory::one_chunk_allocator::*;
//!
//! const MEMORY_SIZE: usize = 1024;
//! let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
//! let mut allocator = OneChunkAllocator::new(NonNull::new(memory.as_mut_ptr()).unwrap(),
//!                                             MEMORY_SIZE);
//!
//! // always returns a slice with the maximum available size
//! let mut memory = allocator.allocate(unsafe{Layout::from_size_align_unchecked(48, 4)})
//!                           .expect("failed to allocate");
//!
//! // will always return the same pointer but shrink the underlying memory
//! let mut shrink_memory = unsafe { allocator.shrink(
//!                             NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
//!                             Layout::from_size_align_unchecked(64, 4),
//!                             Layout::from_size_align_unchecked(32, 4)
//!                         ).expect("failed to shrink memory")};
//!
//! // will always return the same pointer but grow the underlying memory
//! let mut grown_memory = unsafe { allocator.grow_zeroed(
//!                             NonNull::new(shrink_memory.as_mut().as_mut_ptr()).unwrap(),
//!                             Layout::from_size_align_unchecked(48, 4),
//!                             Layout::from_size_align_unchecked(64, 4)
//!                         ).expect("failed to grow memory")};
//!
//! unsafe{ allocator.deallocate(NonNull::new(grown_memory.as_mut().as_mut_ptr()).unwrap(),
//!                              Layout::from_size_align_unchecked(32, 4))};
//! ```
use core::sync::atomic::Ordering;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

pub use core::alloc::Layout;
use iceoryx2_bb_elementary::math::align;
pub use iceoryx2_bb_elementary_traits::allocator::*;

#[derive(Debug)]
pub struct OneChunkAllocator {
    start: usize,
    size: usize,
    allocated_chunk_start: IoxAtomicUsize,
}

impl OneChunkAllocator {
    pub fn new(ptr: NonNull<u8>, size: usize) -> OneChunkAllocator {
        OneChunkAllocator {
            start: ptr.as_ptr() as usize,
            size,
            allocated_chunk_start: IoxAtomicUsize::new(0),
        }
    }

    pub fn has_chunk_available(&self) -> bool {
        self.allocated_chunk_start.load(Ordering::Relaxed) == 0
    }

    fn verify_ptr_is_managed_by_allocator(&self, ptr: NonNull<u8>) {
        debug_assert!(
            ptr.as_ptr() as usize == self.allocated_chunk_start.load(Ordering::Relaxed),
            "Tried to access memory ({ptr:?}) that does not belong to this allocator."
        );
    }

    fn release_chunk(&self) {
        self.allocated_chunk_start.store(0, Ordering::Relaxed)
    }

    pub fn start_address(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl BaseAllocator for OneChunkAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocationError> {
        let adjusted_start = align(self.start, layout.align());
        let msg = "Unable to allocate chunk";

        if !self.has_chunk_available() {
            fail!(from self, with AllocationError::OutOfMemory,
                "{} since there is no more chunk available.", msg);
        }

        let available_size = self.size - (adjusted_start - self.start);
        if available_size <= layout.size() {
            fail!(from self, with AllocationError::OutOfMemory,
                "{} due to insufficient available memory.", msg);
        }

        self.allocated_chunk_start
            .store(adjusted_start, Ordering::Relaxed);
        Ok(NonNull::new(unsafe {
            core::slice::from_raw_parts_mut(adjusted_start as *mut u8, available_size)
        })
        .unwrap())
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        self.verify_ptr_is_managed_by_allocator(ptr);
        self.release_chunk();
    }
}

impl Allocator for OneChunkAllocator {
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationGrowError> {
        let msg = "Unable to grow memory chunk";
        self.verify_ptr_is_managed_by_allocator(ptr);

        if old_layout.size() >= new_layout.size() {
            fail!(from self, with AllocationGrowError::GrowWouldShrink,
                "{} since the new size {} is smaller than the old size {}.", msg, new_layout.size(), old_layout.size());
        }

        if old_layout.align() < new_layout.align() {
            fail!(from self, with AllocationGrowError::AlignmentFailure,
                "{} since this allocator does not support to any alignment increase in this operation.", msg);
        }

        let available_size =
            self.size - (self.allocated_chunk_start.load(Ordering::Relaxed) - self.start);

        if available_size < new_layout.size() {
            fail!(from self, with AllocationGrowError::OutOfMemory,
                "{} since the size of {} exceeds the available memory size of {}.", msg, new_layout.size(), available_size);
        }

        Ok(NonNull::new(core::slice::from_raw_parts_mut(
            ptr.as_ptr(),
            available_size,
        ))
        .unwrap())
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationShrinkError> {
        let msg = "Unable to shrink memory chunk";
        self.verify_ptr_is_managed_by_allocator(ptr);

        if old_layout.size() <= new_layout.size() {
            fail!(from self, with AllocationShrinkError::ShrinkWouldGrow,
                "{} since the new size {} is greater than the old size {}.", msg, new_layout.size(), old_layout.size());
        }

        if old_layout.align() < new_layout.align() {
            fail!(from self, with AllocationShrinkError::AlignmentFailure,
                "{} since this allocator does not support to any alignment increase in this operation.", msg);
        }

        Ok(NonNull::new(core::slice::from_raw_parts_mut(
            ptr.as_ptr(),
            new_layout.size(),
        ))
        .unwrap())
    }
}
