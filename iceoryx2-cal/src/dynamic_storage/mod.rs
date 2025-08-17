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

//! Traits that provide modifyable memory which can be accessed by multiple processes
//! identified by a name.
//!
//! A [`DynamicStorage`] has to fulfill the following contract:
//!  * zero sized names are not valid
//!  * **unique:** multiple [`DynamicStorage`]s with the same cannot be created
//!  * non-existing [`DynamicStorage`]s cannot be opened
//!
//! The contract is verified by the corresponding unit tests. Every [`DynamicStorage`] must
//! pass the test.
//!
//! **Important:** It is not the task of the [`DynamicStorage`] to ensure a thread-safe access to
//! the underlying object. If the [`DynamicStorage`] is used in an inter-process environment every
//! access must be considered as a concurrent access!
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_cal::dynamic_storage::*;
//! use iceoryx2_cal::named_concept::*;
//! use core::sync::atomic::{AtomicU64, Ordering};
//!
//! // the following two functions can be implemented in different processes
//! fn process_one<Storage: DynamicStorage<AtomicU64>>() {
//!     let storage_name = FileName::new(b"myStorageName").unwrap();
//!     let mut storage = Storage::Builder::new(&storage_name)
//!                         .create(AtomicU64::new(873)).unwrap();
//!
//!     println!("Created storage {}", storage.name());
//!     storage.get().store(991, Ordering::Relaxed);
//! }
//!
//! fn process_two<Storage: DynamicStorage<AtomicU64>>() {
//!     let storage_name = FileName::new(b"myStorageName").unwrap();
//!     let mut storage = Storage::Builder::new(&storage_name)
//!                         .open().unwrap();
//!
//!     println!("Opened storage {}", storage.name());
//!     println!("Current value {}", storage.get().swap(1001, Ordering::Relaxed));
//! }
//! ```

use core::{fmt::Debug, time::Duration};

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_memory::bump_allocator::BumpAllocator;
use iceoryx2_bb_system_types::file_name::*;
use tiny_fn::tiny_fn;

use crate::static_storage::file::{NamedConcept, NamedConceptBuilder, NamedConceptMgmt};

tiny_fn! {
    /// The callback called to initialize the data inside the [`DynamicStorage`]
    pub struct Initializer<T> = FnMut(value: &mut T, allocator: &mut BumpAllocator) -> bool;
}

impl<T> Debug for Initializer<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "")
    }
}

#[doc(hidden)]
pub mod dynamic_storage_configuration;
pub mod posix_shared_memory;
pub mod process_local;
pub mod recommended;

/// Describes failures when creating a new [`DynamicStorage`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum DynamicStorageCreateError {
    AlreadyExists,
    InsufficientPermissions,
    InitializationFailed,
    InternalError,
}

/// Describes failures when opening a new [`DynamicStorage`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum DynamicStorageOpenError {
    DoesNotExist,
    InitializationNotYetFinalized,
    VersionMismatch,
    InternalError,
}

enum_gen! {
    DynamicStorageOpenOrCreateError
  mapping:
    DynamicStorageOpenError,
    DynamicStorageCreateError
}

/// Builder for the [`DynamicStorage`]. T is not allowed to implement the [`Drop`] trait.
pub trait DynamicStorageBuilder<'builder, T: Send + Sync, D: DynamicStorage<T>>:
    Debug + Sized + NamedConceptBuilder<D>
{
    /// Defines if `T::Drop` shall be called when the [`DynamicStorage`] is removed. The default
    /// is [`true`].
    fn call_drop_on_destruction(self, value: bool) -> Self;

    /// Defines if a newly created [`DynamicStorage`] owns the underlying resources. The default
    /// is [`true`].
    fn has_ownership(self, value: bool) -> Self;

    /// Sets the size of the supplementary data. Only relevant when it is newly created otherwise
    /// the already initialized [`DynamicStorage`] with the full size is used.
    fn supplementary_size(self, value: usize) -> Self;

    /// The timeout defines how long the [`DynamicStorageBuilder`] should wait for
    /// [`DynamicStorageBuilder::create()`]
    /// to finialize the initialization. This is required when the [`DynamicStorage`] is
    /// created and initialized concurrently from another process.
    /// By default it is set to [`Duration::ZERO`] for no timeout.
    fn timeout(self, value: Duration) -> Self;

    /// Before the construction is finalized the initializer is called
    /// with a mutable reference to the new value and a mutable reference to a bump allocator
    /// which provides access to the supplementary memory. If the initialization failed it
    /// shall return false, otherwise true.
    fn initializer<F: FnMut(&mut T, &mut BumpAllocator) -> bool + 'builder>(self, value: F)
        -> Self;

    /// Creates a new [`DynamicStorage`]. The returned object has the ownership of the
    /// [`DynamicStorage`] and when it goes out of scope the underlying resources shall be
    /// removed without corrupting already opened [`DynamicStorage`]s.
    fn create(self, initial_value: T) -> Result<D, DynamicStorageCreateError>;

    /// Opens a [`DynamicStorage`]. The implementation must ensure that a [`DynamicStorage`]
    /// which is in the midst of creation cannot be opened. If the [`DynamicStorage`] does not
    /// exist or is not initialized it fails.
    fn open(self) -> Result<D, DynamicStorageOpenError>;

    /// Opens the [`DynamicStorage`] if it exists, otherwise it creates it.
    fn open_or_create(self, initial_value: T) -> Result<D, DynamicStorageOpenOrCreateError>;
}

/// Is being built by the [`DynamicStorageBuilder`]. The [`DynamicStorage`] trait shall provide
/// inter-process access to a modifyable piece of memory identified by some name.
pub trait DynamicStorage<T: Send + Sync>:
    Sized + Debug + NamedConceptMgmt + NamedConcept + Send + Sync
{
    type Builder<'builder>: DynamicStorageBuilder<'builder, T, Self>;

    /// Returns if the [`DynamicStorage`] supports persistency, meaning that the underlying OS
    /// resource remain even when every [`DynamicStorage`] instance in every process was removed.
    fn does_support_persistency() -> bool;

    /// Returns true if the storage holds the ownership, otherwise false.
    fn has_ownership(&self) -> bool;

    /// Releases the ownership of the [`DynamicStorage`]. When the object goes out of scope it is
    /// no longer removed.
    fn release_ownership(&self);

    /// Acquires the ownership of the [`DynamicStorage`]. When the object goes out of scope the
    /// underlying resources will be removed.
    fn acquire_ownership(&self);

    /// Returns a const reference to the underlying object. It is const since the [`DynamicStorage`]
    /// can be accessed by multiple processes concurrently therefore it must be constant or
    /// thread-safe.
    fn get(&self) -> &T;

    /// The default suffix of every dynamic storage
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".dyn") }
    }

    #[doc(hidden)]
    /// # Safety
    ///
    /// * ensure that the contained type matches the semantic type_name given with `T`
    /// * if `T` is some arbitary placeholder, then only use it to remove the concept or list it
    ///   * DO NOT OPEN IT
    unsafe fn __internal_set_type_name_in_config(config: &mut Self::Configuration, type_name: &str);
}
