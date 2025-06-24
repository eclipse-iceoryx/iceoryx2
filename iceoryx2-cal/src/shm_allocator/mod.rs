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
pub mod pointer_offset;
pub mod pool_allocator;

use core::{alloc::Layout, fmt::Debug, ptr::NonNull};

use iceoryx2_bb_elementary::enum_gen;
pub use iceoryx2_bb_elementary_traits::allocator::AllocationError;
use iceoryx2_bb_elementary_traits::allocator::BaseAllocator;
pub use pointer_offset::*;

/// Trait that identifies a configuration of a [`ShmAllocator`].
pub trait ShmAllocatorConfig: Copy + Default + Debug + Send {}

enum_gen! {
/// Describes the errors that can occur when [`ShmAllocator::allocate()`] is called.
    ShmAllocationError
  entry:
    ExceedsMaxSupportedAlignment
  mapping:
    AllocationError
}

/// Describes generically an [`AllocationStrategy`], meaning how the memory is increased when the
/// available memory is insufficient.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum AllocationStrategy {
    /// Increases the memory so that it perfectly fits the new size requirements. This may lead
    /// to a lot of reallocations but has the benefit that no byte is wasted.
    BestFit,
    /// Increases the memory by rounding the increased memory size up to the next power of two.
    /// Reduces reallocations a lot at the cost of increased memory usage.
    PowerOfTwo,
    /// The memory is not increased. This may lead to an out-of-memory error when allocating.
    #[default]
    Static,
}

/// Describes error that may occur when a [`ShmAllocator`] is initialized.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ShmAllocatorInitError {
    /// The [`SharedMemory`](crate::shared_memory::SharedMemory) max supported alignment does not
    /// satisfy the required alignment of the [`ShmAllocator`].
    MaxSupportedMemoryAlignmentInsufficient,
    /// The [`ShmAllocator`] requires more memory to initialize than available.
    AllocationFailed,
}

/// Returned by [`ShmAllocator::resize_hint()`] and [`ShmAllocator::initial_setup_hint()`].
/// It contains a payload size and [`ShmAllocator`] configuration suggestion for the given
/// parameters.
pub struct SharedMemorySetupHint<Config: ShmAllocatorConfig> {
    /// The payload size of the [`SharedMemory`](crate::shared_memory::SharedMemory)
    pub payload_size: usize,
    /// The [`ShmAllocatorConfig`] that shall be used for the
    /// [`SharedMemory`](crate::shared_memory::SharedMemory)
    pub config: Config,
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
