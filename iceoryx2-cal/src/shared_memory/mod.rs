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

//! A [`SharedMemory`] is identified by a name and allows two processes to share memory with each
//! other (inter-process memory).
//!
//! One process creates the [`SharedMemory`] and multiple processes can then open the
//! [`SharedMemory`] with the [`SharedMemoryBuilder`].
//!
//! # Example
//!
//! ```
//! use iceoryx2_cal::shared_memory::*;
//! use iceoryx2_cal::named_concept::*;
//! use iceoryx2_cal::shm_allocator::pool_allocator::PoolAllocator;
//! use iceoryx2_cal::shm_allocator::pool_allocator;
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use core::alloc::Layout;
//!
//! fn process_one<Shm: SharedMemory<PoolAllocator>>() {
//!     let shm_name = FileName::new(b"myShmName").unwrap();
//!     let allocator_config = pool_allocator::Config {
//!         // we want to allocate [`u64`]
//!         bucket_layout: Layout::new::<u64>()
//!     };
//!     let shm = Shm::Builder::new(&shm_name).size(1024).create(&allocator_config).unwrap();
//!     let mut shm_pointer = shm.allocate(Layout::new::<u64>()).unwrap();
//!     unsafe { shm_pointer.data_ptr.write(123) };
//!
//!     // send shm_pointer to another process with [`ZeroCopyConnection`]
//! }
//!
//! fn process_two<Shm: SharedMemory<PoolAllocator>>() {
//!     let shm_name = FileName::new(b"myShmName").unwrap();
//!     let allocator_config = pool_allocator::Config {
//!         // we want to allocate [`u64`]
//!         bucket_layout: Layout::new::<u64>()
//!     };
//!     let shm = Shm::Builder::new(&shm_name).size(1024).open().unwrap();
//!     let mut shm_pointer = shm.allocate(Layout::new::<u64>()).unwrap();
//!     unsafe { shm_pointer.data_ptr.write(31) }
//!
//!     // send shm_pointer to another process with [`ZeroCopyConnection`]
//! }
//! ```

pub mod common;
pub mod posix;
pub mod process_local;
pub mod recommended;

use core::{fmt::Debug, time::Duration};

pub use crate::shm_allocator::*;
use crate::static_storage::file::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};
use iceoryx2_bb_system_types::file_name::*;
use pool_allocator::PoolAllocator;

/// Failure returned by [`SharedMemoryBuilder::create()`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SharedMemoryCreateError {
    AlreadyExists,
    SizeIsZero,
    InsufficientPermissions,
    InternalError,
}

/// Failure returned by [`SharedMemoryBuilder::open()`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SharedMemoryOpenError {
    DoesNotExist,
    InsufficientPermissions,
    SizeIsZero,
    SizeDoesNotFit,
    WrongAllocatorSelected,
    InitializationNotYetFinalized,
    VersionMismatch,
    InternalError,
}

/// Represents a pointer pointing to some [`SharedMemory`]. Consists of the actual data pointer and
/// an [`PointerOffset`] which can be used in combination with a
/// [`crate::zero_copy_connection::ZeroCopyConnection`]
#[derive(Debug)]
pub struct ShmPointer {
    pub offset: PointerOffset,
    pub data_ptr: *mut u8,
}

#[doc(hidden)]
pub(crate) mod details {
    use super::*;

    pub trait SharedMemoryLowLevelAPI<Allocator: ShmAllocator> {
        fn allocator(&self) -> &Allocator;
    }
}

/// Creates [`SharedMemory`].
pub trait SharedMemoryBuilder<Allocator: ShmAllocator, Shm: SharedMemory<Allocator>>:
    NamedConceptBuilder<Shm>
{
    /// Defines if a newly created [`SharedMemory`] owns the underlying resources
    fn has_ownership(self, value: bool) -> Self;

    /// Sets the size of the [`SharedMemory`]. Only relevant when the [`SharedMemory`] is created
    /// otherwise the already initialized [`SharedMemory`] is completely mapped into the process
    /// space.
    fn size(self, value: usize) -> Self;

    /// The timeout defines how long the [`SharedMemoryBuilder`] should wait for
    /// [`SharedMemoryBuilder::create()`] to finialize
    /// the initialization. This is required when the [`SharedMemory`] is created and initialized
    /// concurrently from another process. By default it is set to [`Duration::ZERO`] for no
    /// timeout.
    fn timeout(self, value: Duration) -> Self;

    /// Creates new [`SharedMemory`]. If it already exists the method will fail.
    fn create(
        self,
        allocator_config: &Allocator::Configuration,
    ) -> Result<Shm, SharedMemoryCreateError>;

    /// Opens already existing [`SharedMemory`]. If it does not exist or the initialization is not
    /// yet finished the method will fail.
    fn open(self) -> Result<Shm, SharedMemoryOpenError>;
}

/// Abstract concept of a memory shared between multiple processes. Can be created with the
/// [`SharedMemoryBuilder`].
pub trait SharedMemory<Allocator: ShmAllocator>:
    Sized + Debug + NamedConcept + NamedConceptMgmt + details::SharedMemoryLowLevelAPI<Allocator> + Send
{
    type Builder: SharedMemoryBuilder<Allocator, Self>;

    /// Returns the size of the shared memory.
    fn size(&self) -> usize;

    /// Returns the max supported alignment.
    fn max_alignment(&self) -> usize;

    /// Returns the start address of the shared memory. Used by the [`ShmPointer`] to calculate
    /// the actual memory position.
    fn payload_start_address(&self) -> usize;

    /// Allocates memory. The alignment in the layout must be smaller or equal
    /// [`SharedMemory::max_alignment()`] otherwise the method will fail.
    fn allocate(&self, layout: core::alloc::Layout) -> Result<ShmPointer, ShmAllocationError>;

    /// Release previously allocated memory
    ///
    /// # Safety
    ///
    ///  * the offset must be acquired with [`SharedMemory::allocate()`] - extracted from the
    ///    [`ShmPointer`]
    ///  * the layout must be identical to the one used in [`SharedMemory::allocate()`]
    unsafe fn deallocate(&self, offset: PointerOffset, layout: core::alloc::Layout);

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

pub trait SharedMemoryForPoolAllocator: SharedMemory<PoolAllocator> {
    /// Release previously allocated memory
    ///
    /// # Safety
    ///
    ///  * the offset must be acquired with [`SharedMemory::allocate()`] - extracted from the
    ///    [`ShmPointer`]
    unsafe fn deallocate_bucket(&self, offset: PointerOffset);

    /// Returns the bucket size of the [`PoolAllocator`]
    fn bucket_size(&self) -> usize;
}
