// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

//! Safe abstraction of the POSIX calls `mmap`, `munmap` and `mprotect`.
//!
//! # Examples
//!
//! ## Mapping Anonymous Memory (`malloc` equivalent)
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_log::set_log_level;
//! use iceoryx2_bb_posix::memory_mapping::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let mut mmap = MemoryMappingBuilder::from_anonymous()
//!     .initial_mapping_permission(MappingPermission::ReadWrite)
//!     .size(65536)
//!     .create()?;
//!
//! // access memory
//! let some_ptr = mmap.base_address_mut();
//! # Ok(())
//! # }
//! ```
//!
//! ## Opening A File And Mapping It Into Memory
//!
//! ```no_run
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::memory_mapping::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let mmap = MemoryMappingBuilder::from_file(&FilePath::new(b"some_mapable_file")?)
//!     .file_access_mode(AccessMode::ReadWrite)
//!     .mapping_behavior(MappingBehavior::Shared)
//!     .initial_mapping_permission(MappingPermission::ReadWrite)
//!     .size(65536)
//!     .create()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Mapping [`FileDescriptor`] Contents Into Memory
//!
//! ```no_run
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::memory_mapping::*;
//! use iceoryx2_bb_posix::file_descriptor::FileDescriptor;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! # let file_descriptor = FileDescriptor::new(12).unwrap();
//! let mmap = MemoryMappingBuilder::from_file_descriptor(file_descriptor)
//!     .mapping_behavior(MappingBehavior::Private)
//!     .initial_mapping_permission(MappingPermission::ReadWrite)
//!     .size(65536)
//!     .create()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Update Permissions To Mapped Memory (`mprotect`)
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::memory_mapping::*;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//! let mut mmap = MemoryMappingBuilder::from_anonymous()
//!     .initial_mapping_permission(MappingPermission::ReadWrite)
//!     .size(65536)
//!     .create()?;
//!
//! // update permission from 0 to 32768
//! mmap.set_permission(0)
//!     .region_size(32768)
//!     .apply(MappingPermission::Read)?;
//!
//! # Ok(())
//! # }
//! ```

pub use crate::access_mode::AccessMode;
pub use iceoryx2_bb_container::semantic_string::SemanticString;
pub use iceoryx2_bb_system_types::file_path::FilePath;

use crate::{file_descriptor::FileDescriptor, system_configuration::SystemInfo};
use iceoryx2_log::{fail, fatal_panic, trace};
use iceoryx2_pal_posix::posix::{self, Errno, MAP_FAILED};

/// Error that can occur when a new [`MemoryMapping`] is created with [`MemoryMappingBuilder::create()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryMappingCreationError {
    /// Insufficient permissions to open the file or map the mapping.
    InsufficientPermissions,
    /// The size was not set, or set to zero.
    MappingSizeIsZero,
    /// Exceeds the limit of mapped regions.
    ExceedsTheMaximumNumberOfMappedRegions,
    /// A [`FileDescriptor`] was provided that does not support `mmap`.
    FileDescriptorDoesNotSupportMemoryMappings,
    /// Insufficient resources to map the memory into the process.
    InsufficientResources,
    /// The provided size was larger than the corresponding file.
    MappingLargerThanCorrespondingFile,
    /// An address hint was provided but it could not be enforced.
    FailedToEnforceAddressHint,
    /// The corresponding device does not support synchronized IO
    SynchronizedIONotSupported,
    /// An interrupt signal was raised.
    InterruptSignal,
    /// The provided [`FilePath`] is actually a [`Directory`](crate::directory::Directory).
    FileIsADirectory,
    /// There are too many symbolic links in the provided [`FilePath`].
    TooManySymbolicLinksInPath,
    /// The process wide limit of [`FileDescriptor`]s was reached.
    PerProcessFileHandleLimitReached,
    /// The system wide limit of [`FileDescriptor`]s was reached.
    SystemWideFileHandleLimitReached,
    /// The file does not exist.
    FileDoesNotExist,
    /// The file is larger than what `off_t` can represent.
    FileTooBig,
    /// The POSIX `open` call returned a [`FileDescriptor`] that is not valid. Should never happen!
    OpenReturnedBrokedFileDescriptor,
    /// An unknown failure occurred.
    UnknownFailure(i32),
}

impl core::fmt::Display for MemoryMappingCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MemoryMappingCreationError::{self:?}")
    }
}

impl core::error::Error for MemoryMappingCreationError {}

/// Error that can occur when the [`MappingPermission`] is updated with
/// [`MemoryMapping::set_permission()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryMappingPermissionUpdateError {
    /// The size was not a multiple of the [`SystemInfo::PageSize`]
    SizeNotAlignedToPageSize,
    /// The address offset was not a multiple of the [`SystemInfo::PageSize`]
    RegionOffsetNotAlignedToPageSize,
    /// Insufficient permissions to update the [`MappingPermission`]
    InsufficientPermissions,
    /// Insufficient memory to update the [`MappingPermission`]
    InsufficientMemory,
    /// The size and address offset are larger than the mapped memory range
    InvalidAddressRange,
    /// The size was either not set or set to zero.
    RegionSizeIsZero,
    /// An unknown failure occurred.
    UnknownFailure(i32),
}

impl core::fmt::Display for MemoryMappingPermissionUpdateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "MemoryMappingPermissionUpdateError::{self:?}")
    }
}

impl core::error::Error for MemoryMappingPermissionUpdateError {}

/// Defines the access permission of the process to the [`MemoryMapping`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MappingPermission {
    /// The process can only read, writing memory will terminate the process.
    Read = posix::PROT_READ,
    /// The process can only write, reading memory will terminate the process.
    Write = posix::PROT_WRITE,
    /// The process can read and write
    ReadWrite = posix::PROT_READ | posix::PROT_WRITE,
    /// The process can execute the contents of the memory
    Exec = posix::PROT_EXEC,
    /// Read and execute the contents
    ReadExec = posix::PROT_READ | posix::PROT_EXEC,
    /// Write and execute the contents
    WriteExec = posix::PROT_WRITE | posix::PROT_EXEC,
    /// Provide all permissions to the mapping
    ReadWriteExec = posix::PROT_READ | posix::PROT_WRITE | posix::PROT_EXEC,
    /// Just map the memory but do not grant any access permission
    None = posix::PROT_NONE,
}

impl From<AccessMode> for MappingPermission {
    fn from(value: AccessMode) -> Self {
        match value {
            AccessMode::None => MappingPermission::None,
            AccessMode::Read => MappingPermission::Read,
            AccessMode::Write => MappingPermission::Write,
            AccessMode::ReadWrite => MappingPermission::ReadWrite,
        }
    }
}

/// Defines the memory synchronization behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MappingBehavior {
    /// Memory changes are written back and are visible to other processes
    Shared = posix::MAP_SHARED,
    /// Changes are local to the process.
    Private = posix::MAP_PRIVATE,
}

#[allow(clippy::large_enum_variant)] // `Box` is not allowed in a mission-critical context
#[derive(Debug)]
enum MappingOrigin {
    FileDescriptor(FileDescriptor),
    File((FilePath, AccessMode)),
    Anonymous,
}

#[derive(Debug)]
struct MemoryMappingBuilderSettings {
    mapping_permission: MappingPermission,
    mapping_behavior: MappingBehavior,
    address_hint: usize,
    size: usize,
    offset: isize,
    enforce_address_hint: bool,
}

impl MemoryMappingBuilderSettings {
    fn new() -> Self {
        Self {
            mapping_behavior: MappingBehavior::Private,
            mapping_permission: MappingPermission::None,
            address_hint: 0,
            enforce_address_hint: false,
            offset: 0,
            size: 0,
        }
    }
}

/// Helper struct to create a new [`MemoryMapping`] based on an existing
/// file.
pub struct FileMappingBuilder {
    file_path: FilePath,
    access_mode: AccessMode,
    settings: MemoryMappingBuilderSettings,
}

impl FileMappingBuilder {
    /// Defines the [`AccessMode`] under which the file shall be opened.
    pub fn file_access_mode(mut self, access_mode: AccessMode) -> Self {
        self.access_mode = access_mode;
        self
    }

    /// Defines the [`MappingBehavior`] under which the memory shall be mapped
    /// into the process space.
    pub fn mapping_behavior(mut self, value: MappingBehavior) -> MemoryMappingBuilder {
        self.settings.mapping_behavior = value;
        MemoryMappingBuilder {
            settings: self.settings,
            origin: MappingOrigin::File((self.file_path, self.access_mode)),
        }
    }
}

/// Helper struct to create a new [`MemoryMapping`] based on an existing
/// [`FileDescriptor`].
pub struct FileDescriptorMappingBuilder {
    file_descriptor: FileDescriptor,
    settings: MemoryMappingBuilderSettings,
}

impl FileDescriptorMappingBuilder {
    /// Defines the [`MappingBehavior`] under which the memory shall be mapped
    /// into the process space.
    pub fn mapping_behavior(mut self, value: MappingBehavior) -> MemoryMappingBuilder {
        self.settings.mapping_behavior = value;
        MemoryMappingBuilder {
            settings: self.settings,
            origin: MappingOrigin::FileDescriptor(self.file_descriptor),
        }
    }
}

/// Builder to create a new [`MemoryMapping`].
#[derive(Debug)]
pub struct MemoryMappingBuilder {
    settings: MemoryMappingBuilderSettings,
    origin: MappingOrigin,
}

impl MemoryMappingBuilder {
    /// Creates an anonymous [`MemoryMapping`]. Is equivalent to `malloc`
    pub fn from_anonymous() -> Self {
        Self {
            settings: MemoryMappingBuilderSettings::new(),
            origin: MappingOrigin::Anonymous,
        }
    }

    /// Opens an existing file and maps its content into the process space
    pub fn from_file(file_path: &FilePath) -> FileMappingBuilder {
        FileMappingBuilder {
            file_path: *file_path,
            access_mode: AccessMode::None,
            settings: MemoryMappingBuilderSettings::new(),
        }
    }

    /// Maps the contents of the file descriptor into the process space
    pub fn from_file_descriptor(file_descriptor: FileDescriptor) -> FileDescriptorMappingBuilder {
        FileDescriptorMappingBuilder {
            file_descriptor,
            settings: MemoryMappingBuilderSettings::new(),
        }
    }

    /// Defines the initial [`MappingPermission`]s that shall be used for the
    /// [`MemoryMapping`]
    pub fn initial_mapping_permission(mut self, value: MappingPermission) -> Self {
        self.settings.mapping_permission = value;
        self
    }

    /// Provides an optional address hint to which the memory shall be mapped to.
    /// It is just a hint and the operating system is allowed to use a different
    /// address if it is not available. With
    /// [`MemoryMappingBuilder::enforce_mapping_address_hint()`] it will be enforced.
    pub fn mapping_address_hint(mut self, value: usize) -> Self {
        self.settings.address_hint = value;
        self
    }

    /// Tries to enforce the provided address hint. If it is not possible to enforce
    /// it, the creation of the [`MemoryMapping`] will fail.
    pub fn enforce_mapping_address_hint(mut self, value: bool) -> Self {
        self.settings.enforce_address_hint = value;
        self
    }

    /// Defines the size of the [`MemoryMapping`]
    pub fn size(mut self, value: usize) -> Self {
        self.settings.size = value;
        self
    }

    /// Defines the optional offset for the [`MemoryMapping`]
    pub fn offset(mut self, value: isize) -> Self {
        self.settings.offset = value;
        self
    }

    /// Tries to create a new [`MemoryMapping`].
    pub fn create(self) -> Result<MemoryMapping, MemoryMappingCreationError> {
        match self.origin {
            MappingOrigin::Anonymous => Self::create_mapping(
                &self.settings,
                posix::MAP_ANONYMOUS | posix::MAP_PRIVATE,
                None,
                None,
            ),
            MappingOrigin::FileDescriptor(file_descriptor) => Self::create_mapping(
                &self.settings,
                self.settings.mapping_behavior as _,
                Some(file_descriptor),
                None,
            ),
            MappingOrigin::File((ref file_path, access_mode)) => {
                let msg = "Unable to create memory mapping since the corresponding file could not be opened";
                let mem_fd = unsafe {
                    posix::open(file_path.as_c_str(), access_mode.as_oflag() | posix::O_SYNC)
                };

                if mem_fd == -1 {
                    match Errno::get() {
                        Errno::EACCES => {
                            fail!(from self, with MemoryMappingCreationError::InsufficientPermissions,
                                "{msg} due to insufficient permissions.");
                        }
                        Errno::EINTR => {
                            fail!(from self, with MemoryMappingCreationError::InterruptSignal,
                                "{msg} since an interrupt signal was raised.");
                        }
                        Errno::EINVAL => {
                            fail!(from self, with MemoryMappingCreationError::SynchronizedIONotSupported,
                                "{msg} since the file does not support synchronized IO.");
                        }
                        Errno::EIO => {
                            fail!(from self, with MemoryMappingCreationError::FileDescriptorDoesNotSupportMemoryMappings,
                                "{msg} since the file does not support memory mappings.");
                        }
                        Errno::EISDIR => {
                            fail!(from self, with MemoryMappingCreationError::FileIsADirectory,
                                "{msg} since the file is a directory.");
                        }
                        Errno::ELOOP => {
                            fail!(from self, with MemoryMappingCreationError::TooManySymbolicLinksInPath,
                                "{msg} since there are too many symbolic links in the path.");
                        }
                        Errno::EMFILE => {
                            fail!(from self, with MemoryMappingCreationError::PerProcessFileHandleLimitReached,
                                "{msg} since the process wide file handle limit was reached.");
                        }
                        Errno::ENFILE => {
                            fail!(from self, with MemoryMappingCreationError::SystemWideFileHandleLimitReached,
                                "{msg} since the system wide file handle limit was reached.");
                        }
                        Errno::ENOENT => {
                            fail!(from self, with MemoryMappingCreationError::FileDoesNotExist,
                                "{msg} since the file does not exist.");
                        }
                        Errno::EOVERFLOW => {
                            fail!(from self, with MemoryMappingCreationError::FileTooBig,
                                "{msg} since the file is too big.");
                        }
                        Errno::ENOMEM => {
                            fail!(from self, with MemoryMappingCreationError::InsufficientResources,
                                "{msg} due to insufficient resources.");
                        }
                        e => {
                            fail!(from self, with MemoryMappingCreationError::UnknownFailure(e as i32),
                                "{msg} due to an unknown failure ({e:?}).");
                        }
                    }
                }

                let fd = match FileDescriptor::new(mem_fd) {
                    Some(fd) => fd,
                    None => {
                        fail!(from self, with MemoryMappingCreationError::OpenReturnedBrokedFileDescriptor,
                            "{msg} since open returned a broken file descriptor.");
                    }
                };

                trace!(from self, "opened file");

                Self::create_mapping(
                    &self.settings,
                    self.settings.mapping_behavior as _,
                    Some(fd),
                    Some(*file_path),
                )
            }
        }
    }

    fn create_mapping(
        settings: &MemoryMappingBuilderSettings,
        mapping_behavior: i32,
        file_descriptor: Option<FileDescriptor>,
        file_path: Option<FilePath>,
    ) -> Result<MemoryMapping, MemoryMappingCreationError> {
        let fd_value = if let Some(fd) = &file_descriptor {
            unsafe { fd.native_handle() }
        } else {
            -1
        };
        let msg = "Failed to create memory mapping";
        if settings.size == 0 {
            fail!(from settings, with MemoryMappingCreationError::MappingSizeIsZero,
                "{msg} since the size must be greater than 0.");
        }

        let ret_val = unsafe {
            posix::mmap(
                settings.address_hint as *mut posix::void,
                settings.size,
                settings.mapping_permission as _,
                mapping_behavior,
                fd_value,
                settings.offset as _,
            )
        };

        if ret_val == MAP_FAILED {
            match Errno::get() {
                Errno::EACCES => {
                    fail!(from settings, with MemoryMappingCreationError::InsufficientPermissions,
                        "{msg} due to insufficient permissions.");
                }
                Errno::EMFILE => {
                    fail!(from settings, with MemoryMappingCreationError::ExceedsTheMaximumNumberOfMappedRegions,
                        "{msg} since it would exceed the maximum supported number of memory mappings.");
                }
                Errno::ENODEV => {
                    fail!(from settings, with MemoryMappingCreationError::FileDescriptorDoesNotSupportMemoryMappings,
                        "{msg} since the provided file or file descriptor do not support memory mappings.");
                }
                Errno::ENOMEM => {
                    fail!(from settings, with MemoryMappingCreationError::InsufficientResources,
                        "{msg} due to insufficient resources.");
                }
                Errno::EOVERFLOW => {
                    fail!(from settings, with MemoryMappingCreationError::MappingLargerThanCorrespondingFile,
                        "{msg} since the mapping size is larger than the corresponding file.");
                }
                e => {
                    fail!(from settings, with MemoryMappingCreationError::UnknownFailure(e as i32),
                        "{msg} due to an unknown failure ({e:?}).");
                }
            }
        }

        let mapping = MemoryMapping {
            file_descriptor,
            file_path,
            base_address: ret_val.cast(),
            size: settings.size,
            offset: settings.offset,
        };

        if settings.enforce_address_hint && ret_val as usize != settings.address_hint {
            fail!(from settings, with MemoryMappingCreationError::FailedToEnforceAddressHint,
                "{msg} since the address hint of {:#x?} could not be enforced.",
                settings.address_hint);
        }

        trace!(from mapping, "mapped");

        Ok(mapping)
    }
}

/// A memory mapping that was created with [`MemoryMappingBuilder`]. Abstraction
/// over `mmap`.
///
/// When it goes out of scope all resources are cleaned up.
#[derive(Debug)]
pub struct MemoryMapping {
    file_descriptor: Option<FileDescriptor>,
    file_path: Option<FilePath>,
    base_address: *mut u8,
    size: usize,
    offset: isize,
}

impl Drop for MemoryMapping {
    fn drop(&mut self) {
        if unsafe { posix::munmap(self.base_address.cast(), self.size) } == -1 {
            fatal_panic!(from self,
                "This should never happen! Unable to unmap a mapped memory region.");
        }
        trace!(from self, "removed");
    }
}

impl MemoryMapping {
    /// Updates the permissions of a region of the [`MemoryMapping`]. The start
    /// offset must be a multiple of page size.
    pub fn set_permission(&mut self, region_offset: usize) -> ProtectBuilder {
        ProtectBuilder {
            mapping_base_address: self.base_address as usize,
            mapping_size: self.size,
            region_size: 0,
            region_offset,
        }
    }

    /// Returns a [`FileDescriptor`] when it is not an anonymous [`MemoryMapping`].
    pub fn file_descriptor(&self) -> &Option<FileDescriptor> {
        &self.file_descriptor
    }

    /// Returns the const base address of the [`MemoryMapping`]
    pub fn base_address(&self) -> *const u8 {
        self.base_address
    }

    /// Returns the mutable base address of the [`MemoryMapping`]
    pub fn base_address_mut(&mut self) -> *mut u8 {
        self.base_address
    }

    /// Returns a const slice to the underlying memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.base_address(), self.size()) }
    }

    /// Returns a mutable slice to the underlying memory
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.base_address_mut(), self.size()) }
    }

    /// Returns the [`FilePath`] if the [`MemoryMapping`] was created from a
    /// file
    pub fn file_path(&self) -> &Option<FilePath> {
        &self.file_path
    }

    /// Returns the size of the [`MemoryMapping`]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the offset of the [`MemoryMapping`]
    pub fn offset(&self) -> isize {
        self.offset
    }
}

/// Helper struct to update the [`MappingPermission`]s of a part of the
/// [`MemoryMapping`].
#[derive(Debug)]
pub struct ProtectBuilder {
    mapping_base_address: usize,
    mapping_size: usize,
    region_size: usize,
    region_offset: usize,
}

impl ProtectBuilder {
    /// Defines the size of the memory range. Must be a multiple of the page size.
    pub fn region_size(mut self, value: usize) -> Self {
        self.region_size = value;
        self
    }

    /// Applies the defined [`MappingPermission`]s to the memory range.
    pub fn apply(
        self,
        mapping_permission: MappingPermission,
    ) -> Result<(), MemoryMappingPermissionUpdateError> {
        let msg = "Failed to adjust the permissions of the memory mapping";
        let page_size = SystemInfo::PageSize.value();
        if self.region_size % page_size != 0 {
            fail!(from self, with MemoryMappingPermissionUpdateError::SizeNotAlignedToPageSize,
                "{msg} since the region size is not aligned to the page size of {page_size}.");
        }

        if self.region_offset % page_size != 0 {
            fail!(from self, with MemoryMappingPermissionUpdateError::RegionOffsetNotAlignedToPageSize,
                "{msg} since the region offset {} is not aligned to the page size of {page_size}.",
                self.region_offset);
        }

        if self.region_offset >= self.mapping_size
            || self.mapping_size - self.region_offset < self.region_size
        {
            fail!(from self, with MemoryMappingPermissionUpdateError::InvalidAddressRange,
                "{msg} since it contains an address range outside of the mapped memory range.");
        }

        if self.region_size == 0 {
            fail!(from self, with MemoryMappingPermissionUpdateError::RegionSizeIsZero,
                "{msg} since the provided size is zero.");
        }

        if unsafe {
            posix::mprotect(
                (self.mapping_base_address + self.region_offset) as *mut posix::void,
                self.region_size,
                mapping_permission as _,
            )
        } == -1
        {
            match Errno::get() {
                Errno::EACCES => {
                    fail!(from self, with MemoryMappingPermissionUpdateError::InsufficientPermissions,
                        "{msg} due to insufficient permissions.");
                }
                Errno::EAGAIN | Errno::ENOMEM => {
                    fail!(from self, with MemoryMappingPermissionUpdateError::InsufficientMemory,
                        "{msg} due to insufficient memory.");
                }
                e => {
                    fail!(from self, with MemoryMappingPermissionUpdateError::UnknownFailure(e as _),
                        "{msg} due to an unknown failure ({e:?}).");
                }
            }
        }

        Ok(())
    }
}
