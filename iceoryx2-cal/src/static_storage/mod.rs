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

//! Traits that provide read-only memory which can be accessed by multiple processes
//! identified by a name.

pub mod file;
pub mod process_local;
pub mod recommended;

use core::{fmt::Debug, time::Duration};

use iceoryx2_bb_log::fail;
use iceoryx2_bb_system_types::file_name::*;

use crate::named_concept::{
    NamedConcept, NamedConceptBuilder, NamedConceptConfiguration, NamedConceptMgmt,
};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum StaticStorageCreateError {
    AlreadyExists,
    Creation,
    Write,
    InsufficientPermissions,
    InternalError,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum StaticStorageOpenError {
    DoesNotExist,
    Read,
    InitializationNotYetFinalized,
    InternalError,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum StaticStorageReadError {
    BufferTooSmall,
    ReadError,
    StaticStorageWasModified,
    CreationNotComplete,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum StaticStorageUnlockError {
    InsufficientPermissions,
    NoSpaceLeft,
    InternalError,
}

/// A custom configuration which can be used by the [`StaticStorageBuilder`] to create a
/// [`StaticStorage`] with implementation specific settings.
pub trait StaticStorageConfiguration: Clone + Default + NamedConceptConfiguration {}

/// Creates either a [`StaticStorage`], that can own the [`StaticStorage`] if it was created with
/// [`StaticStorageBuilder::has_ownership()`] (default = true) or a [`StaticStorageLocked`] that is
/// not yet set.
pub trait StaticStorageBuilder<T: StaticStorage>: Sized + NamedConceptBuilder<T> {
    /// Defines if a newly created [`StaticStorage`] owns the underlying resources
    fn has_ownership(self, value: bool) -> Self;

    /// Creates an owning [`StaticStorage`]. When its lifetime ends the underlying resources will
    /// be removed.
    fn create(self, contents: &[u8]) -> Result<T, StaticStorageCreateError> {
        let locked_storage = self.create_locked()?;

        Ok(
            fail!(from "StaticStorageBuilder::create", when locked_storage.unlock(contents),
            with StaticStorageCreateError::Write,
            "Unable to unlock static storage with content"),
        )
    }

    /// Creates an owning [`StaticStorageLocked`]. When its lifetime ends the underlying resource
    /// will be removed. The contents must be set later with [`StaticStorageLocked::unlock()`].
    /// This is useful if the static storage name should be reserved and initialized later.
    fn create_locked(self) -> Result<T::Locked, StaticStorageCreateError>;

    /// Opens an already existing [`StaticStorage`]. If the creation of the [`StaticStorage`] is
    /// not finalized it shall return an error.
    /// The provided defines how long the [`StaticStorageBuilder`]
    /// shall wait for [`StaticStorageBuilder::create_locked()`]
    /// to finalize the initialization and unlock the storage.
    fn open(self, timeout: Duration) -> Result<T, StaticStorageOpenError>;
}

/// A locked (uninitialized) static storage which is present but without content
pub trait StaticStorageLocked<T: StaticStorage>: Sized + NamedConcept {
    /// Unlocks the static storage by writing the contents to it
    fn unlock(self, contents: &[u8]) -> Result<T, StaticStorageUnlockError>;
}

/// A static storage which owns its underlying resources. When it goes out of scope those resources
/// shall be removed.
pub trait StaticStorage: Debug + Sized + NamedConceptMgmt + NamedConcept + Send + Sync {
    type Builder: StaticStorageBuilder<Self> + NamedConceptBuilder<Self>;
    type Locked: StaticStorageLocked<Self>;

    /// Returns the length of the content. Required to provide a buffer in
    /// [`StaticStorage::read()`] which is large enough.
    fn len(&self) -> u64;

    /// Returns true if it does not contain any content, otherwise false.
    fn is_empty(&self) -> bool;

    /// Writes the contents of the [`StaticStorage`] into the provided content buffer. If the
    /// buffer is too small an error must be returned.
    fn read(&self, content: &mut [u8]) -> Result<(), StaticStorageReadError>;

    /// Releases the ownership of the static storage. When the object goes out of scope the
    /// static storage is no longer removed.
    fn release_ownership(&self);

    /// Acquires the ownership of the static storage. If the object goes out of scope the
    /// underlying resources are removed.
    fn acquire_ownership(&self);

    /// The default suffix of every static storage
    fn default_suffix() -> FileName {
        unsafe { FileName::new_unchecked(b".static_storage") }
    }
}
