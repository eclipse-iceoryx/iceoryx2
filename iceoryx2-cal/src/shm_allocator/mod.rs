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

use iceoryx2_bb_elementary::allocation_strategy::AllocationStrategy;
pub use iceoryx2_bb_elementary_traits::allocator::{AllocationError, AllocationGrowError};
use iceoryx2_bb_elementary_traits::{allocator::BaseAllocator, zero_copy_send::ZeroCopySend};
use iceoryx2_bb_memory::pool_allocator::{ContentPlacement, Dealloc, ReallocGrow};
pub use pointer_offset::*;

/// Trait that identifies a configuration of a [`ShmAllocator`].
pub trait ShmAllocatorConfig: Copy + Default + Debug + Send {}

/// Describes error that may occur when a [`ShmAllocator`] is initialized.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ShmAllocatorInitError {
    /// The [`SharedMemory`](crate::shared_memory::SharedMemory) max supported alignment does not
    /// satisfy the required alignment of the [`ShmAllocator`].
    MaxSupportedMemoryAlignmentInsufficient,
    /// The [`ShmAllocator`] requires more memory to initialize than available.
    AllocationFailed,
}

impl core::fmt::Display for ShmAllocatorInitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ShmAllocatorInitError::{self:?}")
    }
}

impl core::error::Error for ShmAllocatorInitError {}

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

pub trait InitializedShmAllocator<'shm_allocator>:
    BaseAllocator<PointerOffset> + Dealloc<PointerOffset> + ReallocGrow<PointerOffset>
{
}

/// Every allocator implementation must be relocatable. The allocator itself must be stored either
/// in the same shared memory segment or in a separate shared memory segment of a different type
/// but accessible by all participating processes.
pub trait ShmAllocator: Debug + Send + Sync + 'static + ZeroCopySend {
    type Configuration: ShmAllocatorConfig;
    type Initialized<'shm_allocator>: InitializedShmAllocator<'shm_allocator>;

    /// Returns an [`InitializedShmAllocator`] to perform allocation operations.
    ///
    /// # Safety
    ///
    /// * [`ShmAllocator::init()`] must have been called before using this method
    ///
    unsafe fn assume_init<'shm_allocator>(
        &'shm_allocator self,
    ) -> Self::Initialized<'shm_allocator>;

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
    unsafe fn init<Allocator: BaseAllocator<NonNull<u8>>>(
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
}
