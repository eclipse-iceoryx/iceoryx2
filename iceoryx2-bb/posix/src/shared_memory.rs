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

//! Provides access to a POSIX [`SharedMemory`]Object used to share memory between processes.
//!
//! # Important
//!
//! When constructing objects into the memory one MUST ensure that the memory representation is
//! identical in every process. Therefore, it is important to add `#[repr(C)]` to the struct. If
//! this struct is a composite every member must have `#[repr(C)]` enabled.
//!
//! # Examples
//!
//! ## Create non-existing shared memory.
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::shared_memory::*;
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::*;
//!
//! let name = FileName::new(b"someShmName").unwrap();
//! let mut shm = SharedMemoryBuilder::new(&name)
//!                     .is_memory_locked(false)
//!           // the SharedMemoryCreationBuilder is used from here on
//!                     .creation_mode(CreationMode::PurgeAndCreate)
//!                     .size(1024)
//!                     .permission(Permission::OWNER_ALL)
//!                     .zero_memory(true)
//!                     .create()
//!                     .expect("failed to create shared memory");
//!
//! println!("shm name: {}", shm.name());
//! println!("shm addr: {:?}", shm.base_address());
//! println!("shm size: {}", shm.size());
//!
//! // set the first byte of the shared memory
//! shm.as_mut_slice()[0] = 0xFF;
//! ```
//!
//! ## Open existing shared memory.
//!
//! ```no_run
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::shared_memory::*;
//! use iceoryx2_bb_system_types::file_name::FileName;
//! use iceoryx2_bb_container::semantic_string::*;
//!
//! let name = FileName::new(b"someShmName").unwrap();
//! let shm = SharedMemoryBuilder::new(&name)
//!                     .is_memory_locked(false)
//!                     .open_existing(AccessMode::Read)
//!                     .expect("failed to open shared memory");
//!
//! // print the first byte of the shared memory
//! println!("first byte: {}", shm.as_slice()[0]);
//! ```

use core::ptr::NonNull;

use alloc::vec;
use alloc::vec::Vec;

use iceoryx2_bb_concurrency::atomic::AtomicBool;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_system_types::file_name::*;
use iceoryx2_bb_system_types::file_path::*;
use iceoryx2_bb_system_types::path::*;
use iceoryx2_log::{error, fail, fatal_panic, trace};
use iceoryx2_pal_configuration::PATH_SEPARATOR;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY;
use iceoryx2_pal_posix::*;

pub use crate::access_mode::AccessMode;
pub use crate::creation_mode::CreationMode;
use crate::file::{FileStatError, FileTruncateError};
use crate::file_descriptor::*;
use crate::memory_lock::{MemoryLock, MemoryLockCreationError};
use crate::memory_mapping::{
    MappingBehavior, MemoryMapping, MemoryMappingBuilder, MemoryMappingCreationError,
};
pub use crate::permission::Permission;
use crate::signal::SignalHandler;
use crate::system_configuration::Limit;

enum_gen! { SharedMemoryCreationError
  entry:
    SizeDoesNotFit,
    InsufficientMemory,
    InsufficientMemoryToBeMemoryLocked,
    UnsupportedSizeOfZero,
    InsufficientPermissions,
    MappedRegionLimitReached,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    NameTooLong,
    InvalidName,
    AlreadyExist,
    DoesNotExist,
    UnknownError(i32)
  mapping:
    FileTruncateError,
    FileStatError,
    MemoryLockCreationError,
    SharedMemoryRemoveError,
    MemoryMappingCreationError
}

enum_gen! { SharedMemoryRemoveError
  entry:
    InsufficientPermissions,
    UnknownError(i32)
}

/// The builder for the [`SharedMemory`].
#[derive(Debug)]
pub struct SharedMemoryBuilder {
    name: FileName,
    size: usize,
    is_memory_locked: bool,
    has_ownership: bool,
    permission: Permission,
    creation_mode: Option<CreationMode>,
    zero_memory: bool,
    access_mode: AccessMode,
    mapping_offset: isize,
    enforce_base_address: Option<u64>,
}

impl SharedMemoryBuilder {
    pub fn new(name: &FileName) -> Self {
        SharedMemoryBuilder {
            name: *name,
            size: 0,
            is_memory_locked: false,
            permission: Permission::OWNER_ALL,
            access_mode: AccessMode::None,
            has_ownership: true,
            creation_mode: None,
            zero_memory: true,
            mapping_offset: 0,
            enforce_base_address: None,
        }
    }

    /// Defines the mapping offset when the shared memory object is mapped into
    /// the process space.
    pub fn mapping_offset(mut self, value: isize) -> Self {
        self.mapping_offset = value;
        self
    }

    /// Locks the shared memory into the heap. If this is enabled swapping of the
    /// created shared memory segment is no longer possible.
    pub fn is_memory_locked(mut self, value: bool) -> Self {
        self.is_memory_locked = value;
        self
    }

    /// Sets a base address for the shared memory which is enforced. When the shared memory
    /// could not mapped at the provided address the creation fails.
    pub fn enforce_base_address(mut self, value: u64) -> Self {
        self.enforce_base_address = Some(value);
        self
    }

    /// Opens an already existing shared memory.
    pub fn open_existing(
        mut self,
        access_mode: AccessMode,
    ) -> Result<SharedMemory, SharedMemoryCreationError> {
        self.access_mode = access_mode;
        Self::open(self)
    }

    fn create_memory_mapping(
        file_descriptor: FileDescriptor,
        config: &SharedMemoryBuilder,
    ) -> Result<MemoryMapping, SharedMemoryCreationError> {
        match MemoryMappingBuilder::from_file_descriptor(file_descriptor)
            .mapping_behavior(MappingBehavior::Shared)
            .initial_mapping_permission(config.access_mode.into())
            .mapping_address_hint(config.enforce_base_address.unwrap_or(0) as usize)
            .enforce_mapping_address_hint(config.enforce_base_address.is_some())
            .offset(config.mapping_offset)
            .size(config.size)
            .create()
        {
            Ok(mapping) => Ok(mapping),
            Err(e) => {
                fail!(from config, with e.into(),
                        "Failed to create shared memory since the memory mapping failed ({e:?}).");
            }
        }
    }

    fn open(mut self) -> Result<SharedMemory, SharedMemoryCreationError> {
        let msg = "Unable to open shared memory";
        let fd = SharedMemory::shm_open(&self.name, &self)?;

        let actual_shm_size = fail!(from self, when fd.metadata(),
                "{} since a failure occurred while acquiring the file attributes.", msg)
        .size();
        self.size = actual_shm_size as usize;

        let memory_mapping = Self::create_memory_mapping(fd, &self)?;

        let shm = SharedMemory {
            name: self.name,
            has_ownership: AtomicBool::new(false),
            memory_lock: None,
            memory_mapping,
            mapping_offset: self.mapping_offset,
        };

        trace!(from shm, "open");
        Ok(shm)
    }

    /// Creates a new shared memory segment.
    pub fn creation_mode(mut self, creation_mode: CreationMode) -> SharedMemoryCreationBuilder {
        self.access_mode = AccessMode::ReadWrite;
        self.creation_mode = Some(creation_mode);
        SharedMemoryCreationBuilder { config: self }
    }
}

#[derive(Debug)]
pub struct SharedMemoryCreationBuilder {
    config: SharedMemoryBuilder,
}

impl SharedMemoryCreationBuilder {
    /// Sets the permissions of the new shared memory
    pub fn permission(mut self, value: Permission) -> Self {
        self.config.permission = value;
        self
    }

    /// Zero the memory of the shared memory. It can serve to purposes.
    /// * Ensure that the memory is clean before using it.
    /// * Ensure that enough memory is actually available. On some operating systems the memory is
    ///   only virtually allocated and when it is later required but there is not enough memory
    ///   left the application fails.
    pub fn zero_memory(mut self, value: bool) -> Self {
        self.config.zero_memory = value;
        self
    }

    /// The size of the shared memory.
    pub fn size(mut self, size: usize) -> Self {
        self.config.size = size;
        self
    }

    /// Defines if a newly created [`SharedMemory`] owns the underlying resources. If they are not
    /// owned they will not be cleaned up and can be opened later but they need to be explicitly
    /// removed.
    pub fn has_ownership(mut self, value: bool) -> Self {
        self.config.has_ownership = value;
        self
    }

    /// Creates the shared memory segment.
    pub fn create(mut self) -> Result<SharedMemory, SharedMemoryCreationError> {
        let msg = "Unable to create shared memory";

        if self.config.size == 0 {
            fail!(from self.config, with SharedMemoryCreationError::UnsupportedSizeOfZero,
                "{msg} since a size of 0 is not supported for a shared memory object.");
        }

        let shm_created;
        let mut fd = match self
            .config
            .creation_mode
            .expect("CreationMode must be set on creation")
        {
            CreationMode::CreateExclusive => {
                shm_created = true;
                SharedMemory::shm_create(&self.config.name, &self.config)?
            }
            CreationMode::PurgeAndCreate => {
                shm_created = true;
                fail!(from self.config, when SharedMemory::shm_unlink(&self.config.name),
                    "Failed to remove already existing shared memory.");
                SharedMemory::shm_create(&self.config.name, &self.config)?
            }
            CreationMode::OpenOrCreate => {
                match SharedMemory::shm_open(&self.config.name, &self.config) {
                    Ok(fd) => {
                        shm_created = false;
                        self.config.has_ownership = false;
                        fd
                    }
                    Err(SharedMemoryCreationError::DoesNotExist) => {
                        shm_created = true;
                        match SharedMemory::shm_create(&self.config.name, &self.config) {
                            Ok(fd) => fd,
                            Err(SharedMemoryCreationError::AlreadyExist) => {
                                SharedMemory::shm_open(&self.config.name, &self.config)?
                            }
                            Err(e) => return Err(e),
                        }
                    }
                    Err(v) => return Err(v),
                }
            }
        };

        if !shm_created {
            let actual_shm_size = fail!(from self.config, when fd.metadata(),
                    "{} since a failure occurred while acquiring the file attributes.", msg)
            .size();
            if self.config.size > actual_shm_size as usize {
                fail!(from self.config, with SharedMemoryCreationError::SizeDoesNotFit,
                    "{} since the actual size {} is not equal to the configured size {}.", msg, actual_shm_size, self.config.size);
            }

            self.config.size = actual_shm_size as _;
            let memory_mapping = SharedMemoryBuilder::create_memory_mapping(fd, &self.config)?;

            let shm = SharedMemory {
                name: self.config.name,
                has_ownership: AtomicBool::new(self.config.has_ownership),
                memory_lock: None,
                memory_mapping,
                mapping_offset: self.config.mapping_offset,
            };

            trace!(from shm, "open");
            return Ok(shm);
        }

        fail!(from self.config, when fd.truncate(self.config.size), "{} since the shared memory truncation failed.", msg);

        let actual_shm_size = fail!(from self.config, when fd.metadata(),
                "{} since a failure occurred while acquiring the file attributes.", msg)
        .size();
        if (actual_shm_size as usize) < self.config.size {
            fail!(from self.config, with SharedMemoryCreationError::SizeDoesNotFit,
                "{} since the actual size {} is less than to the configured size {}.", msg, actual_shm_size, self.config.size);
        }

        self.config.size = actual_shm_size as _;
        let memory_mapping = SharedMemoryBuilder::create_memory_mapping(fd, &self.config)?;

        let mut shm = SharedMemory {
            name: self.config.name,
            has_ownership: AtomicBool::new(self.config.has_ownership),
            memory_lock: None,
            memory_mapping,
            mapping_offset: self.config.mapping_offset,
        };

        if self.config.is_memory_locked {
            shm.memory_lock = Some(
                fail!(from self.config, when unsafe { MemoryLock::new(shm.memory_mapping.base_address().cast(), shm.memory_mapping.size()) },
                        "{} since the memory lock failed.", msg),
            )
        }

        if self.config.zero_memory {
            if POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING {
                let memset_call = || unsafe {
                    posix::memset(
                        shm.memory_mapping.base_address_mut().cast(),
                        0,
                        self.config.size,
                    );
                };
                match SignalHandler::call_and_fetch(memset_call) {
                    None => (),
                    Some(v) => {
                        fail!(from self.config, with SharedMemoryCreationError::InsufficientMemory,
                            "{} since a signal {} was raised while zeroing the memory. Is enough memory available on the system?", msg, v);
                    }
                }
            } else {
                unsafe {
                    posix::memset(
                        shm.memory_mapping.base_address_mut().cast(),
                        0,
                        self.config.size,
                    )
                };
            }
        }

        trace!(from shm, "create");
        Ok(shm)
    }
}

/// A POSIX shared memory object which is build by the [`SharedMemoryBuilder`].
#[derive(Debug)]
pub struct SharedMemory {
    name: FileName,
    has_ownership: AtomicBool,
    memory_mapping: MemoryMapping,
    memory_lock: Option<MemoryLock>,
    mapping_offset: isize,
}

impl Drop for SharedMemory {
    fn drop(&mut self) {
        if self.has_ownership() {
            match self.set_permission(Permission::OWNER_ALL) {
                Ok(()) => match Self::shm_unlink(&self.name) {
                    Ok(_) => {
                        trace!(from self, "delete");
                    }
                    Err(_) => {
                        error!(from self, "Failed to cleanup shared memory.");
                    }
                },
                Err(e) => {
                    error!(from self, "Failed to cleanup shared memory since the permissions could not be adjusted ({:?}).", e);
                }
            }
        }
    }
}

impl SharedMemory {
    /// Returns true if the shared memory exists and is accessible, otherwise false.
    pub fn does_exist(name: &FileName) -> bool {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        FileDescriptor::new(unsafe {
            posix::shm_open(
                file_path.as_c_str(),
                AccessMode::Read.as_oflag(),
                Permission::none().as_mode(),
            )
        })
        .is_some()
    }

    /// Returns the mapping offset used when the shared memory object was mapped into process space
    pub fn mapping_offset(&self) -> isize {
        self.mapping_offset
    }

    /// Returns if the posix implementation supports persistent shared memory, meaning that when every
    /// shared memory handle got out of scope the underlying OS resource remains.
    pub fn does_support_persistency() -> bool {
        POSIX_SUPPORT_PERSISTENT_SHARED_MEMORY
    }

    /// Returns true if the shared memory object has the ownership of the underlying posix shared
    /// memory. Ownership implies hereby that the posix shared memory is removed as soon as this
    /// object goes out of scope.
    pub fn has_ownership(&self) -> bool {
        self.has_ownership.load(Ordering::Relaxed)
    }

    /// Releases the ownership of the underlying posix shared memory. If the object goes out of
    /// scope the shared memory is no longer removed.
    pub fn release_ownership(&self) {
        self.has_ownership.store(false, Ordering::Relaxed)
    }

    /// Acquires the ownership of the underlying posix shared memory. If the object goes out of
    /// scope the shared memory will be removed.
    pub fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed)
    }

    /// Removes a shared memory file.
    pub fn remove(name: &FileName) -> Result<bool, SharedMemoryRemoveError> {
        match Self::shm_unlink(name) {
            Ok(true) => {
                trace!(from "SharedMemory::remove", "\"{}\"", name);
                Ok(true)
            }
            Ok(false) => Ok(false),
            Err(v) => Err(v),
        }
    }

    /// Returns a list of all shared memory objects
    pub fn list() -> Vec<FileName> {
        let mut result = vec![];

        let raw_shm_names = unsafe { posix::shm_list() };
        for name in &raw_shm_names {
            if let Ok(f) = unsafe { FileName::from_c_str(name.as_ptr() as *mut _) } {
                result.push(f)
            }
        }

        result
    }

    /// returns the name of the shared memory
    pub fn name(&self) -> &FileName {
        &self.name
    }

    /// returns the base address of the shared memory. The base address is always aligned to the
    /// page size, this implies that it is aligned with every possible type.
    /// No further alignment required!
    pub fn base_address(&self) -> NonNull<u8> {
        match NonNull::new(self.memory_mapping.base_address().cast_mut()) {
            Some(v) => v,
            None => {
                fatal_panic!(from self,
                    "This should never happen! A valid shared memory object should never contain a base address with null value.");
            }
        }
    }

    /// returns the size of the shared memory
    pub fn size(&self) -> usize {
        self.memory_mapping.size()
    }

    /// returns a slice to the memory
    pub fn as_slice(&self) -> &[u8] {
        self.memory_mapping.as_slice()
    }

    /// returns a mutable slice to the memory
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.memory_mapping.as_mut_slice()
    }

    fn shm_create(
        name: &FileName,
        config: &SharedMemoryBuilder,
    ) -> Result<FileDescriptor, SharedMemoryCreationError> {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        let fd = FileDescriptor::new(unsafe {
            posix::shm_open(
                file_path.as_c_str(),
                CreationMode::CreateExclusive.as_oflag() | config.access_mode.as_oflag(),
                config.permission.as_mode(),
            )
        });

        if let Some(v) = fd {
            return Ok(v);
        }

        let msg = "Unable to create shared memory";
        handle_errno!(SharedMemoryCreationError, from config,
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EINVAL => (InvalidName, "{} since the provided name \"{}\" is invalid.", msg, name),
            Errno::EEXIST => (AlreadyExist, "{} since it already exists.", msg),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process file handle limit was reached.", msg),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide file handle limit was reached.", msg),
            Errno::ENAMETOOLONG => (NameTooLong, "{} since the name exceeds the maximum supported length of {}.", msg, Limit::MaxFileNameLength.value() ),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn shm_open(
        name: &FileName,
        config: &SharedMemoryBuilder,
    ) -> Result<FileDescriptor, SharedMemoryCreationError> {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        let fd = FileDescriptor::new(unsafe {
            posix::shm_open(
                file_path.as_c_str(),
                config.access_mode.as_oflag(),
                Permission::none().as_mode(),
            )
        });

        if let Some(v) = fd {
            return Ok(v);
        }

        let msg = "Unable to open shared memory";
        handle_errno!(SharedMemoryCreationError, from config,
            Errno::ENOENT => (DoesNotExist, "{} since the shared memory does not exist.", msg),
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EINVAL => (InvalidName, "{} since the provided name \"{}\" is invalid.", msg, name),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process file handle limit was reached.", msg),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide file handle limit was reached.", msg),
            Errno::ENAMETOOLONG => (NameTooLong, "{} since the name exceeds the maximum supported length of {}.", msg, Limit::MaxFileNameLength.value() ),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn shm_unlink(name: &FileName) -> Result<bool, SharedMemoryRemoveError> {
        let file_path =
            FilePath::from_path_and_file(&Path::new(&[PATH_SEPARATOR; 1]).unwrap(), name).unwrap();
        if unsafe { posix::shm_unlink(file_path.as_c_str()) } == 0 {
            return Ok(true);
        }

        let msg = "Unable to remove shared memory device file";
        let origin = "SharedMemory::unlink()";
        match posix::Errno::get() {
            posix::Errno::EACCES => {
                fail!(from origin, with SharedMemoryRemoveError::InsufficientPermissions,
                    "{} \"{}\" due to insufficient permissions.", msg, name);
            }
            posix::Errno::ENOENT => Ok(false),
            v => {
                fail!(from origin, with SharedMemoryRemoveError::UnknownError(v as i32),
                    "{} \"{}\" since an unknown error occurred ({}).", msg, name, v);
            }
        }
    }
}

impl FileDescriptorBased for SharedMemory {
    fn file_descriptor(&self) -> &FileDescriptor {
        self.memory_mapping
            .file_descriptor()
            .as_ref()
            .expect("Memory mapping is based on a file descriptor")
    }
}

impl FileDescriptorManagement for SharedMemory {}
