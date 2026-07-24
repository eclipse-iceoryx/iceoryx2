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

use core::{alloc::Layout, ptr::NonNull};

use crate::shm_allocator::{ContentPlacement, InitializedShmAllocator};
use crate::shm_allocator::{ShmAllocator, ShmAllocatorConfig};

use iceoryx2_bb_concurrency::atomic::AtomicUsize;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::allocation_strategy::AllocationStrategy;
use iceoryx2_bb_elementary_traits::allocator::{AllocationGrowError, BaseAllocator};
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_memory::bump_allocator::AllocationError;
use iceoryx2_bb_memory::pool_allocator::{Dealloc, ReallocGrow};
use iceoryx2_log::fail;

use super::{PointerOffset, SharedMemorySetupHint, ShmAllocatorInitError};

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub bucket_layout: Layout,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bucket_layout: unsafe { Layout::from_size_align_unchecked(1024, 8) },
        }
    }
}

impl ShmAllocatorConfig for Config {}

#[derive(Debug, ZeroCopySend)]
#[repr(C)]
pub struct PoolAllocator {
    allocator: iceoryx2_bb_memory::pool_allocator::PoolAllocator,
    // is even with absolut base address relocatable since every process acquire and return
    // the same relative offset which map then to the same absolut base address
    // the allocator only manages a range of numbers
    base_address: usize,
    max_supported_alignment_by_memory: usize,
    number_of_used_buckets: AtomicUsize,
}

impl PoolAllocator {
    pub fn bucket_size(&self) -> usize {
        self.allocator.bucket_size()
    }

    pub fn number_of_buckets(&self) -> u32 {
        self.allocator.number_of_buckets()
    }

    /// # Safety
    ///
    ///  * provided [`PointerOffset`] must be allocated with [`BaseAllocator::allocate()`]
    pub unsafe fn deallocate_bucket(&self, offset: PointerOffset) {
        self.number_of_used_buckets.fetch_sub(1, Ordering::Relaxed);
        unsafe {
            self.allocator.deallocate_bucket(NonNull::new_unchecked(
                (offset.offset() + self.allocator.start_address() as usize) as *mut u8,
            ));
        }
    }
}

pub struct InitializedPoolAllocator<'shm_allocator>(&'shm_allocator PoolAllocator);

impl<'shm_allocator> BaseAllocator<PointerOffset> for InitializedPoolAllocator<'shm_allocator> {
    fn allocate(&self, layout: Layout) -> Result<PointerOffset, AllocationError> {
        let msg = "Unable to allocate memory";
        if layout.align() > self.0.max_alignment() {
            fail!(from self.0, with AllocationError::AlignmentFailure,
                    "{} since an alignment of {} exceeds the maximum supported alignment of {}.",
                    msg, layout.align(), self.0.max_alignment());
        }

        let chunk = fail!(from self.0, when self.0.allocator.allocate(layout), "{}.", msg);
        self.0
            .number_of_used_buckets
            .fetch_add(1, Ordering::Relaxed);
        Ok(PointerOffset::new(
            (chunk.as_ptr() as *const u8) as usize - self.0.allocator.start_address() as usize,
        ))
    }
}

impl<'shm_allocator> Dealloc<PointerOffset> for InitializedPoolAllocator<'shm_allocator> {
    unsafe fn deallocate(&self, ptr: PointerOffset, _layout: Layout) {
        unsafe {
            self.0.deallocate_bucket(ptr);
        }
    }
}

impl<'shm_allocator> ReallocGrow<PointerOffset> for InitializedPoolAllocator<'shm_allocator> {
    unsafe fn grow(
        &self,
        offset: PointerOffset,
        old_layout: Layout,
        new_layout: Layout,
        content_placement: ContentPlacement,
    ) -> Result<PointerOffset, AllocationGrowError> {
        let msg = "Unable to grow memory";
        if new_layout.align() > self.0.max_alignment() {
            fail!(from self.0, with AllocationGrowError::AlignmentFailure,
                    "{} since the alignment of {} exceeds the maximum supported alignment of {}.",
                    msg, new_layout.align(), self.0.max_alignment());
        }

        if new_layout.size() < old_layout.size() {
            fail!(from self.0, with AllocationGrowError::GrowWouldShrink,
                    "{} since new layout has a smaller size of {} than the old layout with {}.",
                    msg, new_layout.size(), old_layout.size());
        }

        if new_layout.size() > self.0.bucket_size() {
            fail!(from self.0,
                        with AllocationGrowError::OutOfMemory,
                        "{} since the requested size {} exceeds the maximum supported size of {}.",
                        msg, new_layout.size(), self.0.bucket_size());
        }

        if new_layout.size() == old_layout.size() {
            return Ok(offset);
        }

        if content_placement == ContentPlacement::Back {
            let src = self.0.allocator.start_address() as usize + offset.offset();
            let dst = src + (new_layout.size() - old_layout.size());
            unsafe { core::ptr::copy(src as *const u8, dst as *mut u8, old_layout.size()) };
        }

        Ok(offset)
    }
}

impl<'shm_allocator> InitializedShmAllocator<'shm_allocator>
    for InitializedPoolAllocator<'shm_allocator>
{
}

impl ShmAllocator for PoolAllocator {
    type Configuration = Config;
    type Initialized<'shm_allocator> = InitializedPoolAllocator<'shm_allocator>;

    unsafe fn assume_init<'shm_allocator>(
        &'shm_allocator self,
    ) -> Self::Initialized<'shm_allocator> {
        InitializedPoolAllocator(self)
    }

    fn resize_hint(
        &self,
        layout: Layout,
        strategy: AllocationStrategy,
    ) -> SharedMemorySetupHint<Self::Configuration> {
        let current_layout = unsafe {
            Layout::from_size_align_unchecked(
                self.allocator.bucket_size(),
                self.allocator.max_alignment(),
            )
        };

        let adjusted_number_of_buckets = if self.number_of_used_buckets.load(Ordering::Relaxed)
            == self.number_of_buckets() as usize
        {
            match strategy {
                AllocationStrategy::BestFit => self.allocator.number_of_buckets() + 1,
                AllocationStrategy::PowerOfTwo => {
                    (self.allocator.number_of_buckets() + 1).next_power_of_two()
                }
                AllocationStrategy::Static => self.allocator.number_of_buckets(),
            }
        } else {
            self.number_of_buckets()
        };

        let adjusted_layout =
            if current_layout.size() < layout.size() || current_layout.align() < layout.align() {
                match strategy {
                    AllocationStrategy::Static => current_layout,
                    AllocationStrategy::BestFit => unsafe {
                        let align = layout.align().max(current_layout.align());
                        let size = layout
                            .size()
                            .max(current_layout.size())
                            .next_multiple_of(align);
                        Layout::from_size_align_unchecked(size, align)
                    },
                    AllocationStrategy::PowerOfTwo => unsafe {
                        let align = layout
                            .align()
                            .max(current_layout.align())
                            .next_power_of_two();
                        let size = layout
                            .size()
                            .max(current_layout.size())
                            .next_power_of_two()
                            .next_multiple_of(align);
                        Layout::from_size_align_unchecked(size, align)
                    },
                }
            } else {
                current_layout
            };

        Self::initial_setup_hint(adjusted_layout, adjusted_number_of_buckets as usize)
    }

    fn initial_setup_hint(
        max_chunk_layout: Layout,
        max_number_of_chunks: usize,
    ) -> SharedMemorySetupHint<Self::Configuration> {
        SharedMemorySetupHint {
            payload_size: max_chunk_layout.size() * max_number_of_chunks,
            config: Self::Configuration {
                bucket_layout: max_chunk_layout,
            },
        }
    }

    fn management_size(memory_size: usize, config: &Self::Configuration) -> usize {
        iceoryx2_bb_memory::pool_allocator::PoolAllocator::memory_size(
            config.bucket_layout,
            memory_size,
        )
    }

    fn relative_start_address(&self) -> usize {
        self.allocator.start_address() as usize - self.base_address
    }

    unsafe fn new_uninit(
        max_supported_alignment_by_memory: usize,
        managed_memory: NonNull<[u8]>,
        config: &Self::Configuration,
    ) -> Self {
        Self {
            allocator: unsafe {
                iceoryx2_bb_memory::pool_allocator::PoolAllocator::new_uninit(
                    config.bucket_layout,
                    NonNull::new_unchecked(managed_memory.as_ptr() as *mut u8),
                    managed_memory.len(),
                )
            },
            base_address: (managed_memory.as_ptr() as *mut u8) as usize,
            max_supported_alignment_by_memory,
            number_of_used_buckets: AtomicUsize::new(0),
        }
    }

    fn max_alignment(&self) -> usize {
        self.allocator.max_alignment()
    }

    unsafe fn init<Allocator: BaseAllocator<NonNull<u8>>>(
        &mut self,
        mgmt_allocator: &Allocator,
    ) -> Result<(), ShmAllocatorInitError> {
        let msg = "Unable to initialize allocator";
        if self.max_supported_alignment_by_memory < self.max_alignment() {
            fail!(from self, with ShmAllocatorInitError::MaxSupportedMemoryAlignmentInsufficient,
                "{} since the required alignment {} exceeds the maximum supported alignment {} of the memory.",
                msg, self.max_alignment(), self.max_supported_alignment_by_memory);
        }

        fail!(from self, when unsafe { self.allocator.init(mgmt_allocator)  },
            with ShmAllocatorInitError::AllocationFailed,
            "{} since the allocation of the allocator managment memory failed.", msg);

        Ok(())
    }

    fn unique_id() -> u8 {
        0
    }
}
