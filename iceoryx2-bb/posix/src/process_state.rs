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
//! let process_state_path = FilePath::new(b"/tmp/process_state_file").unwrap();
//!
//! // remove potentially uncleaned process state file that can remain when this process crashed
//! // before
//! //
//! // # Safety
//! //  * This process owns the state file, therefore it is safe to remove it.
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
//! let process_state_path = FilePath::new(b"/tmp/process_state_file").unwrap();
//!
//! let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
//!
//! match monitor.state().expect("") {
//!     // Process is alive and well
//!     ProcessState::Alive => (),
//!
//!     // The process state file is created, this should state should persist only a very small
//!     // fraction of time
//!     ProcessState::InInitialization => (),
//!
//!     // Process died, we have to inform other interested parties and maybe cleanup some
//!     // resources
//!     ProcessState::Dead => {
//!         // monitored process crashed, perform cleanup
//!
//!         // after cleanup is complete, we remove the process state file to signal that
//!         // everything is clean again
//!         //
//!         // # Safety
//!         //   * We ensured that the process is dead and therefore we can remove the old state.
//!         //     If you use a executor or something like systemd the task shall be done by the
//!         //     process itself.
//!         unsafe { ProcessGuard::remove(&process_state_path).expect("") };
//!     },
//!
//!     // The monitored process does not exist, maybe it did not yet start or already performed
//!     // a clean shutdown.
//!     ProcessState::DoesNotExist => (),
//! }
//! ```

pub use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{debug, error, fail, fatal_panic, trace, warn};
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
    InInitialization,
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
    Interrupt,
    UnknownError(i32),
}

/// Defines all errors that can occur when a new [`ProcessMonitor`] is created.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ProcessMonitorCreateError {
    InsufficientPermissions,
    Interrupt,
    IsDirectory,
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

/// A guard for a process that makes the process monitorable by a [`ProcessMonitor`] as long as it
/// is in scope. When it goes out of scope the process is no longer monitorable.
///
/// ```
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"/tmp/process_state_file").unwrap();
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
    file: Option<File>,
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        let path = *self.path();
        match self.file.take() {
            Some(f) => match f.remove_self() {
                Ok(true) => {
                    trace!(from "ProcessGuard::drop()", "remove process state \"{}\" for monitoring", path);
                }
                Ok(false) => {
                    warn!(from self, "Someone else already removed the owned ProcessGuard file.")
                }
                Err(v) => {
                    error!(from self, "Unable to remove the owned ProcessGuard file ({:?}). This can cause a process to be identified as dead despite it shut down correctly.", v);
                }
            },
            None => {
                fatal_panic!(from self, "This should never happen! The ProcessGuard contains an empty file!");
            }
        }
    }
}

const INIT_PERMISSION: Permission = Permission::OWNER_WRITE;
const FINAL_PERMISSION: Permission = Permission::OWNER_ALL;
impl ProcessGuard {
    /// Creates a new [`ProcessGuard`]. As soon as it is created successfully another process can
    /// monitor the state of the process. One cannot create multiple [`ProcessGuard`]s that use the
    /// same `path`. But one can create multiple [`ProcessGuard`]s that are using different
    /// `path`s.
    ///
    /// ```
    /// use iceoryx2_bb_posix::process_state::*;
    ///
    /// let process_state_path = FilePath::new(b"/tmp/process_state_file").unwrap();
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

        let default_directory_permissions = Permission::OWNER_ALL
            | Permission::GROUP_READ
            | Permission::GROUP_EXEC
            | Permission::OTHERS_READ
            | Permission::OTHERS_EXEC;

        let dir_path = path.path();
        match Directory::does_exist(&dir_path) {
            Ok(true) => (),
            Ok(false) => match Directory::create(&dir_path, default_directory_permissions) {
                Ok(_) | Err(DirectoryCreateError::DirectoryAlreadyExists) => (),
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

        let mut file = match FileBuilder::new(path)
            .creation_mode(CreationMode::CreateExclusive)
            .permission(INIT_PERMISSION)
            .create()
        {
            Ok(f) => f,
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
        };

        let mut new_lock_state = posix::flock::new();
        new_lock_state.l_type = LockType::Write as _;
        new_lock_state.l_whence = posix::SEEK_SET as _;

        if unsafe {
            posix::fcntl(
                file.file_descriptor().native_handle(),
                posix::F_SETLK,
                &mut new_lock_state,
            )
        } == -1
        {
            let errno = Errno::get();

            match file.remove_self() {
                Ok(_) => (),
                Err(v) => {
                    debug!(from origin,
                            "{} since the file could not be locked and the state file could not be removed again ({:?}).",
                            msg, v);
                }
            }

            match errno {
                Errno::EINTR => {
                    fail!(from origin, with ProcessGuardCreateError::Interrupt,
                    "{} since an interrupt signal was received while locking the file.", msg);
                }
                v => {
                    fail!(from origin, with ProcessGuardCreateError::UnknownError(v as _),
                    "{} since an unknown failure occurred while locking the file ({:?}).", msg, v);
                }
            }
        }

        match file.set_permission(FINAL_PERMISSION) {
            Ok(_) => {
                trace!(from "ProcessGuard::new()", "create process state \"{}\" for monitoring", path);
                Ok(Self { file: Some(file) })
            }
            Err(v) => {
                match file.remove_self() {
                    Ok(_) => (),
                    Err(v2) => {
                        debug!(from origin,
                            "{} since the final permissions could not be applied ({:?}) and the state file could not be removed again ({:?}).",
                            msg, v, v2);
                    }
                }
                fail!(from origin, with ProcessGuardCreateError::UnknownError(0),
                    "{} since the final permissions could not be applied due to an internal failure ({:?}).", msg, v);
            }
        }
    }

    /// Removes a stale process state from a crashed process.
    ///
    /// # Safety
    ///
    ///  * Ensure via the [`ProcessMonitor`] or other logic that the process is no longer running
    ///    otherwise you make the process unmonitorable.
    pub unsafe fn remove(path: &FilePath) -> Result<bool, FileRemoveError> {
        Ok(
            fail!(from "ProcessGuard::remove()", when File::remove(path),
            "Unable to remove ProcessGuard file."),
        )
    }

    /// Returns the [`FilePath`] under which the underlying file is stored.
    pub fn path(&self) -> &FilePath {
        match self.file.as_ref() {
            Some(f) => match f.path() {
                Some(path) => path,
                None => {
                    fatal_panic!(from self, "This should never happen! Unable to acquire path from underlying file.")
                }
            },
            None => {
                fatal_panic!(from self, "This should never happen! The underlying file is an empty optional.")
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
/// let process_state_path = FilePath::new(b"/tmp/process_state_file").unwrap();
///
/// let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
///
/// match monitor.state().expect("") {
///     // Process is alive and well
///     ProcessState::Alive => (),
///
///     // The process state file is created, this should state should persist only a very small
///     // fraction of time
///     ProcessState::InInitialization => (),
///
///     // Process died, we have to inform other interested parties and maybe cleanup some
///     // resources
///     ProcessState::Dead => {
///         // monitored process crashed, perform cleanup
///
///         // after cleanup is complete, we remove the process state file to signal that
///         // everything is clean again
///         //
///         // # Safety
///         //   * We ensured that the process is dead and therefore we can remove the old state.
///         //     If you use a executor or something like systemd the task shall be done by the
///         //     process itself.
///         unsafe { ProcessGuard::remove(&process_state_path).expect("") };
///     },
///
///     // The monitored process does not exist, maybe it did not yet start or already performed
///     // a clean shutdown.
///     ProcessState::DoesNotExist => (),
/// }
/// ```
#[derive(Debug)]
pub struct ProcessMonitor {
    file: Option<File>,
    path: FilePath,
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
    /// let process_state_path = FilePath::new(b"/tmp/process_state_file").unwrap();
    ///
    /// let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
    /// ```
    pub fn new(path: &FilePath) -> Result<Self, ProcessMonitorCreateError> {
        let mut new_self = Self {
            file: None,
            path: *path,
        };

        new_self.open_file()?;
        Ok(new_self)
    }

    /// Returns the path of the underlying file of the [`ProcessGuard`].
    pub fn path(&self) -> &FilePath {
        &self.path
    }

    /// Returns the current state of the process that is monitored.
    ///
    /// # Exampel
    ///
    /// ```
    /// use iceoryx2_bb_posix::process_state::*;
    ///
    /// let process_state_path = FilePath::new(b"/tmp/process_state_file").unwrap();
    ///
    /// let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
    ///
    /// match monitor.state().expect("") {
    ///     // Process is alive and well
    ///     ProcessState::Alive => (),
    ///
    ///     // The process state file is created, this should state should persist only a very small
    ///     // fraction of time
    ///     ProcessState::InInitialization => (),
    ///
    ///     // Process died, we have to inform other interested parties and maybe cleanup some
    ///     // resources
    ///     ProcessState::Dead => (),
    ///
    ///     // The monitored process does not exist, maybe it did not yet start or already performed
    ///     // a clean shutdown.
    ///     ProcessState::DoesNotExist => (),
    /// }
    /// ```
    pub fn state(&mut self) -> Result<ProcessState, ProcessMonitorStateError> {
        let msg = "Unable to acquire ProcessState";
        match self.file {
            Some(_) => self.read_state_from_file(),
            None => match File::does_exist(&self.path) {
                Ok(true) => {
                    fail!(from self, when self.open_file(),
                        "{} since the state file could not be opened.", msg);
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

    fn read_state_from_file(&mut self) -> Result<ProcessState, ProcessMonitorStateError> {
        let file = match self.file {
            Some(ref f) => f,
            None => return Ok(ProcessState::InInitialization),
        };

        let msg = "Unable to acquire ProcessState from file";
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
            handle_errno!(ProcessMonitorStateError, from self,
                Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            )
        }

        match current_state.l_type as _ {
            posix::F_WRLCK => Ok(ProcessState::Alive),
            _ => match File::does_exist(&self.path) {
                Ok(true) => match file.permission() {
                    Ok(INIT_PERMISSION) => Ok(ProcessState::InInitialization),
                    Err(_) | Ok(_) => {
                        self.file = None;
                        Ok(ProcessState::Dead)
                    }
                },
                Ok(false) => {
                    self.file = None;
                    Ok(ProcessState::DoesNotExist)
                }
                Err(v) => {
                    fail!(from self, with ProcessMonitorStateError::UnknownError(0),
                            "{} since an unknown failure occurred while checking if the process state file exists ({:?}).", msg, v);
                }
            },
        }
    }

    fn open_file(&mut self) -> Result<(), ProcessMonitorCreateError> {
        let origin = "ProcessMonitor::new()";
        let msg = format!(
            "Unable to open ProcessMonitor with the file \"{}\"",
            self.path
        );
        self.file = match FileBuilder::new(&self.path).open_existing(AccessMode::Read) {
            Ok(f) => Some(f),
            Err(FileOpenError::FileDoesNotExist) => None,
            Err(FileOpenError::IsDirectory) => {
                fail!(from origin, with ProcessMonitorCreateError::IsDirectory,
                    "{} since the path is a directory.", msg);
            }
            Err(FileOpenError::InsufficientPermissions) => {
                if FileBuilder::new(&self.path)
                    .open_existing(AccessMode::Write)
                    .is_ok()
                {
                    None
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
        };

        Ok(())
    }
}
