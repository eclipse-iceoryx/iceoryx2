// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

pub mod dynamic;

use std::alloc::Layout;
use std::fmt::Debug;
use std::time::Duration;

use iceoryx2_bb_elementary::enum_gen;

use crate::named_concept::*;
use crate::shared_memory::{
    AllocationStrategy, SharedMemory, SharedMemoryCreateError, SharedMemoryOpenError, ShmPointer,
};
use crate::shm_allocator::{PointerOffset, ShmAllocationError, ShmAllocator};

enum_gen! { ResizableShmAllocationError
  mapping:
    ShmAllocationError,
    SharedMemoryCreateError
}

pub trait ResizableSharedMemoryBuilder<
    Allocator: ShmAllocator,
    Shm: SharedMemory<Allocator>,
    ResizableShm: ResizableSharedMemory<Allocator, Shm>,
    ResizableShmView: ResizableSharedMemoryView<Allocator, Shm>,
>: NamedConceptBuilder<ResizableShm>
{
    /// Defines if a newly created [`SharedMemory`] owns the underlying resources
    fn has_ownership(self, value: bool) -> Self;

    fn max_chunk_layout_hint(self, value: Layout) -> Self;

    fn max_number_of_chunks_hint(self, value: usize) -> Self;

    fn allocation_strategy(self, value: AllocationStrategy) -> Self;

    /// The timeout defines how long the [`SharedMemoryBuilder`] should wait for
    /// [`SharedMemoryBuilder::create()`] to finialize
    /// the initialization. This is required when the [`SharedMemory`] is created and initialized
    /// concurrently from another process. By default it is set to [`Duration::ZERO`] for no
    /// timeout.
    fn timeout(self, value: Duration) -> Self;

    /// Creates new [`SharedMemory`]. If it already exists the method will fail.
    fn create(self) -> Result<ResizableShm, SharedMemoryCreateError>;

    /// Opens already existing [`SharedMemory`]. If it does not exist or the initialization is not
    /// yet finished the method will fail.
    fn open(self) -> Result<ResizableShmView, SharedMemoryOpenError>;
}

pub trait ResizableSharedMemoryView<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>> {
    fn register_and_translate_offset(
        &mut self,
        offset: PointerOffset,
    ) -> Result<*const u8, SharedMemoryOpenError>;

    fn unregister_offset(&mut self, offset: PointerOffset);

    fn number_of_active_segments(&self) -> usize;
}

pub trait ResizableSharedMemory<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>:
    Sized + NamedConcept + NamedConceptMgmt + Debug
{
    type Builder: ResizableSharedMemoryBuilder<Allocator, Shm, Self, Self::View>;
    type View: ResizableSharedMemoryView<Allocator, Shm>;

    fn max_number_of_reallocations() -> usize;

    fn number_of_active_segments(&self) -> usize;

    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<ShmPointer, ResizableShmAllocationError>;

    /// Release previously allocated memory
    ///
    /// # Safety
    ///
    ///  * the offset must be acquired with [`SharedMemory::allocate()`] - extracted from the
    ///    [`ShmPointer`]
    ///  * the layout must be identical to the one used in [`SharedMemory::allocate()`]
    unsafe fn deallocate(&self, offset: PointerOffset, layout: std::alloc::Layout);

    /// Returns if the [`SharedMemory`] supports persistency, meaning that the underlying OS
    /// resource remain even when every [`SharedMemory`] instance in every process was removed.
    fn does_support_persistency() -> bool;

    /// Returns true if the [`SharedMemory`] holds the ownership, otherwise false
    fn has_ownership(&self) -> bool;

    /// Acquires the ownership of the [`SharedMemory`]. When the object goes out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self);

    /// Releases the ownership of the [`SharedMemory`] meaning when it goes out of scope the
    /// underlying resource will not be removed.
    fn release_ownership(&self);
}
