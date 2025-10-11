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

use crate::{file::AccessMode, file_descriptor::FileDescriptor};
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_pal_posix::posix::{self, Errno, MAP_FAILED};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMappingCreationError {
    InsufficientPermissions,
    MappingSizeIsZero,
    ExceedsTheMaximumNumberOfMappedRegions,
    FileDescriptorDoesNotSupportMemoryMappings,
    InsufficientResources,
    MappingLargerThanCorrespondingFile,
    FailedToEnforceAddressHint,
    SynchronizedIONotSupported,
    InterruptSignal,
    FileIsADirectory,
    TooManySymbolicLinksInPath,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    FileDoesNotExist,
    FileTooBig,
    OpenReturnedBrokedFileDescriptor,
    UnknownFailure(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MappingPermission {
    Read = posix::PROT_READ,
    Write = posix::PROT_WRITE,
    ReadWrite = posix::PROT_READ | posix::PROT_WRITE,
    Exec = posix::PROT_EXEC,
    ReadExec = posix::PROT_READ | posix::PROT_EXEC,
    WriteExec = posix::PROT_WRITE | posix::PROT_EXEC,
    ReadWriteExec = posix::PROT_READ | posix::PROT_WRITE | posix::PROT_EXEC,
    None = posix::PROT_NONE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MappingBehavior {
    /// Memory changes are written back and are visible to other processes
    Shared = posix::MAP_SHARED,
    /// Changes are local to the process.
    Private = posix::MAP_PRIVATE,
}

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

#[derive(Debug)]
pub struct MemoryMappingBuilder {
    settings: MemoryMappingBuilderSettings,
    origin: MappingOrigin,
}

impl MemoryMappingBuilder {
    pub fn from_anonymous() -> Self {
        Self {
            settings: MemoryMappingBuilderSettings::new(),
            origin: MappingOrigin::Anonymous,
        }
    }

    pub fn from_fd(file_descriptor: FileDescriptor) -> Self {
        Self {
            settings: MemoryMappingBuilderSettings::new(),
            origin: MappingOrigin::FileDescriptor(file_descriptor),
        }
    }

    pub fn from_file(file_path: &FilePath, value: AccessMode) -> Self {
        Self {
            settings: MemoryMappingBuilderSettings::new(),
            origin: MappingOrigin::File((file_path.clone(), value)),
        }
    }

    pub fn mapping_behavior(mut self, value: MappingBehavior) -> Self {
        self.settings.mapping_behavior = value;
        self
    }

    pub fn initial_mapping_permission(mut self, value: MappingPermission) -> Self {
        self.settings.mapping_permission = value;
        self
    }

    pub fn mapping_address_hint(mut self, value: usize) -> Self {
        self.settings.address_hint = value;
        self
    }

    pub fn enforce_mapping_address_hint(mut self, value: bool) -> Self {
        self.settings.enforce_address_hint = value;
        self
    }

    pub fn size(mut self, value: usize) -> Self {
        self.settings.size = value;
        self
    }

    pub fn offset(mut self, value: isize) -> Self {
        self.settings.offset = value;
        self
    }

    pub fn create(self) -> Result<MemoryMapping, MemoryMappingCreationError> {
        match self.origin {
            MappingOrigin::Anonymous => {
                Self::create_mapping(&self.settings, posix::MAP_ANONYMOUS, None)
            }
            MappingOrigin::FileDescriptor(file_descriptor) => Self::create_mapping(
                &self.settings,
                self.settings.mapping_behavior as _,
                Some(file_descriptor),
            ),
            MappingOrigin::File((ref file_path, access_mode)) => {
                let msg = "Unable to create memory mapping since the corresponding file could not be opened";
                let mem_fd = unsafe {
                    posix::open(file_path.as_c_str(), access_mode as i32 | posix::O_SYNC)
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

                Self::create_mapping(
                    &self.settings,
                    self.settings.mapping_behavior as _,
                    Some(fd),
                )
            }
        }
    }

    fn create_mapping(
        settings: &MemoryMappingBuilderSettings,
        mapping_behavior: i32,
        file_descriptor: Option<FileDescriptor>,
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
                settings.offset as i64,
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
            file_descriptor: None,
            file_path: None,
            base_address: ret_val.cast(),
            size: settings.size,
        };

        if settings.enforce_address_hint && ret_val as usize != settings.address_hint {
            fail!(from settings, with MemoryMappingCreationError::FailedToEnforceAddressHint,
                "{msg} since the address hint of {:#x?} could not be enforced.",
                settings.address_hint);
        }

        Ok(mapping)
    }
}

#[derive(Debug)]
pub struct MemoryMapping {
    file_descriptor: Option<FileDescriptor>,
    file_path: Option<FilePath>,
    base_address: *mut u8,
    size: usize,
}

impl Drop for MemoryMapping {
    fn drop(&mut self) {
        if unsafe { posix::munmap(self.base_address.cast(), self.size) } == -1 {
            fatal_panic!(from self,
                "This should never happen! Unable to unmap a mapped memory region.");
        }
    }
}

impl MemoryMapping {
    pub fn protect(&self, addr: usize) -> ProtectBuilder {
        todo!()
    }

    pub fn file_descriptor(&self) -> &Option<FileDescriptor> {
        &self.file_descriptor
    }

    pub fn base_address(&self) -> *const u8 {
        self.base_address
    }

    pub fn base_address_mut(&mut self) -> *mut u8 {
        self.base_address
    }

    pub fn file_path(&self) -> &Option<FilePath> {
        &self.file_path
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

pub struct ProtectBuilder {}

impl ProtectBuilder {
    pub fn size(mut self, value: usize) -> Self {
        todo!()
    }

    pub fn mapping_permission(mut self, value: MappingPermission) -> Self {
        todo!()
    }

    pub fn apply(self) {
        todo!()
    }
}
