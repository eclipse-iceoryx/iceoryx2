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

use core::{alloc::Layout, ptr::NonNull, sync::atomic::Ordering};

use crate::shm_allocator::{ShmAllocator, ShmAllocatorConfig};
use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

use super::{
    AllocationStrategy, PointerOffset, SharedMemorySetupHint, ShmAllocationError,
    ShmAllocatorInitError,
};

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

#[derive(Debug)]
pub struct PoolAllocator {
    allocator: iceoryx2_bb_memory::pool_allocator::PoolAllocator,
    // is even with absolut base address relocatable since every process acquire and return
    // the same relative offset which map then to the same absolut base address
    // the allocator only manages a range of numbers
    base_address: usize,
    max_supported_alignment_by_memory: usize,
    number_of_used_buckets: IoxAtomicUsize,
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
    ///  * provided [`PointerOffset`] must be allocated with [`PoolAllocator::allocate()`]
    pub unsafe fn deallocate_bucket(&self, offset: PointerOffset) {
        self.number_of_used_buckets.fetch_sub(1, Ordering::Relaxed);
        self.allocator.deallocate_bucket(NonNull::new_unchecked(
            (offset.offset() + self.allocator.start_address()) as *mut u8,
        ));
    }
}

impl ShmAllocator for PoolAllocator {
    type Configuration = Config;

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
        self.allocator.start_address() - self.base_address
    }

    unsafe fn new_uninit(
        max_supported_alignment_by_memory: usize,
        managed_memory: NonNull<[u8]>,
        config: &Self::Configuration,
    ) -> Self {
        Self {
            allocator: iceoryx2_bb_memory::pool_allocator::PoolAllocator::new_uninit(
                config.bucket_layout,
                unsafe { NonNull::new_unchecked(managed_memory.as_ptr() as *mut u8) },
                managed_memory.len(),
            ),
            base_address: (managed_memory.as_ptr() as *mut u8) as usize,
            max_supported_alignment_by_memory,
            number_of_used_buckets: IoxAtomicUsize::new(0),
        }
    }

    fn max_alignment(&self) -> usize {
        self.allocator.max_alignment()
    }

    unsafe fn init<Allocator: BaseAllocator>(
        &mut self,
        mgmt_allocator: &Allocator,
    ) -> Result<(), ShmAllocatorInitError> {
        let msg = "Unable to initialize allocator";
        if self.max_supported_alignment_by_memory < self.max_alignment() {
            fail!(from self, with ShmAllocatorInitError::MaxSupportedMemoryAlignmentInsufficient,
                "{} since the required alignment {} exceeds the maximum supported alignment {} of the memory.",
                msg, self.max_alignment(), self.max_supported_alignment_by_memory);
        }

        fail!(from self, when self.allocator.init(mgmt_allocator),
            with ShmAllocatorInitError::AllocationFailed,
            "{} since the allocation of the allocator managment memory failed.", msg);
        Ok(())
    }

    fn unique_id() -> u8 {
        0
    }

    unsafe fn allocate(&self, layout: Layout) -> Result<PointerOffset, ShmAllocationError> {
        let msg = "Unable to allocate memory";
        if layout.align() > self.max_alignment() {
            fail!(from self, with ShmAllocationError::ExceedsMaxSupportedAlignment,
                "{} since an alignment of {} exceeds the maximum supported alignment of {}.",
                msg, layout.align(), self.max_alignment());
        }

        let chunk = fail!(from self, when self.allocator.allocate(layout), "{}.", msg);
        self.number_of_used_buckets.fetch_add(1, Ordering::Relaxed);
        Ok(PointerOffset::new(
            (chunk.as_ptr() as *const u8) as usize - self.allocator.start_address(),
        ))
    }

    unsafe fn deallocate(&self, offset: PointerOffset, _layout: Layout) {
        self.deallocate_bucket(offset);
    }
}
