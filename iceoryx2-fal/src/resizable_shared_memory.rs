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

use std::alloc::Layout;
use std::collections::VecDeque;
use std::time::Duration;

use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_cal::shared_memory::{SharedMemory, SharedMemoryCreateError, SharedMemoryOpenError};
use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
use iceoryx2_cal::shm_allocator::ShmAllocator;

#[derive(Default)]
pub enum AllocationStrategy {
    #[default]
    PowerOfTwo,
    BestFit,
    Static,
}

pub struct ResizableSharedMemoryBuilder {}

impl ResizableSharedMemoryBuilder {
    /// Defines the prefix that the concept will use.
    fn prefix(self, value: &FileName) -> Self {
        todo!()
    }

    /// Defines the suffix that the concept will use.
    fn suffix(self, value: &FileName) -> Self {
        todo!()
    }

    /// Defines if a newly created [`SharedMemory`] owns the underlying resources
    fn has_ownership(self, value: bool) -> Self {
        todo!()
    }

    fn bucket_layout_hint(self, layout: Layout) -> Self {
        todo!()
    }

    fn number_of_buckets_hint(self, value: usize) -> Self {
        todo!()
    }

    /// The timeout defines how long the [`SharedMemoryBuilder`] should wait for
    /// [`SharedMemoryBuilder::create()`] to finialize
    /// the initialization. This is required when the [`SharedMemory`] is created and initialized
    /// concurrently from another process. By default it is set to [`Duration::ZERO`] for no
    /// timeout.
    fn timeout(self, value: Duration) -> Self {
        todo!()
    }

    /// Creates new [`SharedMemory`]. If it already exists the method will fail.
    fn create<T: SharedMemory<PoolAllocator>>(
        self,
    ) -> Result<ResizableSharedMemory<T>, SharedMemoryCreateError> {
        todo!()
    }

    /// Opens already existing [`SharedMemory`]. If it does not exist or the initialization is not
    /// yet finished the method will fail.
    fn open<T: SharedMemory<PoolAllocator>>(
        self,
    ) -> Result<ResizableSharedMemory<T>, SharedMemoryOpenError> {
        todo!()
    }
}

pub struct ResizableSharedMemory<T: SharedMemory<PoolAllocator>> {
    shared_memory_vec: VecDeque<T>,
}

impl<T: SharedMemory<PoolAllocator>> ResizableSharedMemory<T> {
    /// Returns the size of the shared memory.
    fn size(&self) -> usize {
        todo!()
    }

    /// Returns the max supported alignment.
    fn max_alignment(&self) -> usize;

    /// Returns the start address of the shared memory. Used by the [`ShmPointer`] to calculate
    /// the actual memory position.
    fn payload_start_address(&self) -> usize;

    /// Allocates memory. The alignment in the layout must be smaller or equal
    /// [`SharedMemory::max_alignment()`] otherwise the method will fail.
    fn allocate(&self, layout: std::alloc::Layout) -> Result<ShmPointer, ShmAllocationError>;

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

    /// The default suffix of every shared memory
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".shm") }
    }
}
