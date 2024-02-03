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

//! Process monitoring via holding a file lock of a specific file. If the process crashes the
//! lock will be released by the operating system and another process can detect the crash. If the
//! process shutdowns correctly the file is removed and another process detects the clean shutdown.
//!
//! # Example
//!
//! ## Monitored Process
//!
//! ```
//! use iceoryx2_bb_posix::process_state::*;
//!
//! let process_state_path = FilePath::new(b"process_state_file").unwrap();
//!
//! // remove leftovers from a previous abnormal process termination
//! //
//! // # Safety
//! //  * This process owns the state file, therefore it is safe to remove a leftover from a
//! //    previous run.
//! unsafe { ProcessGuard::remove(&process_state_path).expect("") };
//!
//! // start monitoring from this point on
//! let guard = ProcessGuard::new(&process_state_path).expect("");
//!
//! // normal application code
//!
//! // stop monitoring
//! drop(guard);
//! ```
//!
//! ## Watchdog (Process That Monitors The State Of Other Processes)
//!
//! ```
//! use iceoryx2_bb_posix::process_state::*;
//!
//! let process_state_path = FilePath::new(b"process_state_file").unwrap();
//!
//! let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
//!
//! match monitor.state().expect("") {
//!     // Process is alive and well
//!     ProcessState::Alive => (),
//!
//!     // The process state file is created, this state should persist only a very small
//!     // fraction of time
//!     ProcessState::Starting => (),
//!
//!     // Process died, we have to inform other interested parties and maybe cleanup some
//!     // resources
//!     ProcessState::Dead => {
//!         // monitored process terminated abnormally, perform cleanup
//!
//!         match ProcessCleaner::new(&process_state_path) {
//!             Ok(guard) => {
//!                 // we own the old resources of the abnormally terminated process
//!                 // this is the place where we remove them
//!             }
//!             Err(ProcessCleanerCreateError::OwnedByAnotherProcess) => {
//!                 // Some other process is cleaning up all resources
//!             }
//!             Err(e) => {
//!                 // custom error handling
//!             },
//!         };
//!     },
//!     ProcessState::CleaningUp => (),
//!     // The monitored process does not exist, maybe it did not yet start or already performed
//!     // a clean shutdown.
//!     ProcessState::DoesNotExist => (),
//! }
//! ```

use std::fmt::Debug;

pub use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_container::semantic_string::SemanticStringError;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{fail, fatal_panic, trace};
pub use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_pal_posix::posix::{self, Errno, Struct};

use crate::{
    access_mode::AccessMode,
    directory::{Directory, DirectoryAccessError, DirectoryCreateError},
    file::{File, FileBuilder, FileCreationError, FileOpenError, FileRemoveError},
    file_descriptor::{FileDescriptorBased, FileDescriptorManagement},
    file_lock::LockType,
    permission::Permission,
    unix_datagram_socket::CreationMode,
};

/// Defines the current state of a process.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProcessState {
    Alive,
    Dead,
    DoesNotExist,
    Starting,
    CleaningUp,
}

/// Defines all errors that can occur when a new [`ProcessGuard`] is created.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProcessGuardCreateError {
    InsufficientPermissions,
    IsDirectory,
    InvalidDirectory,
    AlreadyExists,
    NoSpaceLeft,
    ReadOnlyFilesystem,
    ContractViolation,
    Interrupt,
    InvalidCleanerPathName,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum ProcessGuardLockError {
    OwnedByAnotherProcess,
    Interrupt,
    UnknownError(i32),
}

enum_gen! {
/// Defines all errors that can occur when a stale [`ProcessGuard`] is removed.
    ProcessGuardRemoveError
  entry:
    InsufficientPermissions,
    Interrupt,
    OwnedByAnotherProcess,
    InvalidCleanerPathName,
    UnknownError(i32)
  mapping:
    FileRemoveError
}

/// Defines all errors that can occur when a new [`ProcessMonitor`] is created.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ProcessMonitorCreateError {
    InsufficientPermissions,
    Interrupt,
    IsDirectory,
    InvalidCleanerPathName,
    UnknownError,
}

enum_gen! {
/// Defines all errors that can occur in [`ProcessMonitor::state()`].
    ProcessMonitorStateError
  entry:
    CorruptedState,
    Interrupt,
    UnknownError(i32)

  mapping:
    ProcessMonitorCreateError
}

/// Defines all errors that can occur when a new [`ProcessCleaner`] is created.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ProcessCleanerCreateError {
    ProcessIsStillAlive,
    OwnedByAnotherProcess,
    Interrupt,
    FailedToAcquireLockState,
    UnableToOpenStateFile,
    UnableToOpenCleanerFile,
    InvalidCleanerPathName,
    DoesNotExist,
    UnknownError,
}

/// A guard for a process that makes the process monitorable by a [`ProcessMonitor`] as long as it
/// is in scope. When it goes out of scope the process is no longer monitorable.
///
/// ```
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"process_state_file").unwrap();
///
/// // remove potentially uncleaned process state file that can remain when this process crashed
/// // before
/// //
/// // # Safety
/// //  * This process owns the state file, therefore it is safe to remove it.
/// unsafe { ProcessGuard::remove(&process_state_path).expect("") };
///
/// // start monitoring from this point on
/// let guard = ProcessGuard::new(&process_state_path).expect("");
///
/// // normal application code
///
/// // stop monitoring
/// drop(guard);
/// ```
#[derive(Debug)]
pub struct ProcessGuard {
    file: File,
    _cleaner_file: File,
}

const INIT_PERMISSION: Permission = Permission::OWNER_WRITE;
const FINAL_PERMISSION: Permission = Permission::OWNER_ALL;
const CLEANER_SUFFIX: &[u8] = b"_cleanup";

fn generate_cleaner_path(path: &FilePath) -> Result<FilePath, SemanticStringError> {
    let mut cleaner_path = *path;
    cleaner_path.push_bytes(CLEANER_SUFFIX)?;
    Ok(cleaner_path)
}

impl ProcessGuard {
    /// Creates a new [`ProcessGuard`]. As soon as it is created successfully another process can
    /// monitor the state of the process. One cannot create multiple [`ProcessGuard`]s that use the
    /// same `path`. But one can create multiple [`ProcessGuard`]s that are using different
    /// `path`s.
    ///
    /// ```
    /// use iceoryx2_bb_posix::process_state::*;
    ///
    /// let process_state_path = FilePath::new(b"process_state_file").unwrap();
    ///
    /// // start monitoring from this point on
    /// let guard = ProcessGuard::new(&process_state_path).expect("");
    /// ```
    pub fn new(path: &FilePath) -> Result<Self, ProcessGuardCreateError> {
        let origin = "ProcessGuard::new()";
        let msg = format!(
            "Unable to create new ProcessGuard with the file \"{}\"",
            path
        );

        let cleaner_path = match generate_cleaner_path(path) {
            Ok(f) => f,
            Err(e) => {
                fail!(from origin, with ProcessGuardCreateError::InvalidCleanerPathName,
                "{} since the corresponding cleaner path name would be invalid ({:?}).", msg, e);
            }
        };

        fail!(from origin, when Self::create_directory(path),
            "{} since the directory \"{}\" of the process guard could not be created", msg, path);

        let _cleaner_file = fail!(from origin, when Self::create_file(&cleaner_path, FINAL_PERMISSION),
                                    "{} since the cleaner file \"{}\" could not be created.", msg, cleaner_path);
        let mut file = fail!(from origin, when Self::create_file(path, INIT_PERMISSION),
                                "{} since the state file \"{}\" could not be created.", msg, path);

        match Self::lock_state_file(&file) {
            Ok(()) => (),
            Err(lock_error) => match lock_error {
                ProcessGuardLockError::Interrupt => {
                    fail!(from origin, with ProcessGuardCreateError::Interrupt,
                            "{} since an interrupt signal was received while locking the file.", msg);
                }
                ProcessGuardLockError::OwnedByAnotherProcess => {
                    fail!(from origin, with ProcessGuardCreateError::ContractViolation,
                            "{} since the another process holds the lock of a process state that is in initialization.", msg);
                }
                ProcessGuardLockError::UnknownError(v) => {
                    fail!(from origin, with ProcessGuardCreateError::UnknownError(v),
                            "{} since an unknown failure occurred while locking the file ({:?}).", msg, v);
                }
            },
        };

        match file.set_permission(FINAL_PERMISSION) {
            Ok(_) => {
                trace!(from "ProcessGuard::new()", "create process state \"{}\" for monitoring", path);
                Ok(Self {
                    file,
                    _cleaner_file,
                })
            }
            Err(v) => {
                fail!(from origin, with ProcessGuardCreateError::UnknownError(0),
                    "{} since the final permissions could not be applied due to an internal failure ({:?}).", msg, v);
            }
        }
    }

    fn create_directory(path: &FilePath) -> Result<(), ProcessGuardCreateError> {
        let origin = "ProcessGuard::create_directory()";
        let msg = format!(
            "Unable to create directory \"{}\" for new ProcessGuard state with the file \"{}\"",
            path.path(),
            path
        );

        let default_directory_permissions = Permission::OWNER_ALL
            | Permission::GROUP_READ
            | Permission::GROUP_EXEC
            | Permission::OTHERS_READ
            | Permission::OTHERS_EXEC;

        let dir_path = path.path();

        if dir_path.is_empty() {
            return Ok(());
        }

        match Directory::does_exist(&dir_path) {
            Ok(true) => Ok(()),
            Ok(false) => match Directory::create(&dir_path, default_directory_permissions) {
                Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => Ok(()),
                Err(DirectoryCreateError::InsufficientPermissions) => {
                    fail!(from origin, with ProcessGuardCreateError::InsufficientPermissions,
                    "{} since the directory {} could not be created due to insufficient permissions.",
                    msg, dir_path);
                }
                Err(DirectoryCreateError::ReadOnlyFilesystem) => {
                    fail!(from origin, with ProcessGuardCreateError::ReadOnlyFilesystem,
                    "{} since the directory {} could not be created since it is located on an read-only file system.",
                    msg, dir_path);
                }
                Err(DirectoryCreateError::NoSpaceLeft) => {
                    fail!(from origin, with ProcessGuardCreateError::NoSpaceLeft,
                    "{} since the directory {} could not be created since there is no space left.",
                    msg, dir_path);
                }
                Err(v) => {
                    fail!(from origin, with ProcessGuardCreateError::NoSpaceLeft,
                    "{} since the directory {} could not be created due to an unknown failure ({:?}).",
                    msg, dir_path, v);
                }
            },
            Err(DirectoryAccessError::InsufficientPermissions) => {
                fail!(from origin, with ProcessGuardCreateError::InsufficientPermissions,
                    "{} since the directory {} could not be accessed due to insufficient permissions.",
                    msg, dir_path);
            }
            Err(DirectoryAccessError::PathPrefixIsNotADirectory) => {
                fail!(from origin, with ProcessGuardCreateError::InvalidDirectory,
                    "{} since the directory {} is actually not a valid directory.", msg, dir_path);
            }
            Err(v) => {
                fail!(from origin, with ProcessGuardCreateError::UnknownError(0),
                    "{} since an unknown failure occurred ({:?}) while checking if directory {} exists.",
                    msg, v, dir_path);
            }
        }
    }

    fn create_file(
        path: &FilePath,
        permission: Permission,
    ) -> Result<File, ProcessGuardCreateError> {
        let origin = "ProcessGuard::file()";
        let msg = format!("Unable to create new ProcessGuard state file \"{}\"", path);

        match FileBuilder::new(path)
            .has_ownership(true)
            .creation_mode(CreationMode::CreateExclusive)
            .permission(permission)
            .create()
        {
            Ok(f) => Ok(f),
            Err(FileCreationError::InsufficientPermissions) => {
                fail!(from origin, with ProcessGuardCreateError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
            }
            Err(FileCreationError::FileAlreadyExists) => {
                fail!(from origin, with ProcessGuardCreateError::AlreadyExists,
                    "{} since the underlying file already exists.", msg);
            }
            Err(FileCreationError::IsDirectory) => {
                fail!(from origin, with ProcessGuardCreateError::IsDirectory,
                    "{} since the path is a directory.", msg);
            }
            Err(FileCreationError::NoSpaceLeft) => {
                fail!(from origin, with ProcessGuardCreateError::NoSpaceLeft,
                    "{} since there is no space left on the device.", msg);
            }
            Err(FileCreationError::FilesytemIsReadOnly) => {
                fail!(from origin, with ProcessGuardCreateError::ReadOnlyFilesystem,
                    "{} since the file system is read only.", msg);
            }
            Err(v) => {
                fail!(from origin, with ProcessGuardCreateError::UnknownError(0),
                    "{} due to an internal failure ({:?}).", msg, v);
            }
        }
    }

    fn lock_state_file(file: &File) -> Result<(), ProcessGuardLockError> {
        let msg = format!("Unable to lock process state file {:?}", file);
        let mut new_lock_state = posix::flock::new();
        new_lock_state.l_type = LockType::Write as _;
        new_lock_state.l_whence = posix::SEEK_SET as _;

        if unsafe {
            posix::fcntl(
                file.file_descriptor().native_handle(),
                posix::F_SETLK,
                &mut new_lock_state,
            )
        } != -1
        {
            return Ok(());
        }

        handle_errno!(ProcessGuardLockError, from "ProcessState::lock_state_file()",
            Errno::EACCES => (OwnedByAnotherProcess, "{} since the lock is owned by another process.", msg),
            Errno::EAGAIN => (OwnedByAnotherProcess, "{} since the lock is owned by another process.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            v => (UnknownError(v as i32), "{} due to an unknown failure.", msg)
        );
    }

    /// Removes a stale process state from a crashed process.
    ///
    /// # Safety
    ///
    ///  * Ensure via the [`ProcessMonitor`] or other logic that the process is no longer running
    ///    otherwise you make the process unmonitorable.
    ///  * Ensure that no [`ProcessCleaner`] is active. Otherwise, multiple sources may remove the
    ///    same constructs.
    pub unsafe fn remove(path: &FilePath) -> Result<bool, ProcessGuardRemoveError> {
        let msg = format!(
            "Unable to remove remainings of the ProcessState \"{}\"",
            path
        );
        let origin = "ProcessMonitor::remove()";
        let cleaner_path = match generate_cleaner_path(path) {
            Ok(f) => f,
            Err(e) => {
                fail!(from origin, with ProcessGuardRemoveError::InvalidCleanerPathName,
                "{} since the corresponding cleaner path name would be invalid ({:?}).", msg, e);
            }
        };

        Ok(
            fail!(from "ProcessGuard::remove()", when File::remove(path),
                "{}.", msg)
                && fail!(from "ProcessGuard::remove()", when File::remove(&cleaner_path),
                "{}.", msg),
        )
    }

    /// Returns the [`FilePath`] under which the underlying file is stored.
    pub fn path(&self) -> &FilePath {
        match self.file.path() {
            Some(path) => path,
            None => {
                fatal_panic!(from self, "This should never happen! Unable to acquire path from underlying file.")
            }
        }
    }
}

/// Monitor processes that have created a [`ProcessGuard`]. If the process dies, shutdowns or is
/// alive the monitor will detect it.
///
/// # Example
///
/// ```
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"process_state_file").unwrap();
///
/// let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
///
/// match monitor.state().expect("") {
///     // Process is alive and well
///     ProcessState::Alive => (),
///
///     // The process state file is created, this should state should persist only a very small
///     // fraction of time
///     ProcessState::Starting => (),
///
///     // Process died, we have to inform other interested parties and maybe cleanup some
///     // resources
///     ProcessState::Dead => {
///         // monitored process crashed, perform cleanup
///
///         match ProcessCleaner::new(&process_state_path) {
///             Ok(guard) => {
///                 // we own the old resources of the abnormally terminated process
///                 // this is the place where we remove them
///             }
///             Err(ProcessCleanerCreateError::OwnedByAnotherProcess) => {
///                 // Some other process is cleaning up all resources
///             }
///             Err(e) => {
///                 // custom error handling
///             },
///         };
///     },
///
///     // Process dies and another process is performing the cleanup currently
///     ProcessState::CleaningUp => (),
///
///     // The monitored process does not exist, maybe it did not yet start or already performed
///     // a clean shutdown.
///     ProcessState::DoesNotExist => (),
/// }
/// ```
pub struct ProcessMonitor {
    file: core::cell::Cell<Option<File>>,
    path: FilePath,
    cleaner_path: FilePath,
}

impl Debug for ProcessMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProcessMonitor {{ file = {:?}, path = {:?}}}",
            unsafe { &*self.file.as_ptr() },
            self.path
        )
    }
}

impl ProcessMonitor {
    /// Creates a new [`ProcessMonitor`] that can obtain the state of the process that will be
    /// monitored.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2_bb_posix::process_state::*;
    ///
    /// let process_state_path = FilePath::new(b"process_state_file").unwrap();
    ///
    /// let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
    /// ```
    pub fn new(path: &FilePath) -> Result<Self, ProcessMonitorCreateError> {
        let msg = format!("Unable to open process monitor \"{}\"", path);
        let origin = "ProcessMonitor::new()";
        let cleaner_path = match generate_cleaner_path(path) {
            Ok(f) => f,
            Err(e) => {
                fail!(from origin, with ProcessMonitorCreateError::InvalidCleanerPathName,
                "{} since the corresponding cleaner path name would be invalid ({:?}).", msg, e);
            }
        };

        let new_self = Self {
            file: core::cell::Cell::new(None),
            path: *path,
            cleaner_path,
        };

        new_self.file.set(Self::open_file(&new_self.path)?);
        Ok(new_self)
    }

    /// Returns the path of the underlying file of the [`ProcessGuard`].
    pub fn path(&self) -> &FilePath {
        &self.path
    }

    /// Returns the current state of the process that is monitored.
    ///
    /// # Example
    ///
    /// ```
    /// use iceoryx2_bb_posix::process_state::*;
    ///
    /// let process_state_path = FilePath::new(b"process_state_file").unwrap();
    ///
    /// let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
    ///
    /// match monitor.state().expect("") {
    ///     // Process is alive and well
    ///     ProcessState::Alive => (),
    ///
    ///     // The process state file is created, this should state should persist only a very small
    ///     // fraction of time
    ///     ProcessState::Starting => (),
    ///
    ///     // Process died, we have to inform other interested parties and maybe cleanup some
    ///     // resources
    ///     ProcessState::Dead => (),
    ///
    ///     // The monitored process does not exist, maybe it did not yet start or already performed
    ///     // a clean shutdown.
    ///     ProcessState::DoesNotExist => (),
    ///
    ///     // The monitored process crashed and another process acquired the [`ProcessCleaner`]
    ///     // to remove its remaining resources
    ///     ProcessState::CleaningUp => (),
    /// }
    /// ```
    pub fn state(&self) -> Result<ProcessState, ProcessMonitorStateError> {
        let msg = "Unable to acquire ProcessState";
        match unsafe { &*self.file.as_ptr() } {
            Some(_) => self.read_state_from_file(),
            None => match File::does_exist(&self.path) {
                Ok(true) => {
                    self.file.set(Self::open_file(&self.path)?);
                    self.read_state_from_file()
                }
                Ok(false) => Ok(ProcessState::DoesNotExist),
                Err(v) => {
                    fail!(from self, with ProcessMonitorStateError::UnknownError(0),
                        "{} since an unknown failure occurred while checking if the file exists ({:?}).", msg, v);
                }
            },
        }
    }

    fn get_lock_state(file: &File) -> Result<i64, ProcessMonitorStateError> {
        let msg = format!("Unable to acquire lock on file {:?}", file);
        let mut current_state = posix::flock::new();
        current_state.l_type = LockType::Write as _;

        if unsafe {
            posix::fcntl(
                file.file_descriptor().native_handle(),
                posix::F_GETLK,
                &mut current_state,
            )
        } == -1
        {
            handle_errno!(ProcessMonitorStateError, from "ProcessMonitor::get_lock_state()",
                Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            )
        }

        Ok(current_state.l_type as _)
    }

    fn read_state_from_file(&self) -> Result<ProcessState, ProcessMonitorStateError> {
        let file = match unsafe { &*self.file.as_ptr() } {
            Some(ref f) => f,
            None => return Ok(ProcessState::Starting),
        };

        let msg = format!("Unable to read state from file {:?}", file);
        let lock_state = fail!(from self, when Self::get_lock_state(file),
                            "{} since the lock state of the state file could not be acquired.", msg);

        match lock_state as _ {
            posix::F_WRLCK => Ok(ProcessState::Alive),
            _ => match File::does_exist(&self.path) {
                Ok(true) => match file.permission() {
                    Ok(INIT_PERMISSION) => Ok(ProcessState::Starting),
                    Err(_) | Ok(_) => {
                        self.file.set(None);
                        match Self::open_file(&self.cleaner_path)? {
                            Some(f) => {
                                let lock_state = fail!(from self, when Self::get_lock_state(&f),
                                                "{} since the lock state of the cleaner file could not be acquired.", msg);
                                if lock_state == posix::F_WRLCK as _ {
                                    return Ok(ProcessState::CleaningUp);
                                }

                                Ok(ProcessState::Dead)
                            }
                            None => {
                                fail!(from self, with ProcessMonitorStateError::CorruptedState,
                                    "{} since the corresponding cleaner file \"{}\" does not exist. This indicates a corrupted state.",
                                    msg, self.cleaner_path);
                            }
                        }
                    }
                },
                Ok(false) => {
                    self.file.set(None);
                    Ok(ProcessState::DoesNotExist)
                }
                Err(v) => {
                    fail!(from self, with ProcessMonitorStateError::UnknownError(0),
                            "{} since an unknown failure occurred while checking if the process state file exists ({:?}).", msg, v);
                }
            },
        }
    }

    fn open_file(path: &FilePath) -> Result<Option<File>, ProcessMonitorCreateError> {
        let origin = "ProcessMonitor::new()";
        let msg = format!("Unable to open ProcessMonitor state file \"{}\"", path);

        match FileBuilder::new(path).open_existing(AccessMode::Read) {
            Ok(f) => Ok(Some(f)),
            Err(FileOpenError::FileDoesNotExist) => Ok(None),
            Err(FileOpenError::IsDirectory) => {
                fail!(from origin, with ProcessMonitorCreateError::IsDirectory,
                    "{} since the path is a directory.", msg);
            }
            Err(FileOpenError::InsufficientPermissions) => {
                if FileBuilder::new(path)
                    .open_existing(AccessMode::Write)
                    .is_ok()
                {
                    Ok(None)
                } else {
                    fail!(from origin, with ProcessMonitorCreateError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
                }
            }
            Err(FileOpenError::Interrupt) => {
                fail!(from origin, with ProcessMonitorCreateError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(v) => {
                fail!(from origin, with ProcessMonitorCreateError::UnknownError,
                    "{} since an unknown failure occurred ({:?}).", msg, v);
            }
        }
    }
}

/// A guard for the remains of an abnormal terminated process. The instance that owns the
/// [`ProcessCleaner`] is allowed to cleanup all resources - no one else is. When it goes out of
/// scope it will remove all state files that were created with [`ProcessGuard`].
/// When the process that owns the [`ProcessCleaner`] terminates abnormally as well, the
/// [`ProcessCleaner`] guard can be acquired by another process again.
///
/// ```no_run
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"process_state_file").unwrap();
///
/// match ProcessCleaner::new(&process_state_path) {
///     Ok(guard) => {/* cleanup all process resources */},
///     Err(_) => (),
/// }
/// ```
pub struct ProcessCleaner {
    _file: File,
    _cleaner_file: File,
}

impl ProcessCleaner {
    /// Creates a new [`ProcessCleaner`]. Succeeds when the process that creates the state files
    /// with the [`ProcessGuard`] died an no other process has acquired the resources for cleanup
    /// with [`ProcessCleaner::new()`].
    pub fn new(path: &FilePath) -> Result<Self, ProcessCleanerCreateError> {
        let msg = format!("Unable to instantiate ProcessCleaner \"{}\"", path);
        let origin = "ProcessCleaner::new()";
        let cleaner_path = match generate_cleaner_path(path) {
            Ok(f) => f,
            Err(e) => {
                fail!(from origin, with ProcessCleanerCreateError::InvalidCleanerPathName,
                "{} since the corresponding cleaner path name would be invalid ({:?}).", msg, e);
            }
        };

        let mut cleaner_file = match fail!(from origin, when ProcessMonitor::open_file(&cleaner_path),
            with ProcessCleanerCreateError::UnableToOpenCleanerFile,
            "{} since the cleaner file could not be opened.", msg)
        {
            Some(f) => f,
            None => {
                fail!(from origin, with ProcessCleanerCreateError::DoesNotExist,
                "{} since the process cleaner file does not exist.", msg);
            }
        };

        let mut file = match fail!(from origin, when ProcessMonitor::open_file(path),
            with ProcessCleanerCreateError::UnableToOpenStateFile,
            "{} since the state file could not be opened.", msg)
        {
            Some(f) => f,
            None => {
                fail!(from origin, with ProcessCleanerCreateError::DoesNotExist,
                "{} since the process state file does not exist.", msg);
            }
        };

        let lock_state = fail!(from origin, when ProcessMonitor::get_lock_state(&file),
            with ProcessCleanerCreateError::FailedToAcquireLockState,
            "{} since the lock state could not be acquired.", msg);

        if lock_state == posix::F_WRLCK as _ {
            fail!(from origin, with ProcessCleanerCreateError::ProcessIsStillAlive,
                "{} since the corresponding process is still alive.", msg);
        }

        match ProcessGuard::lock_state_file(&cleaner_file) {
            Ok(()) => {
                file.acquire_ownership();
                cleaner_file.acquire_ownership();
                Ok(Self {
                    _file: file,
                    _cleaner_file: cleaner_file,
                })
            }
            Err(ProcessGuardLockError::OwnedByAnotherProcess) => {
                fail!(from origin, with ProcessCleanerCreateError::OwnedByAnotherProcess,
                    "{} since another process already has instantiated a ProcessCleaner.", msg);
            }
            Err(ProcessGuardLockError::Interrupt) => {
                fail!(from origin, with ProcessCleanerCreateError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(e) => {
                fail!(from origin, with ProcessCleanerCreateError::UnknownError,
                    "{} due to an unknown failure ({:?}).", msg, e);
            }
        }
    }
}
