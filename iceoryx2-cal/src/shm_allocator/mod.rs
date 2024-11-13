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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentId(u16);

impl SegmentId {
    pub fn new(value: u16) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PointerOffset(u64);

impl PointerOffset {
    pub fn new(offset: usize) -> PointerOffset {
        const SEGMENT_ID: u64 = 0;
        Self((offset as u64) << core::mem::size_of::<SegmentId>() | SEGMENT_ID as u64)
    }

    pub fn set_segment_id(&mut self, value: SegmentId) {
        self.0 = (self.offset() as u64) << core::mem::size_of::<SegmentId>() | value.0 as u64;
    }

    pub fn offset(&self) -> usize {
        (self.0 >> core::mem::size_of::<SegmentId>()) as usize
    }

    pub fn segment_id(&self) -> SegmentId {
        SegmentId((self.0 & 0x000000000000ffff) as u16)
    }
}

impl Debug for PointerOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PointerOffset {{ offset: {}, segment_id: {:?} }}",
            self.offset(),
            self.segment_id()
        )
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

pub struct SharedMemorySetupHint<Config: ShmAllocatorConfig> {
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
    ) -> SharedMemorySetupHint<Self::Configuration>;

    /// Suggest a managed payload size under a provided configuration assuming that at most
    /// `max_number_of_chunks` of memory are in use in parallel.
    fn initial_setup_hint(
        max_chunk_layout: Layout,
        max_number_of_chunks: usize,
    ) -> SharedMemorySetupHint<Self::Configuration>;

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
