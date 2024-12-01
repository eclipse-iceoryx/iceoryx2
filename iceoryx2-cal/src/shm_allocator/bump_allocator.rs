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

use crate::shm_allocator::*;
use iceoryx2_bb_log::fail;

#[derive(Default, Clone, Copy, Debug)]
pub struct Config {}

impl ShmAllocatorConfig for Config {}

#[derive(Debug)]
pub struct BumpAllocator {
    allocator: iceoryx2_bb_memory::bump_allocator::BumpAllocator,
    base_address: usize,
    max_supported_alignment_by_memory: usize,
}

impl BumpAllocator {
    pub fn total_space(&self) -> usize {
        self.allocator.total_space()
    }
}

impl ShmAllocator for BumpAllocator {
    type Configuration = Config;

    fn resize_hint(
        &self,
        layout: Layout,
        strategy: AllocationStrategy,
    ) -> SharedMemorySetupHint<Self::Configuration> {
        let current_payload_size = self.allocator.total_space();
        if layout.size() < self.allocator.free_space() {
            return SharedMemorySetupHint {
                payload_size: current_payload_size,
                config: Self::Configuration::default(),
            };
        }

        let payload_size = match strategy {
            AllocationStrategy::BestFit => current_payload_size + layout.size(),
            AllocationStrategy::PowerOfTwo => {
                (current_payload_size + layout.size()).next_power_of_two()
            }
            AllocationStrategy::Static => current_payload_size,
        };

        SharedMemorySetupHint {
            payload_size,
            config: Self::Configuration::default(),
        }
    }

    fn initial_setup_hint(
        max_chunk_layout: Layout,
        max_number_of_chunks: usize,
    ) -> SharedMemorySetupHint<Self::Configuration> {
        SharedMemorySetupHint {
            config: Self::Configuration::default(),
            payload_size: max_chunk_layout.size() * max_number_of_chunks,
        }
    }

    fn management_size(_memory_size: usize, _config: &Self::Configuration) -> usize {
        0
    }

    fn relative_start_address(&self) -> usize {
        self.allocator.start_address() - self.base_address
    }

    unsafe fn new_uninit(
        max_supported_alignment_by_memory: usize,
        managed_memory: NonNull<[u8]>,
        _config: &Self::Configuration,
    ) -> Self {
        Self {
            allocator: iceoryx2_bb_memory::bump_allocator::BumpAllocator::new(
                unsafe { NonNull::new_unchecked(managed_memory.as_ptr() as *mut u8) },
                managed_memory.len(),
            ),
            base_address: (managed_memory.as_ptr() as *mut u8) as usize,
            max_supported_alignment_by_memory,
        }
    }

    fn max_alignment(&self) -> usize {
        8
    }

    unsafe fn init<Allocator: BaseAllocator>(
        &mut self,
        _mgmt_allocator: &Allocator,
    ) -> Result<(), ShmAllocatorInitError> {
        let msg = "Unable to initialize allocator";
        if self.max_supported_alignment_by_memory < self.max_alignment() {
            fail!(from self, with ShmAllocatorInitError::MaxSupportedMemoryAlignmentInsufficient,
                "{} since the required alignment {} exceeds the maximum supported alignment {} of the memory.",
                msg, self.max_alignment(), self.max_supported_alignment_by_memory);
        }

        Ok(())
    }

    fn unique_id() -> u8 {
        1
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

    unsafe fn deallocate(&self, offset: PointerOffset, layout: Layout) {
        self.allocator.deallocate(
            NonNull::new_unchecked((offset.offset() + self.base_address) as *mut u8),
            layout,
        );
    }
}
