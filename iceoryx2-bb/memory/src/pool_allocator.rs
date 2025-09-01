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

//! A **threadsafe**, **lock-free** bucket [`Allocator`] which partitions the provided memory into
//! buckets of equal size with a given alignment.
//! The memory chunks cannot be resized or greater than the maximum bucket size.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_memory::pool_allocator::*;
//!
//! const BUCKET_SIZE: usize = 128;
//! const BUCKET_ALIGNMENT: usize = 8;
//! const MEMORY_SIZE: usize = 1024;
//! const MAX_NUMBER_OF_BUCKETS: usize = 512;
//! let mut memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
//! let mut allocator = FixedSizePoolAllocator::<MAX_NUMBER_OF_BUCKETS>
//!                         ::new(unsafe{ Layout::from_size_align_unchecked(BUCKET_SIZE,
//!                         BUCKET_ALIGNMENT) }, NonNull::new(memory.as_mut_ptr()).unwrap(), MEMORY_SIZE );
//!
//! let mut memory = allocator.allocate(unsafe{Layout::from_size_align_unchecked(48, 4)})
//!                           .expect("failed to allocate");
//!
//! let mut grown_memory = unsafe { allocator.grow_zeroed(
//!                             NonNull::new(memory.as_mut().as_mut_ptr()).unwrap(),
//!                             Layout::from_size_align_unchecked(48, 4),
//!                             Layout::from_size_align_unchecked(64, 4)
//!                         ).expect("failed to grow memory")};
//!
//! let mut shrink_memory = unsafe { allocator.shrink(
//!                             NonNull::new(grown_memory.as_mut().as_mut_ptr()).unwrap(),
//!                             Layout::from_size_align_unchecked(64, 4),
//!                             Layout::from_size_align_unchecked(32, 4)
//!                         ).expect("failed to shrink memory")};
//!
//! unsafe{ allocator.deallocate(NonNull::new(shrink_memory.as_mut().as_mut_ptr()).unwrap(),
//!                              Layout::from_size_align_unchecked(32, 4))};
//! ```

use iceoryx2_bb_elementary::bump_allocator::BumpAllocator;
use iceoryx2_bb_elementary::math::align;
use iceoryx2_bb_elementary_traits::relocatable_container::*;
use iceoryx2_bb_lock_free::mpmc::unique_index_set::*;

pub use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::sync::atomic::Ordering;
pub use iceoryx2_bb_elementary_traits::allocator::*;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::fatal_panic;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

#[derive(Debug)]
pub struct PoolAllocator {
    buckets: UniqueIndexSet,
    bucket_size: usize,
    bucket_alignment: usize,
    start: usize,
    size: usize,
    is_memory_initialized: IoxAtomicBool,
}

impl PoolAllocator {
    fn verify_init(&self, source: &str) {
        debug_assert!(
            self.is_memory_initialized.load(Ordering::Relaxed),
            "From: {self:?}, Undefined behavior when calling \"{source}\" and the object is not initialized."
        );
    }

    pub fn number_of_buckets(&self) -> u32 {
        self.buckets.capacity()
    }

    pub fn bucket_size(&self) -> usize {
        self.bucket_size
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn start_address(&self) -> usize {
        self.start
    }

    pub fn max_alignment(&self) -> usize {
        self.bucket_alignment
    }

    /// Releases an previously allocated bucket of memory.
    ///
    /// # Safety
    ///
    ///  * `ptr` must be allocated previously with [`PoolAllocator::allocate()`] or
    ///    [`PoolAllocator::allocate_zeroed()`]
    ///
    pub unsafe fn deallocate_bucket(&self, ptr: NonNull<u8>) {
        self.verify_init("deallocate");

        self.buckets
            .release_raw_index(self.get_index(ptr), ReleaseMode::Default);
    }

    /// # Safety
    ///
    ///  * `ptr` must point to a piece of memory of length `size`
    ///  * before any other method can be called [`PoolAllocator::init()`] must be called once
    ///
    pub unsafe fn new_uninit(bucket_layout: Layout, ptr: NonNull<u8>, size: usize) -> Self {
        let adjusted_start = align(ptr.as_ptr() as usize, bucket_layout.align());

        PoolAllocator {
            buckets: unsafe {
                UniqueIndexSet::new_uninit(Self::calc_number_of_buckets(bucket_layout, ptr, size))
            },
            bucket_size: bucket_layout.size(),
            bucket_alignment: bucket_layout.align(),
            start: adjusted_start,
            size,
            is_memory_initialized: IoxAtomicBool::new(false),
        }
    }

    /// # Safety
    ///
    ///  * must be called exactly once before any other method can be called
    ///
    pub unsafe fn init<Allocator: BaseAllocator>(
        &mut self,
        allocator: &Allocator,
    ) -> Result<(), AllocationError> {
        if self.is_memory_initialized.load(Ordering::Relaxed) {
            fatal_panic!(
                from self,
                "Memory already initialized. Initializing it twice may lead to undefined behavior."
            );
        }

        fail!(from self, when self.buckets.init(allocator),
                "Unable to initialize pool allocator");

        self.is_memory_initialized.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn memory_size(bucket_layout: Layout, size: usize) -> usize {
        let min_required_buckets = size / bucket_layout.size();

        UniqueIndexSet::memory_size(min_required_buckets)
    }

    fn calc_number_of_buckets(bucket_layout: Layout, ptr: NonNull<u8>, size: usize) -> usize {
        let adjusted_start = align(ptr.as_ptr() as usize, bucket_layout.align());
        let bucket_size = align(bucket_layout.size(), bucket_layout.align());

        (ptr.as_ptr() as usize + size - adjusted_start) / bucket_size
    }

    fn verify_ptr_is_managaed_by_allocator(&self, ptr: NonNull<u8>) {
        let position = ptr.as_ptr() as usize;
        debug_assert!(
            !(position < self.start
                || position > self.start + self.size
                || (position - self.start) % self.bucket_size != 0),
            "The pointer {ptr:?} is not managed by this allocator."
        );
    }

    fn get_index(&self, ptr: NonNull<u8>) -> u32 {
        self.verify_ptr_is_managaed_by_allocator(ptr);
        let position = ptr.as_ptr() as usize;

        ((position - self.start) / self.bucket_size) as u32
    }
}

impl BaseAllocator for PoolAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocationError> {
        self.verify_init("allocate");

        if layout.size() > self.bucket_size {
            fail!(from self, with AllocationError::SizeTooLarge,
                "The requested allocation size {} is greater than the maximum supported size of {}.", layout.size(), self.bucket_size);
        }

        if layout.align() > self.bucket_alignment {
            fail!(from self, with AllocationError::AlignmentFailure,
                "The requested allocation alignment {} is greater than the maximum supported alignment of {}.", layout.align(), self.bucket_alignment);
        }

        match unsafe { self.buckets.acquire_raw_index() } {
            Ok(v) => Ok(unsafe {
                NonNull::new_unchecked(core::slice::from_raw_parts_mut(
                    (self.start + v as usize * self.bucket_size) as *mut u8,
                    layout.size(),
                ))
            }),
            Err(_) => {
                fail!(from self, with AllocationError::OutOfMemory,
                    "No more buckets available to allocate {} bytes with an alignment of {}.",
                        layout.size(), layout.align());
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: Layout) {
        self.deallocate_bucket(ptr);
    }
}

impl Allocator for PoolAllocator {
    /// always returns the input ptr on success but with an increased size
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationGrowError> {
        self.verify_init("grow");

        let msg = "Unable to grow memory chunk";
        self.verify_ptr_is_managaed_by_allocator(ptr);

        if old_layout.size() >= new_layout.size() {
            fail!(from self, with AllocationGrowError::GrowWouldShrink,
                "{} since the new size of {} would be smaller than the old size of {}. Use Allocator::shrink instead.", msg, new_layout.size(), old_layout.size());
        }

        if self.bucket_alignment < new_layout.align() {
            fail!(from self, with AllocationGrowError::AlignmentFailure,
                "{} since the new alignment {} exceeds the maximum supported alignment.", msg, new_layout.align() );
        }

        if self.bucket_size < new_layout.size() {
            fail!(from self, with AllocationGrowError::OutOfMemory,
                "{} since the new size {} exceeds the maximum supported size.", msg, new_layout.size());
        }

        Ok(NonNull::new(core::slice::from_raw_parts_mut(
            ptr.as_ptr(),
            new_layout.size(),
        ))
        .unwrap())
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationShrinkError> {
        self.verify_init("shrink");

        let msg = "Unable to shrink memory chunk";
        self.verify_ptr_is_managaed_by_allocator(ptr);

        if old_layout.size() <= new_layout.size() {
            fail!(from self, with AllocationShrinkError::ShrinkWouldGrow,
                "{} since the new size of {} would be greater than the old size of {}. Use Allocator::grow instead.", msg, new_layout.size(), old_layout.size());
        }

        if self.bucket_alignment < new_layout.align() {
            fail!(from self, with AllocationShrinkError::AlignmentFailure,
                "{} since the new alignment {} exceeds the maximum supported alignment.", msg, new_layout.align() );
        }

        Ok(NonNull::new(core::slice::from_raw_parts_mut(
            ptr.as_ptr(),
            new_layout.size(),
        ))
        .unwrap())
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct FixedSizePoolAllocator<const MAX_NUMBER_OF_BUCKETS: usize> {
    state: PoolAllocator,
    next_free_index: [UnsafeCell<u32>; MAX_NUMBER_OF_BUCKETS],
    next_free_index_plus_one: UnsafeCell<u32>,
}

unsafe impl<const MAX_NUMBER_OF_BUCKETS: usize> Send
    for FixedSizePoolAllocator<MAX_NUMBER_OF_BUCKETS>
{
}

unsafe impl<const MAX_NUMBER_OF_BUCKETS: usize> Sync
    for FixedSizePoolAllocator<MAX_NUMBER_OF_BUCKETS>
{
}

impl<const MAX_NUMBER_OF_BUCKETS: usize> FixedSizePoolAllocator<MAX_NUMBER_OF_BUCKETS> {
    pub fn number_of_buckets(&self) -> u32 {
        self.state.number_of_buckets()
    }

    pub fn bucket_size(&self) -> usize {
        self.state.bucket_size()
    }

    pub fn size(&self) -> usize {
        self.state.size()
    }

    pub fn max_alignment(&self) -> usize {
        self.state.max_alignment()
    }

    pub fn new(bucket_layout: Layout, ptr: NonNull<u8>, size: usize) -> Self {
        let adjusted_start = align(ptr.as_ptr() as usize, bucket_layout.align());
        let bucket_size = align(bucket_layout.size(), bucket_layout.align());
        let number_of_buckets = (ptr.as_ptr() as usize + size - adjusted_start) / bucket_size;

        let mut new_self = FixedSizePoolAllocator {
            state: PoolAllocator {
                buckets: unsafe {
                    UniqueIndexSet::new_uninit(core::cmp::min(
                        number_of_buckets,
                        MAX_NUMBER_OF_BUCKETS,
                    ))
                },
                bucket_size: bucket_layout.size(),
                bucket_alignment: bucket_layout.align(),
                start: adjusted_start,
                size,
                is_memory_initialized: IoxAtomicBool::new(true),
            },
            next_free_index: core::array::from_fn(|i| UnsafeCell::new(i as u32 + 1)),
            next_free_index_plus_one: UnsafeCell::new(MAX_NUMBER_OF_BUCKETS as u32 + 1),
        };

        let allocator = BumpAllocator::new(new_self.next_free_index.as_mut_ptr().cast());
        unsafe {
            new_self
                .state
                .buckets
                .init(&allocator)
                .expect("All required memory is preallocated.")
        };
        new_self
    }
}

impl<const MAX_NUMBER_OF_BUCKETS: usize> BaseAllocator
    for FixedSizePoolAllocator<MAX_NUMBER_OF_BUCKETS>
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocationError> {
        self.state.allocate(layout)
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.state.deallocate(ptr, layout);
    }
}

impl<const MAX_NUMBER_OF_BUCKETS: usize> Allocator
    for FixedSizePoolAllocator<MAX_NUMBER_OF_BUCKETS>
{
    /// always returns the input ptr on success but with an increased size
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationGrowError> {
        self.state.grow(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocationShrinkError> {
        self.state.shrink(ptr, old_layout, new_layout)
    }
}
