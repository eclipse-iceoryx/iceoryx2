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

pub mod bump_allocator;
pub mod pool_allocator;

use std::{alloc::Layout, fmt::Debug, ptr::NonNull};

pub use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_elementary::{allocator::BaseAllocator, enum_gen};

pub trait ShmAllocatorConfig: Copy + Default + Debug {}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct PointerOffset(u64);

impl PointerOffset {
    pub fn new(offset: usize) -> PointerOffset {
        let segment_id = 0;
        Self((offset as u64) << 8 | segment_id as u64)
    }

    pub fn offset(&self) -> usize {
        (self.0 >> 8) as usize
    }

    pub fn segment_id(&self) -> u8 {
        (self.0 & 0x00000000000000ff) as u8
    }
}

enum_gen! { ShmAllocationError
  entry:
    ExceedsMaxSupportedAlignment
  mapping:
    AllocationError
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum AllocationStrategy {
    BestFit,
    #[default]
    PowerOfTwo,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ShmAllocatorInitError {
    MaxSupportedMemoryAlignmentInsufficient,
    AllocationFailed,
}

pub struct ResizeHint<Config: ShmAllocatorConfig> {
    pub(crate) payload_size: usize,
    pub(crate) config: Config,
}

/// Every allocator implementation must be relocatable. The allocator itself must be stored either
/// in the same shared memory segment or in a separate shared memory segment of a different type
/// but accessible by all participating processes.
pub trait ShmAllocator: Debug + Send + Sync + 'static {
    type Configuration: ShmAllocatorConfig;
    /// Suggest a new payload size by considering the current allocation state in combination with
    /// a provided [`AllocationStrategy`] and a `layout` that shall be allocatable.
    fn resize_hint(
        &self,
        layout: Layout,
        strategy: AllocationStrategy,
    ) -> ResizeHint<Self::Configuration>;

    /// Suggest a managed payload size under a provided configuration assuming that at most
    /// `max_number_of_chunks` of memory are in use in parallel.
    fn payload_size_hint(config: &Self::Configuration, max_number_of_chunks: usize) -> usize;

    /// Returns the required memory size of the additional dynamic part of the allocator that is
    /// allocated in [`ShmAllocator::init()`].
    fn management_size(memory_size: usize, config: &Self::Configuration) -> usize;

    /// Creates a new uninitialized shared memory allocator.
    ///
    /// # Safety
    ///
    /// * the method [`ShmAllocator::init()`] must be called before any other method is called
    ///
    unsafe fn new_uninit(
        max_supported_alignment_by_memory: usize,
        managed_memory: NonNull<[u8]>,
        config: &Self::Configuration,
    ) -> Self;

    /// Initializes the shared memory allocator.
    ///
    /// # Safety
    ///
    /// * must be called only once
    /// * must be called before any other method is called
    ///
    unsafe fn init<Allocator: BaseAllocator>(
        &mut self,
        mgmt_allocator: &Allocator,
    ) -> Result<(), ShmAllocatorInitError>;

    /// Returns the unique id of the allocator. It is inequal to any other
    /// [`ShmAllocator::unique_id()`]
    fn unique_id() -> u8;

    /// Returns the max supported alignment by the allocator.
    fn max_alignment(&self) -> usize;

    /// Returns the offset to the beginning of the allocator payload. The smallest offset a user
    /// can allocate.
    fn relative_start_address(&self) -> usize;

    /// Allocates memory and returns the pointer offset.
    ///
    /// # Safety
    ///
    /// * [`ShmAllocator::init()`] must have been called before using this method
    ///
    unsafe fn allocate(&self, layout: Layout) -> Result<PointerOffset, ShmAllocationError>;

    /// Deallocates a previously allocated pointer offset
    ///
    /// # Safety
    ///
    /// * the provided distance must have been allocated before with the same layout
    /// * [`ShmAllocator::init()`] must have been called before using this method
    ///
    unsafe fn deallocate(&self, distance: PointerOffset, layout: Layout);
}
