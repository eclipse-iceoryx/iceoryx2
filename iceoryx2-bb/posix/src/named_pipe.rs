// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

//! Read, create, write or modify named pipes based on a POSIX api. It provides also advanced features
//! like [`Permission`] setting.
//!
//! # Examples
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::named_pipe::*;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let pipe_path = FilePath::new(b"/tmp/iceoryx2/myNamedPipe").unwrap();
//! let mut named_pipe = NamedPipeBuilder::new(&pipe_path)
//!                                  .creation_mode(CreationMode::CreateExclusive)
//!                                  .permission(Permission::OWNER_READ_WRITE | Permission::GROUP_READ)
//!                                  .create().expect("Failed to create named pipe");
//!
//! let content: [u8; 5] = [1, 2, 3, 4, 5];
//! let mut read_buffer: [u8; 5] = [0u8; 5];
//! named_pipe.write(content.as_slice()).expect("Failed to write to named pipe");
//! named_pipe.read(&mut read_buffer).expect("Failed to read from named pipe");
//!
//! match named_pipe.remove_self().expect("Failed to remove") {
//!   true => println!("removed named pipe"),
//!   false => println!("named pipe did not exist"),
//! }
//! ```

use alloc::format;
use core::fmt::Debug;
use core::ptr::NonNull;
use core::time::Duration;

pub use crate::creation_mode::CreationMode;
use crate::file::{
    File, FileOffsetError, FileRemoveError, FileSetOwnerError, FileSetPermissionError,
    FileStatError,
};
use crate::file_descriptor::{FileDescriptor, FileDescriptorBased, FileDescriptorManagement};
use crate::file_descriptor_set::{
    FileDescriptorSet, FileDescriptorSetWaitError, FileEvent, SynchronousMultiplexing,
};
use crate::file_type::FileType;
use crate::group::{Gid, GroupError};
use crate::metadata::{Metadata, MetadataFromPathError};
use crate::ownership::OwnershipBuilder;
use crate::user::{Uid, UserError};
pub use crate::{access_mode::AccessMode, permission::*};

use iceoryx2_bb_concurrency::atomic::{AtomicBool, Ordering};
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_system_types::path::SemanticString;
use iceoryx2_log::{fail, fatal_panic, trace, warn};
use iceoryx2_pal_posix::posix::Errno;
use iceoryx2_pal_posix::*;

enum_gen! { NamedPipeOpenError
  entry:
    InsufficientPermissions,
    Interrupt,
    IsDirectory,
    LoopInSymbolicLinks,
    PerProcessFileHandleLimitReached,
    MaxFilePathLengthExceeded,
    SystemWideFileHandleLimitReached,
    FileDoesNotExist,
    FileTooBig,
    FilesytemIsReadOnly,
    InsufficientMemory,
    DirectoryDoesNotExist,
    UnknownError(i32)
}

enum_gen! { NamedPipeCreationError
  entry:
    AlreadyExist,
    Interrupt,
    DirectoryDoesNotExist,
    IsDirectory,
    LoopInSymbolicLinks,
    FilesytemIsReadOnly,
    PerProcessFileHandleLimitReached,
    MaxFilePathLengthExceeded,
    SystemWideFileHandleLimitReached,
    FileTooBig,
    NoSpaceLeft,
    InsufficientMemory,
    InsufficientPermissions,
    PartsOfThePathDoNotExist,
    NotADirectory,
    UnknownError(i32)
  mapping:
    FileStatError,
    UserError,
    GroupError,
    FileSetOwnerError,
    FileSetPermissionError,
    FileRemoveError,
    MetadataFromPathError
}

enum_gen! { NamedPipeReadError
  entry:
    OpenedWithoutReadAccessMode,
    Interrupt,
    IOerror,
    IsDirectory,
    FileTooBig,
    InsufficientResources,
    InsufficientMemory,
    NonExistingOrIncapableDevice,
    MaxSupportedPathLengthExceeded,
    InvalidFileDescriptor,
    UnknownError(i32)
  mapping:
    FileOffsetError,
    FileStatError,
    NamedPipeSetPropertyError
}

enum_gen! { NamedPipeWriteError
  entry:
    OpenedWithoutWriteAccessMode,
    Interrupt,
    WriteBufferTooBig,
    IOerror,
    NoSpaceLeft,
    InsufficientResources,
    InsufficientPermissions,
    NonExistingOrIncapableDevice,
    UnknownError(i32)
  mapping:
    FileOffsetError,
    NamedPipeSetPropertyError
}

enum_gen! { NamedPipeSetPropertyError
  entry:
    Interrupt,
    InvalidFileDescriptor,
    InvalidCmd,
    FileTooBig,
    WouldCauseOverflow,
    UnknownError(i32)
}

enum_gen! {
    /// The NamedPipeError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward NamedPipeError as more generic return value when a method
    /// returns a NamedPipe***Error.
    /// On a higher level it is again convertible to [`crate::Error`].
    NamedPipeError
  generalization:
    Create <= NamedPipeCreationError,
    Write <= NamedPipeWriteError; FileRemoveError,
    Read <= NamedPipeReadError; NamedPipeOpenError; MetadataFromPathError,
    Credentials <= FileSetPermissionError; FileSetOwnerError
}

/// Opens or creates a new [`NamedPipe`]. When calling [`NamedPipeBuilder::creation_mode`] the
/// [`NamedPipeCreationBuilder`] is returned which provides additional settings only available
/// for newly created pipes.
///
/// # Details
/// The default [`AccessMode`] is [`AccessMode::Read`].
///
/// # Examples
/// ## Open existing named pipe for reading
/// ```
/// # extern crate iceoryx2_bb_loggers;
///
/// use iceoryx2_bb_posix::named_pipe::*;
/// use iceoryx2_bb_system_types::file_path::FilePath;
/// use iceoryx2_bb_container::semantic_string::SemanticString;
///
/// let pipe_path = FilePath::new(b"/tmp/iceoryx2/roehrich").unwrap();
/// let named_pipe = NamedPipeBuilder::new(&pipe_path)
///                     .open_existing(AccessMode::Read);
/// ```
///
/// ## Create new named pipe for writing with extras
/// ```ignore
/// use iceoryx2_bb_posix::named_pipe::*;
/// use iceoryx2_bb_posix::user::UserExt;
/// use iceoryx2_bb_posix::group::GroupExt;
/// use iceoryx2_bb_system_types::file_path::FilePath;
/// use iceoryx2_bb_container::semantic_string::SemanticString;
///
/// let pipe_path = FilePath::new(b"/tmp/iceoryx2/roehrich").unwrap();
/// let named_pipe = NamedPipeBuilder::new(&pipe_path)
///                              .creation_mode(CreationMode::CreateExclusive)
///                              // BEGIN: optional settings
///                              .permission(Permission::OWNER_WRITE)
///                              .owner("testuser1".as_user().unwrap().uid())
///                              .group("testgroup2".as_group().unwrap().gid())
///                              // END: optional settings
///                              .create();
/// ```
#[derive(Debug)]
pub struct NamedPipeBuilder {
    file_path: FilePath,
    access_mode: AccessMode,
    permission: Permission,
    has_ownership: bool,
    is_non_blocking: bool,
    is_async_io: bool,
    owner: Option<Uid>,
    group: Option<Gid>,
    creation_mode: Option<CreationMode>,
}

impl NamedPipeBuilder {
    /// Creates a new NamedPipeBuilder and sets the path of the named pipe which should be opened.
    pub fn new(file_path: &FilePath) -> Self {
        NamedPipeBuilder {
            file_path: *file_path,
            access_mode: AccessMode::ReadWrite,
            permission: Permission::OWNER_READ_WRITE,
            has_ownership: false,
            is_non_blocking: false,
            is_async_io: false,
            owner: None,
            group: None,
            creation_mode: None,
        }
    }

    /// Defines if the created or opened pipe is owned by the [`NamedPipe`] object. If it is owned, the
    /// [`NamedPipe`] object will remove the underlying named pipe when it goes out of scope.
    pub fn has_ownership(mut self, value: bool) -> Self {
        self.has_ownership = value;
        self
    }

    /// Returns a [`NamedPipeCreationBuilder`] object to define further settings exclusively
    /// for newly created named pipes. Sets the [`AccessMode`] of the pipe to [`AccessMode::ReadWrite`].
    pub fn creation_mode(mut self, value: CreationMode) -> NamedPipeCreationBuilder {
        self.creation_mode = Some(value);
        self.access_mode = AccessMode::ReadWrite;
        NamedPipeCreationBuilder { config: self }
    }

    /// Opens an existing pipe at the given file_path and defines how the files [`AccessMode`],
    /// for reading, writing. Is independent of
    /// the current permissions of the pipe. But writing to a read-only pipe can result
    /// in some kind of error.
    pub fn open_existing(mut self, value: AccessMode) -> Result<NamedPipe, NamedPipeOpenError> {
        self.access_mode = value;
        NamedPipe::open(self)
    }
}

/// Sets additional settings for pipes which are being newly created. Is returned when
/// [`NamedPipeBuilder::creation_mode()`] is called in [`NamedPipeBuilder`].
pub struct NamedPipeCreationBuilder {
    config: NamedPipeBuilder,
}

impl NamedPipeCreationBuilder {
    pub fn permission(mut self, value: Permission) -> Self {
        self.config.permission = value;
        self
    }

    pub fn owner(mut self, value: Uid) -> Self {
        self.config.owner = Some(value);
        self
    }

    pub fn group(mut self, value: Gid) -> Self {
        self.config.group = Some(value);
        self
    }

    /// Creates a new named pipe
    pub fn create(self) -> Result<NamedPipe, NamedPipeCreationError> {
        let mut named_pipe = NamedPipe::create(&self.config)?;
        fail!(from self.config, when named_pipe.set_permission(self.config.permission), "Failed to set permissions.");

        if self.config.owner.is_some() || self.config.group.is_some() {
            let owner = fail!(from self.config, when named_pipe.ownership(), "Failed to acquire current owners.");

            let owner_id = match self.config.owner.as_ref() {
                Some(v) => *v,
                None => owner.uid(),
            };

            let group_id = match self.config.group.as_ref() {
                Some(v) => *v,
                None => owner.gid(),
            };

            fail!(from self.config, when named_pipe.set_ownership(OwnershipBuilder::new().uid(owner_id).gid(group_id).create()),
                "Failed to set ownership.");
        }

        trace!(from self.config, "created");
        Ok(named_pipe)
    }
}

/// ####### Section NamedPipe
/// Opens, creates or send/receive with a named Pipe. Can be created by the [`NamedPipeBuilder`].
#[derive(Debug)]
pub struct NamedPipe {
    path: Option<FilePath>,
    file_descriptor: FileDescriptor,
    access_mode: AccessMode,
    has_ownership: AtomicBool,
    is_non_blocking: AtomicBool,
    is_async_io: AtomicBool,
}

impl Abandonable for NamedPipe {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        unsafe { core::ptr::drop_in_place(&mut this.file_descriptor) };
    }
}

impl Drop for NamedPipe {
    fn drop(&mut self) {
        if self.has_ownership.load(Ordering::Relaxed) {
            match &self.path {
                None => {
                    warn!(from self, "Named pipes created from file descriptors cannot remove themselves.")
                }
                Some(p) => match File::remove(p) {
                    Ok(false) | Err(_) => {
                        warn!(from self, "Failed to remove owned named pipe");
                    }
                    Ok(true) => (),
                },
            };
        }
        trace!(from self, "closed");
    }
}

impl NamedPipe {
    fn create_fifo_special_file(
        path: &FilePath,
        perm: &Permission,
    ) -> Result<(), NamedPipeCreationError> {
        let msg = "Unable to create fifo special file";
        if unsafe { posix::mkfifo(path.as_c_str(), perm.as_mode()) } == -1 {
            match Errno::get() {
                Errno::EACCES => {
                    fail!(with NamedPipeCreationError::InsufficientPermissions,
                        "{msg} due to insufficient permissions.");
                }
                Errno::ELOOP => {
                    fail!(with NamedPipeCreationError::LoopInSymbolicLinks,
                        "{msg} due to a loop in the symbolic links.");
                }
                Errno::EEXIST => {
                    fail!(with NamedPipeCreationError::AlreadyExist,
                        "{msg} since the named pipe already exist.");
                }
                Errno::ENAMETOOLONG => {
                    fail!(with NamedPipeCreationError::MaxFilePathLengthExceeded,
                        "{msg} since the file path length exceeds the maximum supported file path length.");
                }
                Errno::ENOENT => {
                    fail!(with NamedPipeCreationError::PartsOfThePathDoNotExist,
                        "{msg} since parts of the path do not exist.");
                }
                Errno::ENOSPC => {
                    fail!(with NamedPipeCreationError::NoSpaceLeft,
                        "{msg} since there is no space left on the target file-system.");
                }
                Errno::ENOTDIR => {
                    fail!(with NamedPipeCreationError::NotADirectory,
                        "{msg} since the provided pathname is not a path.");
                }
                Errno::EROFS => {
                    fail!(with NamedPipeCreationError::FilesytemIsReadOnly,
                        "{msg} since the parent directory resides on a read-only filesystem.");
                }
                e => {
                    fail!(with NamedPipeCreationError::UnknownError(e as i32),
                        "{msg} due to an unknown failure ({e:?}).");
                }
            }
        }
        Ok(())
    }

    fn create(config: &NamedPipeBuilder) -> Result<NamedPipe, NamedPipeCreationError> {
        let msg = "Unable to create named pipe";

        let create_fifo = || -> Result<Option<FileDescriptor>, NamedPipeCreationError> {
            NamedPipe::create_fifo_special_file(&config.file_path, &config.permission)?;
            Ok(FileDescriptor::new(unsafe {
                posix::open(config.file_path.as_c_str(), config.access_mode.as_oflag())
            }))
        };

        let file_descriptor = match config
            .creation_mode
            .expect("The creation mode must always be defined when creating a named pipe.")
        {
            CreationMode::CreateExclusive => create_fifo(),
            CreationMode::PurgeAndCreate => {
                if fail!(from config, when NamedPipe::does_exist(&config.file_path), "{} since the named pipe existence verification failed.", msg)
                {
                    fail!(from config, when File::remove(&config.file_path), "{} since the removal of the already existing named pipe failed.", msg);
                }

                create_fifo()
            }
            CreationMode::OpenOrCreate => {
                match fail!(from config, when NamedPipe::does_exist(&config.file_path), "{} since the named pipe existence verification failed.", msg)
                {
                    true => Ok(FileDescriptor::new(unsafe {
                        posix::open(config.file_path.as_c_str(), config.access_mode.as_oflag())
                    })),
                    false => create_fifo(),
                }
            }
        }?;

        if let Some(v) = file_descriptor {
            return Ok(NamedPipe {
                path: Some(config.file_path),
                file_descriptor: v,
                has_ownership: AtomicBool::new(config.has_ownership),
                access_mode: config.access_mode,
                is_non_blocking: AtomicBool::new(config.is_non_blocking),
                is_async_io: AtomicBool::new(config.is_async_io),
            });
        }

        match posix::Errno::get() {
            Errno::EACCES => {
                fail!(with NamedPipeCreationError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Errno::EEXIST => {
                fail!(with NamedPipeCreationError::AlreadyExist,
                    "{msg} since the named pipe already exist.");
            }
            Errno::EINTR => {
                fail!(with NamedPipeCreationError::Interrupt,
                    "{msg} since an interrupt signal was received.");
            }
            Errno::ENOENT => {
                fail!(with NamedPipeCreationError::DirectoryDoesNotExist,
                    "{msg} since parts of the path do not exist.");
            }
            Errno::EISDIR => {
                fail!(with NamedPipeCreationError::IsDirectory,
                    "{msg} since the path is a directory.");
            }
            Errno::ELOOP => {
                fail!(with NamedPipeCreationError::LoopInSymbolicLinks,
                    "{msg} due to a loop in the symbolic links.");
            }
            Errno::EROFS => {
                fail!(with NamedPipeCreationError::FilesytemIsReadOnly,
                    "{msg} since the parent directory resides on a read-only filesystem.");
            }
            Errno::EMFILE => {
                fail!(with NamedPipeCreationError::PerProcessFileHandleLimitReached,
                    "{msg} since the current process already holds the maximum amount of file descriptors.");
            }
            Errno::ENAMETOOLONG => {
                fail!(with NamedPipeCreationError::MaxFilePathLengthExceeded,
                    "{msg} since the file path length exceeds the maximum supported file path length.");
            }
            Errno::ENFILE => {
                fail!(with NamedPipeCreationError::SystemWideFileHandleLimitReached,
                    "{msg} since the system-wide maximum of filedescriptors is reached.");
            }
            Errno::EOVERFLOW => {
                fail!(with NamedPipeCreationError::FileTooBig,
                    "{msg} since it is too large to be represented with 'off_t'.");
            }
            Errno::ENOSPC => {
                fail!(with NamedPipeCreationError::NoSpaceLeft,
                    "{msg} since there is no space left on the target file-system.");
            }
            Errno::ENOMEM => {
                fail!(with NamedPipeCreationError::InsufficientMemory,
                    "{msg} due to insufficient memory.");
            }
            e => {
                fail!(with NamedPipeCreationError::UnknownError(e as i32),
                    "{msg} due to an unknown failure ({e:?}).");
            }
        }
    }

    fn open(config: NamedPipeBuilder) -> Result<NamedPipe, NamedPipeOpenError> {
        let msg = "Unable to open named pipe";
        let file_descriptor = FileDescriptor::new(unsafe {
            posix::open(config.file_path.as_c_str(), config.access_mode.as_oflag())
        });

        if let Some(v) = file_descriptor {
            trace!(from config, "opened");
            let new_self = NamedPipe {
                path: Some(config.file_path),
                file_descriptor: v,
                has_ownership: AtomicBool::new(config.has_ownership),
                access_mode: config.access_mode,
                is_non_blocking: AtomicBool::new(config.is_non_blocking),
                is_async_io: AtomicBool::new(config.is_async_io),
            };

            return Ok(new_self);
        }

        match posix::Errno::get() {
            Errno::EACCES => {
                fail!(with NamedPipeOpenError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            Errno::EINTR => {
                fail!(with NamedPipeOpenError::Interrupt,
                    "{msg} since an interrupt signal was received.");
            }
            Errno::EISDIR => {
                fail!(with NamedPipeOpenError::IsDirectory,
                    "{msg} since the path is a directory.");
            }
            Errno::ELOOP => {
                fail!(with NamedPipeOpenError::LoopInSymbolicLinks,
                    "{msg} due to a loop in the symbolic links.");
            }
            Errno::EMFILE => {
                fail!(with NamedPipeOpenError::PerProcessFileHandleLimitReached,
                    "{msg} since the current process already holds the maximum amount of file descriptors.");
            }
            Errno::ENAMETOOLONG => {
                fail!(with NamedPipeOpenError::MaxFilePathLengthExceeded,
                    "{msg} since the file path length exceeds the maximum supported file path length.");
            }
            Errno::ENFILE => {
                fail!(with NamedPipeOpenError::SystemWideFileHandleLimitReached,
                    "{msg} since the system-wide maximum of filedescriptors is reached.");
            }
            Errno::ENOENT => {
                fail!(with NamedPipeOpenError::FileDoesNotExist,
                    "{msg} since the named pipe does not exist.");
            }
            Errno::ENOTDIR => {
                fail!(with NamedPipeOpenError::DirectoryDoesNotExist,
                    "{msg} since the directory to the named pipe does not exist.");
            }
            Errno::EOVERFLOW => {
                fail!(with NamedPipeOpenError::FileTooBig,
                    "{msg} since it is too large to be represented with 'off_t'.");
            }
            Errno::EROFS => {
                fail!(with NamedPipeOpenError::FilesytemIsReadOnly,
                    "{msg} since the parent directory resides on a read-only filesystem.");
            }
            Errno::ENOMEM => {
                fail!(with NamedPipeOpenError::InsufficientMemory,
                    "{msg} due to insufficient memory.");
            }
            e => {
                fail!(with NamedPipeOpenError::UnknownError(e as i32),
                    "{msg} due to an unknown failure ({e:?}).");
            }
        }
    }

    /// Returns `true` if the [`NamedPipe`] has the O_ASYNC flag set
    /// removed when it goes out-of-scope, otherwise it returns `false`.
    pub fn is_async_io(&self) -> bool {
        self.is_async_io.load(Ordering::Relaxed)
    }

    /// Returns the [`AccessMode`] under which the [`NamedPipe`] was opened. The [`AccessMode`] defines if
    /// the [`NamedPipe`] can be read or written.
    pub fn access_mode(&self) -> AccessMode {
        self.access_mode
    }

    /// Returns `true` if the [`NamedPipe`] is owned by the construct and automatically
    /// removed when it goes out-of-scope, otherwise it returns `false`.
    pub fn has_ownership(&self) -> bool {
        self.has_ownership.load(Ordering::Relaxed)
    }

    /// Takes the ownership to the underlying named pipe, meaning when [`NamedPipe`] goes out of scope the
    /// pipe is removed from the file system.
    pub fn acquire_ownership(&self) {
        self.has_ownership.store(true, Ordering::Relaxed);
    }

    /// Releases the ownership to the underlying named pipe, meaning when [`NamedPipe`] goes out of scope, the
    /// pipe will not be removed from the file system.
    pub fn release_ownership(&self) {
        self.has_ownership.store(false, Ordering::Relaxed);
    }

    pub fn does_exist(file_path: &FilePath) -> Result<bool, MetadataFromPathError> {
        let origin = "NamedPipe::does_exist()";
        let msg = format!("Unable to determine if named pipe \"{file_path}\" exists");
        Metadata::does_exist(&file_path.into(), origin, &msg, FileType::FiFo)
    }

    /// Deletes the NamedPipe managed by self
    pub fn remove_self(self) -> Result<bool, FileRemoveError> {
        match &self.path {
            None => {
                fail!(from self, with FileRemoveError::FileDoesNotExist, "NamedPipe does not exist in path");
            }
            Some(p) => {
                self.release_ownership();
                File::remove(p)
            }
        }
    }

    fn fcntl(&self, command: i32, value: i32, msg: &str) -> Result<i32, NamedPipeSetPropertyError> {
        let result =
            unsafe { posix::fcntl_int(self.file_descriptor.native_handle(), command, value) };

        if result >= 0 {
            Ok(result)
        } else {
            match Errno::get() {
                Errno::EBADF => {
                    fail!(with NamedPipeSetPropertyError::InvalidFileDescriptor,
                        "{msg} since an invalid file-descriptor was provided.");
                }
                Errno::EINVAL => {
                    fail!(with NamedPipeSetPropertyError::InvalidCmd,
                        "{msg} since command is invalid.");
                }
                Errno::EOVERFLOW => {
                    fail!(with NamedPipeSetPropertyError::FileTooBig,
                        "{msg} since it is too large to be represented with 'off_t'.");
                }
                Errno::EINTR => {
                    fail!(with NamedPipeSetPropertyError::Interrupt,
                        "{msg} since an interrupt signal was received.");
                }
                e => {
                    fail!(with NamedPipeSetPropertyError::UnknownError(e as i32),
                        "{msg} due to an unknown failure ({e:?}).");
                }
            }
        }
    }

    fn set_non_blocking(&self, value: bool) -> Result<(), NamedPipeSetPropertyError> {
        if self.is_non_blocking.load(Ordering::Relaxed) == value {
            return Ok(());
        }

        let current_flags = self.fcntl(
            posix::F_GETFL,
            0,
            "Unable to acquire current named pipe filedescriptor flags",
        )?;
        let new_flags = match value {
            true => current_flags | posix::O_NONBLOCK,
            false => current_flags & !posix::O_NONBLOCK,
        };

        self.fcntl(posix::F_SETFL, new_flags, "Unable to set blocking mode")?;
        self.is_non_blocking.store(value, Ordering::Relaxed);
        Ok(())
    }

    // Set the O_ASYNC status flag. When enabled it generates a SIGIO signal
    // when input/output is available on the fd of the named pipe
    pub fn set_async_io(&self, value: bool) -> Result<(), NamedPipeSetPropertyError> {
        if self.is_async_io.load(Ordering::Relaxed) == value {
            return Ok(());
        }

        let current_flags = self.fcntl(
            posix::F_GETFL,
            0,
            "Unable to acquire current named pipe filedescriptor flags",
        )?;
        let new_flags = match value {
            true => current_flags | posix::O_ASYNC,
            false => current_flags & !posix::O_ASYNC,
        };

        self.fcntl(posix::F_SETFL, new_flags, "Unable to set async IO mode")?;
        self.is_async_io.store(value, Ordering::Relaxed);
        Ok(())
    }

    /// Reads the content of a pipe into a slice and returns the number of bytes read but at most
    /// `buf.len()` bytes.
    pub fn read(&self, buf: &mut [u8]) -> Result<u64, NamedPipeReadError> {
        let msg = "Unable to read named pipe";
        if self.access_mode != AccessMode::Read && self.access_mode != AccessMode::ReadWrite {
            fail!(from self, with NamedPipeReadError::OpenedWithoutReadAccessMode,
                "{msg} since the named pipe was not opened with AccessMode::Read or AccessMode::ReadWrite.");
        }

        let bytes_read = unsafe {
            posix::read(
                self.file_descriptor.native_handle(),
                buf.as_mut_ptr() as *mut posix::void,
                buf.len(),
            )
        };

        if bytes_read >= 0 {
            return Ok(bytes_read as u64);
        }

        match posix::Errno::get() {
            Errno::EINTR => {
                fail!(with NamedPipeReadError::Interrupt,
                    "{msg} since an interrupt signal was received.");
            }
            Errno::EIO => {
                fail!(with NamedPipeReadError::IOerror,
                    "{msg} due to an I/O error.");
            }
            Errno::EISDIR => {
                fail!(with NamedPipeReadError::InvalidFileDescriptor,
                    "{msg} since an invalid file-descriptor was provided.");
            }
            Errno::EOVERFLOW => {
                fail!(with NamedPipeReadError::FileTooBig,
                    "{msg} since it is too large to be represented with 'off_t'.");
            }
            Errno::ENOBUFS => {
                fail!(with NamedPipeReadError::InsufficientResources,
                    "{msg} due to insufficient resources to perform the operation.");
            }
            Errno::ENOMEM => {
                fail!(with NamedPipeReadError::InsufficientMemory,
                    "{msg} due to insufficient memory.");
            }
            Errno::ENXIO => {
                fail!(with NamedPipeReadError::NonExistingOrIncapableDevice,
                    "{msg} since the device either does not exist or is not capable of that operation.");
            }
            e => {
                fail!(with NamedPipeReadError::UnknownError(e as i32),
                    "{msg} due to an unknown failure ({e:?}).");
            }
        }
    }

    pub fn try_read(&self, buf: &mut [u8]) -> Result<u64, NamedPipeReadError> {
        fail!(from self, when self.set_non_blocking(true),
                "Unable to try receive message since the named pipe could not bet set into unblocking state.");
        self.read(buf)
    }

    pub fn blocking_read(&self, buf: &mut [u8]) -> Result<u64, NamedPipeReadError> {
        let msg = "Unable to blocking receive message";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the named pipe could not bet set into blocking state.", msg);
        self.read(buf)
    }

    /// Blocks until either a message was received or the timeout has passed. If no
    /// message was received the method returns 0 otherwise the number of bytes received.
    pub fn timed_read(
        &self,
        buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<u64, NamedPipeReadError> {
        let msg = "Failed to timed receive on named pipe";

        fail!(from self, when self.set_non_blocking(false),
            "{} since the named pipe could not activate the blocking mode.", msg);

        let fd_set = FileDescriptorSet::new();
        let _guard = fatal_panic!(from self, when fd_set.add(self),
                            "This should never happen! {} since the named pipe could not be attached to a fd set.", msg);

        let mut received_bytes = Ok(0);
        let receive_call = |_: &FileDescriptor| {
            received_bytes = self.read(buffer);
        };

        match fd_set.timed_wait(timeout, FileEvent::Read, receive_call) {
            Err(FileDescriptorSetWaitError::Interrupt) => {
                fail!(from self, with NamedPipeReadError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(_) => {
                fail!(from self, with NamedPipeReadError::UnknownError(-1),
                    "{} since an unknown failure occurred.", msg);
            }
            Ok(_) => received_bytes,
        }
    }

    /// Writes a slice into a named pipe and returns the number of bytes which were written.
    /// write() is always non-blocking
    pub fn write(&self, buf: &[u8]) -> Result<u64, NamedPipeWriteError> {
        let msg = "Unable to write content";

        fail!(from self, when self.set_non_blocking(true),
                "{} since the named pipe could not bet set into non-blocking state.", msg);

        if self.access_mode != AccessMode::Write && self.access_mode != AccessMode::ReadWrite {
            fail!(from self, with NamedPipeWriteError::OpenedWithoutWriteAccessMode,
                "{msg} since the named pipe was not opened with AccessMode::Write or AccessMode::ReadWrite.");
        }

        let bytes_written = unsafe {
            posix::write(
                self.file_descriptor.native_handle(),
                buf.as_ptr() as *const posix::void,
                buf.len(),
            )
        };

        if bytes_written >= 0 {
            return Ok(bytes_written as u64);
        }

        match posix::Errno::get() {
            Errno::EFBIG => {
                fail!(with NamedPipeWriteError::WriteBufferTooBig,
                    "{msg} since the named pipe size would then exceed the internal maximum file size limit.");
            }
            Errno::EINTR => {
                fail!(with NamedPipeWriteError::Interrupt,
                    "{msg} since an interrupt signal was received.");
            }
            Errno::EIO => {
                fail!(with NamedPipeWriteError::IOerror,
                    "{msg} due to an I/O error.");
            }
            Errno::ENOSPC => {
                fail!(with NamedPipeWriteError::NoSpaceLeft,
                    "{msg} since there is no space left on the named pipe device.");
            }
            Errno::ENOBUFS => {
                fail!(with NamedPipeWriteError::InsufficientResources,
                    "{msg} due to insufficient resources to perform the operation.");
            }
            Errno::ENXIO => {
                fail!(with NamedPipeWriteError::NonExistingOrIncapableDevice,
                    "{msg} since the device either does not exist or is not capable of that operation.");
            }
            Errno::EACCES => {
                fail!(with NamedPipeWriteError::InsufficientPermissions,
                    "{msg} due to insufficient permissions.");
            }
            e => {
                fail!(with NamedPipeWriteError::UnknownError(e as i32),
                    "{msg} due to an unknown failure ({e:?}).");
            }
        }
    }
}

impl FileDescriptorBased for NamedPipe {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl FileDescriptorManagement for NamedPipe {}

impl SynchronousMultiplexing for NamedPipe {}
