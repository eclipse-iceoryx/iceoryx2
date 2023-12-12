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

use std::{alloc::Layout, ptr::NonNull};

pub use iceoryx2_bb_elementary::allocator::AllocationError;
use iceoryx2_bb_elementary::{
    allocator::{BaseAllocator, DeallocationError},
    enum_gen,
};

pub trait ShmAllocatorConfig: Copy + Default {}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct PointerOffset(usize);

impl PointerOffset {
    pub fn new(value: usize) -> PointerOffset {
        Self(value)
    }

    pub fn value(&self) -> usize {
        self.0
    }
}

enum_gen! { ShmAllocationError
  entry:
    ExceedsMaxSupportedAlignment
  mapping:
    AllocationError
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ShmAllocatorInitError {
    MaxSupportedMemoryAlignmentInsufficient,
    AllocationFailed,
}

/// Every allocator implementation must be relocatable. The allocator itself must be stored either
/// in the same shared memory segment or in a separate shared memory segment of a different type
/// but accessible by all participating processes.
pub trait ShmAllocator: Send + Sync + 'static {
    type Configuration: ShmAllocatorConfig;

    fn management_size(memory_size: usize, config: &Self::Configuration) -> usize;

    /// Creates a new uninitialized shared memory allocator.
    ///
    /// # Safety
    ///
    /// * the method [`ShmAllocator::init()`] must be called before any other method is called
    ///
    unsafe fn new_uninit(
        max_supported_alignment_by_memory: usize,
        base_address: NonNull<[u8]>,
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
        &self,
        allocator: &Allocator,
    ) -> Result<(), ShmAllocatorInitError>;

    /// Returns the unique id of the allocator. It is inequal to any other
    /// [`ShmAllocator::unique_id()`]
    fn unique_id() -> u8;

    /// Returns the max supported alignment by the allocator.
    fn max_alignment(&self) -> usize;

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
    unsafe fn deallocate(
        &self,
        distance: PointerOffset,
        layout: Layout,
    ) -> Result<(), DeallocationError>;
}
