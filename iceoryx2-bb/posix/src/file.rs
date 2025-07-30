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

//! Read, create, write or modify files based on a POSIX api. It provides also advanced features
//! like [`Permission`] setting and to be created from a [`FileDescriptor`].
//!
//! # Examples
//! ```no_run
//! use iceoryx2_bb_posix::file::*;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let file_name = FilePath::new(b"ooh_makes_the_file.md").unwrap();
//! let mut file = FileBuilder::new(&file_name)
//!                                  .creation_mode(CreationMode::CreateExclusive)
//!                                  .permission(Permission::OWNER_ALL | Permission::GROUP_READ)
//!                                  .truncate_size(1024)
//!                                  .create().expect("Failed to create file");
//!
//! let mut content: Vec<u8> = vec![];
//! file.read_to_vector(&mut content).expect("Failed to read file");
//! file.write(content.as_slice()).expect("Failed to write file");
//!
//! if File::does_exist(&file_name).expect("Failed to check for existance") {
//!   println!("Woohoo file exists");
//! }
//!
//! match File::remove(&file_name).expect("Failed to remove") {
//!   true => println!("removed file"),
//!   false => println!("file did not exist"),
//! }
//! ```

use crate::file_descriptor::{FileDescriptor, FileDescriptorBased, FileDescriptorManagement};
use crate::group::Gid;
use crate::group::GroupError;
use crate::handle_errno;
use crate::ownership::OwnershipBuilder;
use crate::user::{Uid, UserError};
use core::fmt::Debug;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{fail, trace, warn};
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::posix::MemZeroedStruct;
use iceoryx2_pal_posix::*;

pub use crate::creation_mode::CreationMode;
pub use crate::{access_mode::AccessMode, permission::*};

enum_gen! { FileRemoveError
  entry:
    InsufficientPermissions,
    CurrentlyInUse,
    LoopInSymbolicLinks,
    MaxSupportedPathLengthExceeded,
    PartOfReadOnlyFileSystem,
    UnknownError(i32)
}

impl core::fmt::Display for FileRemoveError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileRemoveError::{self:?}")
    }
}

impl core::error::Error for FileRemoveError {}

enum_gen! { FileAccessError
  entry:
    LoopInSymbolicLinks,
    MaxSupportedPathLengthExceeded,
    InsufficientPermissions,
    UnknownError(i32)
}

impl core::fmt::Display for FileAccessError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileAccessError::{self:?}")
    }
}

impl core::error::Error for FileAccessError {}

enum_gen! { FileCreationError
  entry:
    InsufficientPermissions,
    InsufficientMemory,
    FileAlreadyExists,
    NoSpaceLeft,
    FileTooBig,
    Interrupt,
    IsDirectory,
    LoopInSymbolicLinks,
    FilesytemIsReadOnly,
    DirectoryDoesNotExist,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    MaxFilePathLengthExceeded,
    UnknownError(i32)

  mapping:
    FileStatError,
    UserError,
    GroupError,
    FileSetOwnerError,
    FileTruncateError,
    FileSetPermissionError,
    FileAccessError,
    FileRemoveError
}

impl core::fmt::Display for FileCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileCreationError::{self:?}")
    }
}

impl core::error::Error for FileCreationError {}

enum_gen! { FileOpenError
  entry:
    InsufficientPermissions,
    InsufficientMemory,
    FileDoesNotExist,
    FileTooBig,
    Interrupt,
    IsDirectory,
    LoopInSymbolicLinks,
    FilesytemIsReadOnly,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    MaxFilePathLengthExceeded,
    UnknownError(i32)
}

impl core::fmt::Display for FileOpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileOpenError::{self:?}")
    }
}

impl core::error::Error for FileOpenError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileTruncateError {
    Interrupt,
    SizeTooBig,
    IOerror,
    FileNotOpenedForWriting,
    ReadOnlyFilesystem,
    UnknownError(i32),
}

impl core::fmt::Display for FileTruncateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileTruncateError::{self:?}")
    }
}

impl core::error::Error for FileTruncateError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileStatError {
    InvalidFileDescriptor,
    IOerror,
    FileTooBig,
    UnknownFileType,
    UnknownError(i32),
}

impl core::fmt::Display for FileStatError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileStatError::{self:?}")
    }
}

impl core::error::Error for FileStatError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileSetPermissionError {
    InvalidFileDescriptor,
    InsufficientPermissions,
    ReadOnlyFilesystem,
    UnknownError(i32),
}

impl core::fmt::Display for FileSetPermissionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileSetPermissionError::{self:?}")
    }
}

impl core::error::Error for FileSetPermissionError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileSetOwnerError {
    InvalidFileDescriptor,
    InsufficientPermissions,
    ReadOnlyFilesystem,
    InvalidId,
    IOerror,
    Interrupt,
    UnknownError(i32),
}

impl core::fmt::Display for FileSetOwnerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileSetOwnerError::{self:?}")
    }
}

impl core::error::Error for FileSetOwnerError {}

enum_gen! { FileReadError
  entry:
    Interrupt,
    IOerror,
    IsDirectory,
    FileTooBig,
    InsufficientResources,
    InsufficientMemory,
    NonExistingOrIncapableDevice,
    UnknownError(i32)

  mapping:
    FileOffsetError,
    FileStatError
}

impl core::fmt::Display for FileReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileReadError::{self:?}")
    }
}

impl core::error::Error for FileReadError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileOffsetError {
    InvalidFileDescriptor,
    FileTooBig,
    DoesNotSupportSeeking,
    UnknownError(i32),
}

impl core::fmt::Display for FileOffsetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileOffsetError::{self:?}")
    }
}

impl core::error::Error for FileOffsetError {}

enum_gen! { FileWriteError
  entry:
    Interrupt,
    WriteBufferTooBig,
    IOerror,
    NoSpaceLeft,
    InsufficientResources,
    InsufficientPermissions,
    NonExistingOrIncapableDevice,
    UnknownError(i32)
  mapping:
    FileOffsetError
}

impl core::fmt::Display for FileWriteError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileWriteError::{self:?}")
    }
}

impl core::error::Error for FileWriteError {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileSyncError {
    Interrupt,
    NotSupported,
    IOerror,
    UnknownError(i32),
}

impl core::fmt::Display for FileSyncError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileSyncError::{self:?}")
    }
}

impl core::error::Error for FileSyncError {}

enum_gen! {
    /// The FileError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward FileError as more generic return value when a method
    /// returns a File***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    FileError
  generalization:
    Create <= FileCreationError,
    Write <= FileSyncError; FileWriteError; FileTruncateError; FileRemoveError,
    Read <= FileOffsetError; FileReadError; FileOpenError; FileAccessError,
    Credentials <= FileSetOwnerError; FileSetPermissionError,
    Stat <= FileStatError
}

impl core::fmt::Display for FileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FileError::{self:?}")
    }
}

impl core::error::Error for FileError {}

impl From<()> for FileStatError {
    fn from(_: ()) -> Self {
        FileStatError::UnknownFileType
    }
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileReadLineState {
    LineLen(usize),
    EndOfFile(usize),
}

/// Opens or creates a new [`File`]. When calling [`FileBuilder::creation_mode`] the
/// [`FileCreationBuilder`] is returned which provides additional settings only available
/// for newly created files.
///
/// # Details
/// The default [`AccessMode`] is [`AccessMode::Read`].
///
/// # Examples
/// ## Open existing file for reading
/// ```
/// use iceoryx2_bb_posix::file::*;
/// use iceoryx2_bb_system_types::file_path::FilePath;
/// use iceoryx2_bb_container::semantic_string::SemanticString;
///
/// let file_name = FilePath::new(b"/some/where/over/the/rainbow.md").unwrap();
/// let file = FileBuilder::new(&file_name)
///                              .open_existing(AccessMode::Read);
/// ```
///
/// ## Create new file for reading and writing with extras
/// ```ignore
/// use iceoryx2_bb_posix::file::*;
/// use iceoryx2_bb_posix::user::UserExt;
/// use iceoryx2_bb_posix::group::GroupExt;
/// use iceoryx2_bb_system_types::file_path::FilePath;
/// use iceoryx2_bb_container::semantic_string::SemanticString;
///
/// let file_name = FilePath::new(b"/below/the/surface.md").unwrap();
/// let file = FileBuilder::new(&file_name)
///                              .creation_mode(CreationMode::CreateExclusive)
///                              // BEGIN: optional settings
///                              .permission(Permission::OWNER_ALL | Permission::GROUP_READ)
///                              .owner("testuser1".as_user().unwrap().uid())
///                              .group("testgroup2".as_group().unwrap().gid())
///                              .truncate_size(1024)
///                              // END: optional settings
///                              .create();
/// ```
#[derive(Debug)]
pub struct FileBuilder {
    file_path: FilePath,
    access_mode: AccessMode,
    permission: Permission,
    has_ownership: bool,
    owner: Option<Uid>,
    group: Option<Gid>,
    truncate_size: Option<usize>,
    creation_mode: Option<CreationMode>,
}

impl FileBuilder {
    /// Creates a new FileBuilder and sets the path of the file which should be opened.
    pub fn new(file_path: &FilePath) -> Self {
        FileBuilder {
            file_path: file_path.clone(),
            access_mode: AccessMode::Read,
            permission: Permission::OWNER_ALL,
            has_ownership: false,
            owner: None,
            group: None,
            truncate_size: None,
            creation_mode: None,
        }
    }

    /// Defines if the created or opened file is owned by the [`File`] object. If it is owned, the
    /// [`File`] object will remove the underlying file when it goes out of scope.
    pub fn has_ownership(mut self, value: bool) -> Self {
        self.has_ownership = value;
        self
    }

    /// Returns a [`FileCreationBuilder`] object to define further settings exclusively
    /// for newly created files. Sets the [`AccessMode`] of the file to [`AccessMode::ReadWrite`].
    pub fn creation_mode(mut self, value: CreationMode) -> FileCreationBuilder {
        self.creation_mode = Some(value);
        self.access_mode = AccessMode::ReadWrite;
        FileCreationBuilder { config: self }
    }

    /// Opens an existing file at the given file_path and defines how the files [`AccessMode`],
    /// for reading, writing or both. Is independent of
    /// the current permissions of the file. But writing to a read-only file can result
    /// in some kind of error.
    pub fn open_existing(mut self, value: AccessMode) -> Result<File, FileOpenError> {
        self.access_mode = value;
        File::open(self)
    }
}

/// Sets additional settings for files which are being newly created. Is returned when
/// [`FileBuilder::creation_mode()`] is called in [`FileBuilder`].
pub struct FileCreationBuilder {
    config: FileBuilder,
}

impl FileCreationBuilder {
    /// Sets the permission of the new file. [`Permission`] behave like a bitset and can
    /// be used accordingly.
    ///
    /// # Examples
    /// ```no_run
    /// use iceoryx2_bb_posix::file::*;
    /// use iceoryx2_bb_system_types::file_path::FilePath;
    /// use iceoryx2_bb_container::semantic_string::SemanticString;
    ///
    /// let file_name = FilePath::new(b"someFileName.txt").unwrap();
    /// let file = FileBuilder::new(&file_name)
    ///                              .creation_mode(CreationMode::CreateExclusive)
    ///                              .permission(Permission::OWNER_ALL | Permission::GROUP_READ)
    ///                              .create().expect("failed to create file");
    /// ```
    pub fn permission(mut self, value: Permission) -> Self {
        self.config.permission = value;
        self
    }

    /// Sets the user from a string. If only a user id is available one can acquire the user
    /// name via [`crate::user::User`] and the trait [`crate::user::UserExt`].
    ///
    /// # Examples
    /// ```no_run
    /// use iceoryx2_bb_posix::file::*;
    /// use iceoryx2_bb_posix::user::UserExt;
    /// use iceoryx2_bb_system_types::file_path::FilePath;
    /// use iceoryx2_bb_container::semantic_string::SemanticString;
    ///
    /// let file_name = FilePath::new(b"anotherFile.md").unwrap();
    /// let file = FileBuilder::new(&file_name)
    ///                              .creation_mode(CreationMode::CreateExclusive)
    ///                              .owner("testuser1".as_user().expect("user invalid").uid())
    ///                              .create().expect("failed to create file");
    /// ```
    pub fn owner(mut self, value: Uid) -> Self {
        self.config.owner = Some(value);
        self
    }

    /// Sets the group from a string. If only a group id is available one can acquire the group
    /// name via [`crate::group::Group`] and the trait [`crate::group::GroupExt`].
    ///
    /// # Examples
    /// ```no_run
    /// use iceoryx2_bb_posix::file::*;
    /// use iceoryx2_bb_posix::group::*;
    /// use iceoryx2_bb_system_types::file_path::FilePath;
    /// use iceoryx2_bb_container::semantic_string::SemanticString;
    ///
    /// let file_name = FilePath::new(b"fileName.md").unwrap();
    /// let file = FileBuilder::new(&file_name)
    ///                              .creation_mode(CreationMode::CreateExclusive)
    ///                              .group("testgroup1".as_group().expect("group invalid").gid())
    ///                              .create().expect("failed to create file");
    /// ```
    pub fn group(mut self, value: Gid) -> Self {
        self.config.group = Some(value);
        self
    }

    /// Sets the size of the newly created file.
    pub fn truncate_size(mut self, value: usize) -> Self {
        self.config.truncate_size = Some(value);
        self
    }

    /// Creates a new file
    pub fn create(self) -> Result<File, FileCreationError> {
        let mut file = File::create(&self.config)?;
        fail!(from self.config, when file.set_permission(self.config.permission), "Failed to set permissions.");

        if self.config.truncate_size.is_some() {
            fail!(from self.config, when File::truncate(&file, self.config.truncate_size.unwrap()), "Failed to truncate file size.");
        }

        if self.config.owner.is_some() || self.config.group.is_some() {
            let owner =
                fail!(from self.config, when file.ownership(), "Failed to acquire current owners.");

            let owner_id = match self.config.owner.as_ref() {
                Some(v) => *v,
                None => owner.uid(),
            };

            let group_id = match self.config.group.as_ref() {
                Some(v) => *v,
                None => owner.gid(),
            };

            fail!(from self.config, when file.set_ownership(OwnershipBuilder::new().uid(owner_id).gid(group_id).create()),
                "Failed to set ownership.");
        }

        trace!(from self.config, "created");
        Ok(file)
    }
}

/// Opens, creates or modifies files. Can be created by the [`FileBuilder`].
#[derive(Debug)]
pub struct File {
    path: Option<FilePath>,
    file_descriptor: FileDescriptor,
    has_ownership: bool,
}

impl Drop for File {
    fn drop(&mut self) {
        if self.has_ownership {
            match &self.path {
                None => {
                    warn!(from self, "Files created from file descriptors cannot remove themselves.")
                }
                Some(p) => match File::remove(p) {
                    Ok(false) | Err(_) => {
                        warn!(from self, "Failed to remove owned file");
                    }
                    Ok(true) => (),
                },
            };
        }
    }
}

impl File {
    fn create(config: &FileBuilder) -> Result<File, FileCreationError> {
        let msg = "Unable to create file";
        let create_file = || -> Result<Option<FileDescriptor>, FileCreationError> {
            Ok(FileDescriptor::new(unsafe {
                posix::open_with_mode(
                    config.file_path.as_c_str(),
                    config
                        .creation_mode
                        .expect("CreationMode required when creating new file.")
                        .as_oflag()
                        | config.access_mode.as_oflag(),
                    config.permission.as_mode(),
                )
            }))
        };

        let file_descriptor = match config
            .creation_mode
            .expect("The creation mode must always be defined when creating a file.")
        {
            CreationMode::CreateExclusive => create_file(),
            CreationMode::PurgeAndCreate => {
                if fail!(from config, when File::does_exist(&config.file_path), "{} since the file existance verification failed.", msg)
                {
                    fail!(from config, when File::remove(&config.file_path), "{} since the removal of the already existing file failed.", msg);
                }

                create_file()
            }
            CreationMode::OpenOrCreate => {
                match fail!(from config, when File::does_exist(&config.file_path), "{} since the file existance verification failed.", msg)
                {
                    true => Ok(FileDescriptor::new(unsafe {
                        posix::open(config.file_path.as_c_str(), config.access_mode.as_oflag())
                    })),
                    false => create_file(),
                }
            }
        }?;

        if let Some(v) = file_descriptor {
            return Ok(File {
                path: Some(config.file_path.clone()),
                file_descriptor: v,
                has_ownership: config.has_ownership,
            });
        }

        handle_errno!(FileCreationError, from config,
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EEXIST => (FileAlreadyExists, "{} since the file already exists.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::ENOENT => (DirectoryDoesNotExist, "{} since the path points to a directory that does not exist.", msg),
            Errno::EISDIR => (IsDirectory, "{} since the path is a directory.",msg),
            Errno::ELOOP => (LoopInSymbolicLinks, "{} since a loop in the symbolic links was detected.", msg),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the current process already holds the maximum amount of file descriptors.", msg),
            Errno::ENAMETOOLONG => (MaxFilePathLengthExceeded, "{} since the file path length exceeds the maximum supported file path length.", msg),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide maximum of filedescriptors is reached.", msg),
            Errno::EOVERFLOW => (FileTooBig, "{} since it is too large to be represented with 'off_t'.", msg),
            Errno::ENOSPC => (NoSpaceLeft, "{} since there is no space left on the target file-system.", msg),
            Errno::EROFS => (FilesytemIsReadOnly, "{} with write access on an read-only file system.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).",msg, v)
        );
    }

    fn open(config: FileBuilder) -> Result<File, FileOpenError> {
        let msg = "Unable to open file";
        let file_descriptor = FileDescriptor::new(unsafe {
            posix::open(config.file_path.as_c_str(), config.access_mode.as_oflag())
        });

        if let Some(v) = file_descriptor {
            trace!(from config, "opened");
            return Ok(File {
                path: Some(config.file_path),
                file_descriptor: v,
                has_ownership: config.has_ownership,
            });
        }

        handle_errno!(FileOpenError, from config,
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EISDIR => (IsDirectory, "{} since the path is a directory.",msg),
            Errno::ELOOP => (LoopInSymbolicLinks, "{} since a loop in the symbolic links was detected.", msg),
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the current process already holds the maximum amount of file descriptors.", msg),
            Errno::ENAMETOOLONG => (MaxFilePathLengthExceeded, "{} since the file path length exceeds the maximum supported file path length.", msg),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide maximum of filedescriptors is reached.", msg),
            Errno::ENOENT => (FileDoesNotExist, "{} since it does not exist.", msg),
            Errno::EOVERFLOW => (FileTooBig, "{} since it is too large to be represented with 'off_t'.", msg),
            Errno::EROFS => (FilesytemIsReadOnly, "{} with write access on an read-only file system.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).",msg, v)
        );
    }

    /// Takes the ownership to the underlying file, meaning when [`File`] goes out of scope the
    /// file is removed from the file system.
    pub fn acquire_ownership(&mut self) {
        self.has_ownership = true;
    }

    /// Releases the ownership to the underlying file, meaning when [`File`] goes out of scope, the
    /// file will not be removed from the file system.
    pub fn release_ownership(&mut self) {
        self.has_ownership = false;
    }

    /// Takes ownership of a [`FileDescriptor`]. When [`File`] goes out of scope the file
    /// descriptor is closed.
    pub fn from_file_descriptor(file_descriptor: FileDescriptor) -> Self {
        trace!(from "File::from_file_descriptor", "opened {:?}", file_descriptor);
        Self {
            path: None,
            file_descriptor,
            has_ownership: false,
        }
    }

    /// Reads the current line into a provided vector.
    pub fn read_line_to_vector(
        &self,
        buf: &mut Vec<u8>,
    ) -> Result<FileReadLineState, FileReadError> {
        let mut buffer = [0u8; 1];

        let mut counter = 0;
        loop {
            if self.read(&mut buffer)? == 0 {
                return Ok(FileReadLineState::EndOfFile(counter));
            }

            if buffer[0] == b'\n' {
                return Ok(FileReadLineState::LineLen(counter));
            }

            buf.push(buffer[0]);
            counter += 1;
        }
    }

    /// Reads the current line into a provided string.
    pub fn read_line_to_string(
        &self,
        buf: &mut String,
    ) -> Result<FileReadLineState, FileReadError> {
        self.read_line_to_vector(unsafe { buf.as_mut_vec() })
    }

    /// Reads the content of a file into a slice and returns the number of bytes read but at most
    /// `buf.len()` bytes.
    pub fn read(&self, buf: &mut [u8]) -> Result<u64, FileReadError> {
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

        let msg = "Unable to read file";
        handle_errno!(FileReadError, from self,
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EIO => (IOerror, "{} since an I/O error occurred.", msg),
            Errno::EISDIR => (IsDirectory, "{} since it is actually a directory.", msg),
            Errno::EOVERFLOW => (FileTooBig, "{} since the file is too big and would cause an overflow in an internal structure.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources to perform the operation.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory to perform the operation.", msg),
            Errno::ENXIO => (NonExistingOrIncapableDevice, "{} since the device either does not exist or is not capable of that operation.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        )
    }

    /// Reads and appending the content of a file into a vector and returns the number of bytes read.
    pub fn read_to_vector(&self, buf: &mut Vec<u8>) -> Result<u64, FileReadError> {
        let attr = fail!(from self, when File::acquire_attributes(self), "Unable to acquire file length to read contents of file.");

        let start = buf.len();
        buf.resize(attr.st_size as usize + start, 0u8);
        self.read(&mut buf[start..])
    }

    /// Reads and appending the content of a file into a string and returns the number of bytes read.
    pub fn read_to_string(&self, buf: &mut String) -> Result<u64, FileReadError> {
        self.read_to_vector(unsafe { buf.as_mut_vec() })
    }

    /// Reads a range of a file beginning from `start`. The range length is determined by
    /// to length of the slice `buf`. Returns the bytes read.
    pub fn read_range(&self, start: u64, buf: &mut [u8]) -> Result<u64, FileReadError> {
        let offset = fail!(from self, when self.seek(start), "Unable to set offset to read a range from the file.");

        if offset != start {
            return Ok(0);
        }

        self.read(buf)
    }

    /// Reads a range of a file beginning from `start` until `end` and returns the bytes read.
    pub fn read_range_to_vector(
        &self,
        start: u64,
        end: u64,
        buf: &mut Vec<u8>,
    ) -> Result<u64, FileReadError> {
        if start >= end {
            return Ok(0);
        }

        let start_of_vec = buf.len();
        buf.resize((end - start) as usize + start_of_vec, 0u8);
        self.read_range(start, &mut buf[start_of_vec..])
    }

    /// Reads a range of a file beginning from `start` until `end` and returns the bytes read.
    pub fn read_range_to_string(
        &self,
        start: u64,
        end: u64,
        buf: &mut String,
    ) -> Result<u64, FileReadError> {
        self.read_range_to_vector(start, end, unsafe { buf.as_mut_vec() })
    }

    /// Writes a slice into a file and returns the number of bytes which were written.
    pub fn write(&mut self, buf: &[u8]) -> Result<u64, FileWriteError> {
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

        let msg = "Unable to write content";
        handle_errno!(FileWriteError, from self,
            Errno::EFBIG => (WriteBufferTooBig, "{} since the file size would then exceed the internal maximum file size limit.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EIO => (IOerror, "{} due to an I/O error.", msg),
            Errno::ENOSPC => (NoSpaceLeft, "{} since there is no space left on the device containing the file.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::ENXIO => (NonExistingOrIncapableDevice, "{} since the operation is outside of the capabilities of the device or the device does not exists.", msg),
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).",msg, v)
        );
    }

    /// Writes a slice into a file beginning from `start` and returns the number of bytes which were written.
    pub fn write_at(&mut self, start: u64, buf: &[u8]) -> Result<u64, FileWriteError> {
        let offset = fail!(from self, when self.seek(start), "Unable to set offset to write content at a specific position.");

        if offset != start {
            return Ok(0);
        }

        self.write(buf)
    }

    /// Syncs all file modification with the file system.
    pub fn flush(&mut self) -> Result<(), FileSyncError> {
        if unsafe { posix::fsync(self.file_descriptor.native_handle()) } == 0 {
            return Ok(());
        }

        let msg = "Unable to sync file to device";
        handle_errno!(FileSyncError, from self,
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EINVAL => (NotSupported, "{} since this operation is not supported by the file.", msg),
            Errno::EIO => (IOerror, "{} due to an I/O error.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
        );
    }

    /// Returns true if `path` exists, otherwise false.
    pub fn does_exist(path: &FilePath) -> Result<bool, FileAccessError> {
        let msg = "Unable to determine if file";
        if unsafe { posix::access(path.as_c_str(), posix::F_OK) } == 0 {
            return Ok(true);
        }

        handle_errno!(FileAccessError, from "File::does_exist",
            success Errno::ENOENT => false,
            Errno::ELOOP => (LoopInSymbolicLinks, "{} \"{}\" exists since a loop exists in the symbolic links.", msg, path),
            Errno::ENAMETOOLONG => (MaxSupportedPathLengthExceeded, "{} \"{}\" exists since it is longer than the maximum path name length", msg, path),
            Errno::EACCES => (InsufficientPermissions, "{} \"{}\" due to insufficient permissions.", msg, path),
            Errno::EPERM => (InsufficientPermissions, "{} \"{}\" due to insufficient permissions.", msg, path),
            v => (UnknownError(v as i32), "{} \"{}\" exists caused by an unknown error ({}).", msg, path, v)
        );
    }

    /// Deletes the file managed by self
    pub fn remove_self(self) -> Result<bool, FileRemoveError> {
        match &self.path {
            None => {
                warn!(from self, "Files created from file descriptors cannot remove themselves.");
                Ok(false)
            }
            Some(p) => File::remove(p),
        }
    }

    /// Returns [`Some`] when the file was created via path. If it was created via a
    /// [`FileDescriptor`] it returns [`None`].
    pub fn path(&self) -> Option<&FilePath> {
        match self.path {
            None => None,
            Some(_) => self.path.as_ref(),
        }
    }

    /// Deletes a file. Returns true if the file existed and was removed and false if the
    /// file did not exist.
    pub fn remove(path: &FilePath) -> Result<bool, FileRemoveError> {
        let msg = "Unable to remove file";
        if unsafe { posix::remove(path.as_c_str()) } >= 0 {
            trace!(from "File::remove", "\"{}\"", path);
            return Ok(true);
        }

        handle_errno!(FileRemoveError, from "File::remove",
            success Errno::ENOENT => false,
            Errno::EACCES => (InsufficientPermissions, "{} \"{}\" due to insufficient permissions.", msg, path),
            Errno::EPERM => (InsufficientPermissions, "{} \"{}\" due to insufficient permissions.", msg, path),
            Errno::EBUSY => (CurrentlyInUse, "{} \"{}\" since it is currently in use.", msg, path),
            Errno::ELOOP => (LoopInSymbolicLinks, "{} \"{}\" since a loop exists in the symbolic links.", msg, path),
            Errno::ENAMETOOLONG => (MaxSupportedPathLengthExceeded, "{} \"{}\" since it is longer than the maximum path name length.", msg, path),
            Errno::EROFS => (PartOfReadOnlyFileSystem, "{} \"{}\" since it is part of a read-only filesystem.", msg, path),
            v => (UnknownError(v as i32), "{} \"{}\" since an unkown error occurred ({}).", msg, path, v)
        );
    }

    /// Seek to an absolute position in the file.
    pub fn seek(&self, offset: u64) -> Result<u64, FileOffsetError> {
        Self::set_offset(self, offset)
    }

    pub(crate) fn set_offset<T: FileDescriptorBased + Debug>(
        this: &T,
        offset: u64,
    ) -> Result<u64, FileOffsetError> {
        let new_offset = unsafe {
            posix::lseek(
                this.file_descriptor().native_handle(),
                offset as posix::off_t,
                posix::SEEK_SET,
            )
        };

        if new_offset >= 0 {
            return Ok(new_offset as u64);
        }

        let msg = "Unable to change read/write position to";
        handle_errno!(FileOffsetError, from this,
            Errno::EBADF => (InvalidFileDescriptor, "{} {} since the provide file-descriptor was not valid.", msg, offset),
            Errno::EOVERFLOW => (FileTooBig, "{} {} since the file size is so large that ic cannot be represented by an internal structure.", msg, offset),
            Errno::ESPIPE => (DoesNotSupportSeeking, "{} {} since the file type does not support seeking.", msg, offset),
            v => (UnknownError(v as i32), "{} {} due to an unknown error ({}).", msg, offset, v)
        );
    }

    pub(crate) fn truncate<T: FileDescriptorBased + Debug>(
        this: &T,
        size: usize,
    ) -> Result<(), FileTruncateError> {
        if unsafe { posix::ftruncate(this.file_descriptor().native_handle(), size as posix::off_t) }
            == 0
        {
            return Ok(());
        }

        let msg = "Unable to resize file to";
        handle_errno!(FileTruncateError, from this,
            Errno::EINTR => (Interrupt, "{} {} since an interrupt signal was received.", msg, size),
            Errno::EFBIG => (SizeTooBig, "{} {} since the size is too big. Maybe the file can only shrink?", msg, size),
            Errno::EIO => (IOerror, "{} {} due to an I/O error while writing to the file system.", msg, size),
            Errno::EBADF => (FileNotOpenedForWriting, "{} {} file is not opened for writing.", msg, size),
            Errno::EROFS => (ReadOnlyFilesystem, "{} {} since the file resides on a read-only file system.", msg, size),
            v => (UnknownError(v as i32), "{} {} due to an unknown error ({}).", msg, size, v)
        );
    }

    pub(crate) fn acquire_attributes<T: FileDescriptorBased + Debug>(
        this: &T,
    ) -> Result<posix::stat_t, FileStatError> {
        let mut attr = posix::stat_t::new_zeroed();
        if unsafe { posix::fstat(this.file_descriptor().native_handle(), &mut attr) } == -1 {
            let msg = "Unable to acquire file stats";
            handle_errno!(FileStatError, from this,
                Errno::EBADF => (InvalidFileDescriptor, "{} since an invalid file-descriptor was provided.", msg),
                Errno::EIO => (IOerror, "{} due to an I/O error while reading from the file system.", msg),
                Errno::EOVERFLOW => (FileTooBig, "{} since the file size is so large that it cannot be represented by an internal structure.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        Ok(attr)
    }

    pub(crate) fn set_permission<T: FileDescriptorBased + Debug>(
        this: &T,
        permission: Permission,
    ) -> Result<(), FileSetPermissionError> {
        if unsafe { posix::fchmod(this.file_descriptor().native_handle(), permission.as_mode()) }
            == 0
        {
            return Ok(());
        }

        let msg = "Unable to update permission";
        handle_errno!(FileSetPermissionError, from this,
            Errno::EBADF => (InvalidFileDescriptor, "{} to {} since an invalid file-descriptor was provided.", msg,  permission),
            Errno::EPERM => (InsufficientPermissions, "{} {} due to insufficient permissions.", msg, permission),
            Errno::EROFS => (ReadOnlyFilesystem, "{} {} since the file resides on a read-only file system.", msg, permission),
            v => (UnknownError(v as i32), "{} {} due to an unknown error ({}).", msg, permission, v)
        );
    }

    pub(crate) fn set_ownership<T: FileDescriptorBased + Debug>(
        this: &T,
        uid: Uid,
        gid: Gid,
    ) -> Result<(), FileSetOwnerError> {
        if unsafe {
            posix::fchown(
                this.file_descriptor().native_handle(),
                uid.to_native(),
                gid.to_native(),
            )
        } == 0
        {
            return Ok(());
        }

        let msg = "Unable to update ownership";
        handle_errno!(FileSetOwnerError, from this,
            Errno::EBADF => (InvalidFileDescriptor, "{} to uid {}, gid {} since an invalid file-descriptor was provided.", msg, uid, gid),
            Errno::EPERM => (InsufficientPermissions, "{} to uid {}, gid {} due to insufficient permissions.", msg, uid, gid),
            Errno::EROFS => (ReadOnlyFilesystem, "{} to uid {}, gid {} since the file is located on an read-only filesystem.", msg, uid, gid),
            Errno::EINVAL => (InvalidId, "{} to uid {}, gid {} since the owner or group id is not a valid id.", msg, uid, gid),
            Errno::EIO => (IOerror, "{} to uid {}, gid {} due to an I/O error.", msg, uid, gid),
            Errno::EINTR => (Interrupt, "{} to uid {}, gid {} since an interrupt signal was received.", msg, uid, gid),
            v => (UnknownError(v as i32), "{} to uid {}, gid {} due to an unknown error ({}).", msg, uid, gid, v)
        );
    }
}

impl FileDescriptorBased for File {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl FileDescriptorManagement for File {}
