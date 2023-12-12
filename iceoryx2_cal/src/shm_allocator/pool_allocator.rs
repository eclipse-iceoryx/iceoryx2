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

use std::{alloc::Layout, ptr::NonNull};

use crate::shm_allocator::{ShmAllocator, ShmAllocatorConfig};
use iceoryx2_bb_elementary::allocator::{BaseAllocator, DeallocationError};
use iceoryx2_bb_log::fail;

use super::{PointerOffset, ShmAllocationError, ShmAllocatorInitError};

#[derive(Clone, Copy)]
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
}

impl PoolAllocator {
    pub fn bucket_size(&self) -> usize {
        self.allocator.bucket_size()
    }

    pub fn number_of_buckets(&self) -> u32 {
        self.allocator.number_of_buckets()
    }
}

impl ShmAllocator for PoolAllocator {
    type Configuration = Config;

    fn management_size(memory_size: usize, config: &Self::Configuration) -> usize {
        iceoryx2_bb_memory::pool_allocator::PoolAllocator::memory_size(
            config.bucket_layout,
            memory_size,
        )
    }

    unsafe fn new_uninit(
        max_supported_alignment_by_memory: usize,
        base_address: NonNull<[u8]>,
        config: &Self::Configuration,
    ) -> Self {
        Self {
            allocator: iceoryx2_bb_memory::pool_allocator::PoolAllocator::new_uninit(
                config.bucket_layout,
                unsafe { NonNull::new_unchecked(base_address.as_ptr() as *mut u8) },
                base_address.len(),
            ),
            base_address: (base_address.as_ptr() as *mut u8) as usize,
            max_supported_alignment_by_memory,
        }
    }

    fn max_alignment(&self) -> usize {
        self.allocator.max_alignment()
    }

    unsafe fn init<Allocator: BaseAllocator>(
        &self,
        allocator: &Allocator,
    ) -> Result<(), ShmAllocatorInitError> {
        let msg = "Unable to initialize allocator";
        if self.max_supported_alignment_by_memory < self.max_alignment() {
            fail!(from self, with ShmAllocatorInitError::MaxSupportedMemoryAlignmentInsufficient,
                "{} since the required alignment {} exceeds the maximum supported alignment {} of the memory.",
                msg, self.max_alignment(), self.max_supported_alignment_by_memory);
        }

        fail!(from self, when self.allocator.init(allocator),
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

        let chunk = fail!(from self, when self.allocator.allocate(layout),
                                        "{}.", msg);
        Ok(PointerOffset::new(
            (chunk.as_ptr() as *const u8) as usize - self.base_address,
        ))
    }

    unsafe fn deallocate(
        &self,
        offset: PointerOffset,
        layout: Layout,
    ) -> Result<(), DeallocationError> {
        fail!(from self, when self.allocator.deallocate(NonNull::new_unchecked(
                    (offset.0 + self.base_address) as *mut u8), layout),
            "Failed to release shared memory chunk");

        Ok(())
    }
}
