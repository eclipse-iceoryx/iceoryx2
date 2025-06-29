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

//! Create and read directory contents based on a POSIX api. It provides also advanced features
//! like [`Permission`] setting and to be created from a [`FileDescriptor`]
//!
//! # Examples
//! ```
//! use iceoryx2_bb_posix::directory::*;
//! use iceoryx2_bb_system_types::path::Path;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let dir_name = Path::new(b"a_dir_over_the_rainbow").unwrap();
//! let dir = if !Directory::does_exist(&dir_name).unwrap() {
//!   Directory::create(&dir_name, Permission::OWNER_ALL).unwrap()
//! } else {
//!   Directory::new(&dir_name).unwrap()
//! };
//!
//! let contents = dir.contents().unwrap();
//! for entry in contents {
//!   println!("{}", entry.name());
//! }
//!
//! Directory::remove(&dir_name).unwrap();
//! ```
use iceoryx2_bb_container::byte_string::strnlen;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary::scope_guard::ScopeGuardBuilder;
use iceoryx2_bb_log::{error, fail, fatal_panic, trace};
use iceoryx2_bb_system_types::{file_name::FileName, file_path::FilePath, path::Path};
use iceoryx2_pal_configuration::PATH_SEPARATOR;
use iceoryx2_pal_posix::posix::MemZeroedStruct;
use iceoryx2_pal_posix::*;
use iceoryx2_pal_posix::{posix::errno::Errno, posix::S_IFDIR};

use crate::file::{File, FileRemoveError};
use crate::file_type::FileType;
pub use crate::permission::Permission;
use crate::{config::EINTR_REPETITIONS, file_descriptor::*, metadata::*};

enum_gen! { DirectoryOpenError
  entry:
    LoopInSymbolicLinks,
    InsufficientPermissions,
    NotADirectory,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    DoesNotExist,
    UnknownError(i32)
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum DirectoryStatError {
    InsufficientPermissions,
    IOerror,
    DoesNotExist,
    PathPrefixIsNotADirectory,
    DataOverflowInStatStruct,
    LoopInSymbolicLinks,
    UnknownError(i32),
}

enum_gen! { DirectoryReadError
  entry:
    InsufficientPermissions,
    DirectoryDoesNoLongerExist,
    InsufficientMemory,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    UnknownError(i32)

  mapping:
    DirectoryStatError
}

enum_gen! { DirectoryCreateError
  entry:
    InsufficientPermissions,
    DirectoryAlreadyExists,
    LoopInSymbolicLinks,
    ExceedsParentsLinkCount,
    PartsOfThePathDoNotExist,
    PartsOfThePathAreNotADirectory,
    NoSpaceLeft,
    ReadOnlyFilesystem,
    UnknownError(i32)
  mapping:
    DirectoryOpenError
}

enum_gen! { DirectoryRemoveError
  entry:
    InsufficientPermissions,
    CurrentlyInUse,
    NotEmptyOrHardLinksPointingToTheDirectory,
    IOerror,
    LastComponentIsDot,
    LoopInSymbolicLinks,
    DirectoryDoesNotExist,
    NotADirectory,
    ResidesOnReadOnlyFileSystem,
    DanglingSymbolicLink,
    UnknownError(i32)
  mapping:
    DirectoryOpenError,
    DirectoryReadError,
    FileRemoveError
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum DirectoryAccessError {
    InsufficientPermissions,
    IOerror,
    PathPrefixIsNotADirectory,
    DataOverflowInStatStruct,
    LoopInSymbolicLinks,
    UnknownError(i32),
}

enum_gen! {
    /// The DirectoryError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward DirectoryError as more generic return value when a method
    /// returns a Directory***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    DirectoryError
  generalization:
    Create <= DirectoryCreateError,
    Open <= DirectoryOpenError,
    Read <= DirectoryReadError; DirectoryStatError,
    Remove <= DirectoryRemoveError
}

/// Represents an entry of a [`Directory`]. It provides the name of the entry and [`Metadata`] to
/// acquire additional informations about the entry.
///
/// # Example
///
/// ```
/// use iceoryx2_bb_posix::directory::*;
/// use iceoryx2_bb_system_types::path::Path;
/// use iceoryx2_bb_container::semantic_string::SemanticString;
///
/// let dir_name = Path::new(b"i_am_a_directory").unwrap();
/// let dir = Directory::create(&dir_name, Permission::OWNER_ALL).unwrap();
/// let content = dir.contents().unwrap();
///
/// for entry in content {
///   println!("name {}, type {:?}, size {}", entry.name(), entry.metadata().file_type(),
///     entry.metadata().size());
/// }
/// Directory::remove(&dir_name).unwrap();
/// ```
pub struct DirectoryEntry {
    name: FileName,
    metadata: Metadata,
}

impl DirectoryEntry {
    pub fn name(&self) -> &FileName {
        &self.name
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
}

/// Represents a directory implement based on the POSIX API. It implements the traits
/// [`FileDescriptorBased`] and [`FileDescriptorManagement`] to provide extended permission
/// and ownership handling as well as [`Metadata`].
#[derive(Debug)]
pub struct Directory {
    path: Path,
    directory_stream: *mut posix::DIR,
    file_descriptor: FileDescriptor,
}

impl Drop for Directory {
    fn drop(&mut self) {
        let mut counter = 0;
        loop {
            if unsafe { posix::closedir(self.directory_stream) } == 0 {
                break;
            }

            let msg = "Unable to close directory stream";
            match Errno::get() {
                Errno::EBADF => {
                    fatal_panic!(from self, "This should never happen! {} due to an invalid file-descriptor.", msg);
                }
                Errno::EINTR => {
                    counter += 1;
                    if counter > EINTR_REPETITIONS {
                        error!(from self, "{} since too many interrupt signals were received.", msg);
                    }
                }
                v => {
                    fatal_panic!(from self, "This should never happen! {} since an unknown error occurred ({}).", msg, v);
                }
            }

            if counter > EINTR_REPETITIONS {
                error!(from self, "Tried {} times to close the file but failed.", counter);
            }
        }
    }
}

impl Directory {
    pub fn new(path: &Path) -> Result<Self, DirectoryOpenError> {
        let directory_stream = unsafe { posix::opendir(path.as_c_str()) };

        let msg = format!("Unable to open directory \"{path}\"");
        if directory_stream.is_null() {
            handle_errno!(DirectoryOpenError, from "Directory::new",
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::ELOOP => (LoopInSymbolicLinks, "{} due to a loop in the symbolic links.", msg),
                Errno::ENOENT => (DoesNotExist, "{} since the path does not exist.", msg),
                Errno::ENOTDIR => (NotADirectory, "{} since the path is not a directory.", msg),
                Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the file descriptor limit of the process was reached.", msg),
                Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide limit of file descriptors was reached.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        let file_descriptor =
            FileDescriptor::non_owning_new(unsafe { posix::dirfd(directory_stream) });
        if file_descriptor.is_none() {
            fatal_panic!(from "Directory::new",
                "This should never happen! {} since 'dirfd' states that the acquired directory stream is invalid.", msg);
        }

        Ok(Directory {
            path: path.clone(),
            directory_stream,
            file_descriptor: file_descriptor.unwrap(),
        })
    }

    fn create_single_directory(
        path: &Path,
        permission: Permission,
    ) -> Result<(), DirectoryCreateError> {
        let msg = format!("Unable to create directory \"{path}\"");

        if unsafe { posix::mkdir(path.as_c_str(), permission.as_mode()) } == -1 {
            handle_errno!(DirectoryCreateError, from "Directory::create",
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::EEXIST => (DirectoryAlreadyExists, "{} since the directory already exists.", msg),
                Errno::ELOOP => (LoopInSymbolicLinks, "{} due to a loop in the symbolic links.", msg),
                Errno::EMLINK => (ExceedsParentsLinkCount, "{} since it would exceed the parents link count.", msg),
                Errno::ENOENT => (PartsOfThePathDoNotExist, "{} since parts of the path either do not exist.", msg),
                Errno::ENOSPC => (NoSpaceLeft, "{} since there is no space left on the target device.", msg),
                Errno::ENOTDIR => (PartsOfThePathAreNotADirectory, "{} since parts of the path are not a directory.", msg),
                Errno::EROFS => (ReadOnlyFilesystem, "{} since the parent directory resides on a read-only filesystem.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        Ok(())
    }

    /// Creates a new directory at the provided path.
    pub fn create(path: &Path, permission: Permission) -> Result<Self, DirectoryCreateError> {
        let origin = "Directory::create()";
        let msg = format!("Unable to create directory \"{path}\"");
        let entries = path.entries();

        let mut inc_path = if path.is_absolute() {
            Path::new_root_path()
        } else {
            Path::new_empty()
        };

        for entry in entries {
            inc_path
                .add_path_entry(&entry.into())
                .expect("Always works since it recreates the provided path");

            match Directory::does_exist(&inc_path) {
                Ok(true) => (),
                Ok(false) => match Directory::create_single_directory(&inc_path, permission) {
                    Ok(()) | Err(DirectoryCreateError::DirectoryAlreadyExists) => (),
                    Err(e) => {
                        fail!(from origin, with e,
                            "{} since the directory {} could not be created due to {:?}.",
                            msg, inc_path, e);
                    }
                },
                Err(DirectoryAccessError::InsufficientPermissions) => {
                    fail!(from origin, with DirectoryCreateError::InsufficientPermissions,
                        "{} since the path {} could not be accessed due to insufficient permissions.", msg, inc_path);
                }
                Err(DirectoryAccessError::PathPrefixIsNotADirectory) => {
                    fail!(from origin, with DirectoryCreateError::PartsOfThePathAreNotADirectory,
                        "{} since the path {} is not a directory.", msg, inc_path);
                }
                Err(v) => {
                    fail!(from origin, with DirectoryCreateError::UnknownError(0),
                        "{} due to a failure while accessing {} ({:?}).", msg, inc_path, v);
                }
            };
        }

        match Directory::new(path) {
            Ok(d) => {
                trace!(from d, "created");
                Ok(d)
            }
            Err(e) => {
                fail!(from origin, with e.into(),
                    "Failed to open newly created directory \"{}\".", path);
            }
        }
    }

    /// Returns the path of the directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Removes an empty directory. If the directory is not empty it returns an error.
    pub fn remove_empty(path: &Path) -> Result<(), DirectoryRemoveError> {
        if unsafe { posix::rmdir(path.as_c_str()) } == -1 {
            let msg = format!("Unable to remove empty directory \"{path}\"");
            handle_errno!(DirectoryRemoveError, from "Directory::remove",
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::EBUSY => (CurrentlyInUse, "{} since the directory is currently in use.", msg),
                Errno::EINVAL => (LastComponentIsDot, "{} since the last path component is \".\".", msg),
                Errno::ELOOP => (LoopInSymbolicLinks, "{} due to a loop in the symbolic links of the path \".\".", msg),
                Errno::ENOENT => (DanglingSymbolicLink, "{} since the path contains a dangling symbolic link.", msg),
                Errno::EEXIST => (DirectoryDoesNotExist, "{} since the directory does not exist.", msg),
                Errno::ENOTDIR => (NotADirectory, "{} since it is not a directory.", msg),
                Errno::EROFS => (ResidesOnReadOnlyFileSystem, "{} since the directory resides on a read only file system.", msg),
                Errno::ENOTEMPTY => (NotEmptyOrHardLinksPointingToTheDirectory, "{} since the directory is not empty or there are hard links pointing to the directory.", msg),
                Errno::EIO => (IOerror, "{} due to a physicial I/O error.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        trace!(from "Directory::remove", "removed \"{}\"", path);
        Ok(())
    }

    /// Removes and existing directory with all of its contents.
    pub fn remove(path: &Path) -> Result<(), DirectoryRemoveError> {
        let msg = format!("Unable to remove directory \"{path}\"");
        let origin = "Directory::remove()";

        let dir = fail!(from origin, when Directory::new(path),
                            "{} since the directory {} could not be opened.", msg, path);
        let contents = fail!(from origin, when dir.contents(),
                            "{} since the directory contents of {} could not be read.", msg, path);

        for entry in contents {
            let mut sub_path = path.clone();
            sub_path
                .add_path_entry(&entry.name().into())
                .expect("always a valid path entry");
            if entry.metadata().file_type() == FileType::Directory {
                fail!(from origin, when Directory::remove(&sub_path),
                    "{} since the sub-path {} could not be removed.", msg, sub_path);
            } else {
                fail!(from origin, when File::remove(&unsafe{FilePath::new_unchecked(sub_path.as_bytes())}),
                    "{} since the file {} could not be removed.", msg, sub_path);
            }
        }

        Self::remove_empty(path)
    }

    /// Returns the contents of the directory inside a vector of [`DirectoryEntry`]s.
    pub fn contents(&self) -> Result<Vec<DirectoryEntry>, DirectoryReadError> {
        let mut namelist: *mut *mut posix::types::dirent =
            core::ptr::null_mut::<*mut posix::types::dirent>();
        let number_of_directory_entries =
            unsafe { posix::scandir(self.path.as_c_str(), &mut namelist) };

        let _memory_cleanup_guard = ScopeGuardBuilder::new(namelist)
            .on_init(|_| {
                if number_of_directory_entries < 0 {
                    let msg = "Unable to read directory contents";
                    handle_errno!(DirectoryReadError, from self,
                        Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                        Errno::ENOENT => (DirectoryDoesNoLongerExist, "{} since the directory does not exist anymore.", msg),
                        Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                        Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the file descriptor limit of the process was reached.", msg),
                        Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since the system-wide limit of file descriptors was reached.", msg),
                        v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
                    );
                }

                Ok(())
            })
            .on_drop(|v| {
                for i in 0..number_of_directory_entries {
                    unsafe { posix::free(*(v.offset(i as isize)) as *mut posix::void) };
                }
                unsafe { posix::free(*v as *mut posix::void) };
            }).create()?;

        let mut contents: Vec<DirectoryEntry> = vec![];
        for i in 0..number_of_directory_entries {
            let raw_name =
                unsafe { (*(*namelist.offset(i as isize))).d_name.as_ptr() as *mut posix::c_char };
            let raw_name_length = unsafe { strnlen(raw_name, FileName::max_len()) };

            if raw_name_length == 0 {
                continue;
            }

            const DOT: posix::c_char = b'.' as _;
            // dot is skipped
            if raw_name_length == 1 && unsafe { *raw_name == DOT } {
                continue;
            }

            // dot dot is skipped
            if raw_name_length == 2
                && unsafe { *raw_name == DOT }
                && unsafe { *raw_name.offset(1) == DOT }
            {
                continue;
            }

            match unsafe { FileName::from_c_str(raw_name) } {
                Ok(name) => {
                    let msg = format!(
                        "Failed to acquire stats \"{name}\" while reading directory content"
                    );
                    match Self::acquire_metadata(self, &name, &msg) {
                        Ok(metadata) => contents.push(DirectoryEntry { name, metadata }),
                        Err(DirectoryStatError::DoesNotExist)
                        | Err(DirectoryStatError::InsufficientPermissions) => (),
                        Err(e) => {
                            fail!(from self, with e.into(),
                                    "{} due to an internal failure {:?}.", msg, e);
                        }
                    }
                }
                Err(v) => {
                    error!(from self, "Directory contains entries that are not representable with FileName struct ({:?}).", v);
                }
            }
        }

        Ok(contents)
    }

    /// Returns true if a directory already exists, otherwise false
    pub fn does_exist(path: &Path) -> Result<bool, DirectoryAccessError> {
        let mut buffer = posix::stat_t::new_zeroed();
        let msg = format!("Unable to determine if \"{path}\" does exist");

        if unsafe { posix::stat(path.as_c_str(), &mut buffer) } == -1 {
            handle_errno!(DirectoryAccessError, from "Directory::does_exist",
                success Errno::ENOENT => false,
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions to open path.", msg),
                Errno::EIO => (IOerror, "{} due to an io error while reading directory stats.", msg),
                Errno::ELOOP => (LoopInSymbolicLinks, "{} due to a symbolic link loop in the path.", msg),
                Errno::ENOTDIR => (PathPrefixIsNotADirectory, "{} since the path prefix is not a directory.", msg),
                Errno::EOVERFLOW => (DataOverflowInStatStruct, "{} since certain properties like size would cause an overflow in the underlying stat struct.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        Ok(buffer.st_mode & S_IFDIR != 0)
    }

    fn acquire_metadata(&self, file: &FileName, msg: &str) -> Result<Metadata, DirectoryStatError> {
        let mut buffer = posix::stat_t::new_zeroed();
        let mut path = self.path().clone();
        path.push(PATH_SEPARATOR).unwrap();
        path.push_bytes(file.as_bytes()).unwrap();

        if unsafe { posix::stat(path.as_c_str(), &mut buffer) } == -1 {
            handle_errno!(DirectoryStatError, from self,
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions to open path.", msg),
                Errno::EIO => (IOerror, "{} due to an io error while reading directory stats.", msg),
                Errno::ELOOP => (LoopInSymbolicLinks, "{} due to a symbolic link loop in the path.", msg),
                Errno::ENOENT => (DoesNotExist, "{} since the path does not exist.", msg),
                Errno::ENOTDIR => (PathPrefixIsNotADirectory, "{} since the path prefix is not a directory.", msg),
                Errno::EOVERFLOW => (DataOverflowInStatStruct, "{} since certain properties like size would cause an overflow in the underlying stat struct.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        Ok(Metadata::create(&buffer))
    }
}

impl FileDescriptorBased for Directory {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl FileDescriptorManagement for Directory {}
