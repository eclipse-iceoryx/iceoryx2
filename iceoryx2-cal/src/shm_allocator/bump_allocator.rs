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
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary::allocation_strategy::AllocationStrategy;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::{fail, fatal_panic};

#[derive(Default, Clone, Copy, Debug)]
pub struct Config {}

impl ShmAllocatorConfig for Config {}

#[derive(Debug, ZeroCopySend)]
#[repr(C)]
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

pub struct InitializedBumpAllocator<'shm_allocator>(&'shm_allocator BumpAllocator);

impl<'shm_allocator> BaseAllocator<PointerOffset> for InitializedBumpAllocator<'shm_allocator> {
    fn allocate(&self, layout: Layout) -> Result<PointerOffset, AllocationError> {
        let msg = "Unable to allocate memory";
        if layout.align() > self.0.max_alignment() {
            fail!(from self.0, with AllocationError::AlignmentFailure,
                    "{} since an alignment of {} exceeds the maximum supported alignment of {}.",
                    msg, layout.align(), self.0.max_alignment());
        }

        let chunk = fail!(from self.0, when self.0.allocator.allocate(layout),
                                            "{}.", msg);
        Ok(PointerOffset::new(
            (chunk.as_ptr() as *const u8) as usize - self.0.base_address,
        ))
    }
}

impl<'shm_allocator> Dealloc<PointerOffset> for InitializedBumpAllocator<'shm_allocator> {
    unsafe fn deallocate(&self, _ptr: PointerOffset, _layout: Layout) {
        self.0.allocator.reset();
    }
}

impl<'shm_allocator> ReallocGrow<PointerOffset> for InitializedBumpAllocator<'shm_allocator> {
    unsafe fn grow(
        &self,
        offset: PointerOffset,
        old_layout: Layout,
        new_layout: Layout,
        content_placement: ContentPlacement,
    ) -> Result<PointerOffset, AllocationGrowError> {
        let msg = "Unable to grow memory";
        if new_layout.size() < old_layout.size() {
            fail!(from self.0, with AllocationGrowError::GrowWouldShrink,
                        "{} since new layout has a smaller size of {} than the old layout with {}.",
                        msg, new_layout.size(), old_layout.size());
        }

        if new_layout.align() > old_layout.align() {
            fail!(from self.0, with AllocationGrowError::AlignmentFailure,
                        "{} since new layout alignment increased which is not supported - from {} to {}.",
                        msg, old_layout.align(), new_layout.align());
        }

        if new_layout.size() == old_layout.size() {
            return Ok(offset);
        }

        // this chunk is located at the end of the managed memory range and we need to allocate just the size
        // diff
        if old_layout.size() + offset.offset() == self.0.allocator.used_space() {
            let additional_size = new_layout.size() - old_layout.size();
            match self
                .0
                .allocator
                .allocate(unsafe { Layout::from_size_align_unchecked(additional_size, 1) })
            {
                Ok(_) => (),
                Err(AllocationError::OutOfMemory) | Err(AllocationError::SizeTooLarge) => {
                    fail!(from self.0,
                                with AllocationGrowError::OutOfMemory,
                                "{} since the allocator is out-of-memory.", msg);
                }
                Err(e) => {
                    fatal_panic!(from self.0,
                                "This should never happen! Failed to allocate memory to grow the memory chunk. [{e:?}]");
                }
            }

            if content_placement == ContentPlacement::Back {
                let src = self.0.allocator.start_address() as usize + offset.offset();
                let dst = src + (new_layout.size() - old_layout.size());
                unsafe { core::ptr::copy(src as *const u8, dst as *mut u8, old_layout.size()) };
            }

            Ok(offset)
        } else {
            match self.allocate(new_layout) {
                Ok(new_offset) => {
                    let src = self.0.allocator.start_address() as usize + offset.offset();
                    match content_placement {
                        ContentPlacement::Front => {
                            let dst =
                                self.0.allocator.start_address() as usize + new_offset.offset();
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    src as *const u8,
                                    dst as *mut u8,
                                    old_layout.size(),
                                )
                            };

                            Ok(new_offset)
                        }
                        ContentPlacement::Back => {
                            let dst = self.0.allocator.start_address() as usize
                                + new_offset.offset()
                                + new_layout.size()
                                - old_layout.size();
                            unsafe {
                                core::ptr::copy_nonoverlapping(
                                    src as *const u8,
                                    dst as *mut u8,
                                    old_layout.size(),
                                )
                            };

                            Ok(new_offset)
                        }
                    }
                }
                Err(AllocationError::OutOfMemory) | Err(AllocationError::SizeTooLarge) => {
                    fail!(from self.0,
                                with AllocationGrowError::OutOfMemory,
                                "{} since the allocator is out-of-memory.", msg);
                }
                Err(e) => {
                    fatal_panic!(from self.0,
                                "This should never happen! Failed to allocate memory to grow the memory chunk. [{e:?}]");
                }
            }
        }
    }
}

impl<'shm_allocator> InitializedShmAllocator<'shm_allocator>
    for InitializedBumpAllocator<'shm_allocator>
{
}

impl ShmAllocator for BumpAllocator {
    type Configuration = Config;
    type Initialized<'shm_allocator> = InitializedBumpAllocator<'shm_allocator>;

    unsafe fn assume_init<'shm_allocator>(
        &'shm_allocator self,
    ) -> Self::Initialized<'shm_allocator> {
        InitializedBumpAllocator(self)
    }

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
        self.allocator.start_address() as usize - self.base_address
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

    unsafe fn init<Allocator: BaseAllocator<NonNull<u8>>>(
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
}
