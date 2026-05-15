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
//! ## Application (That Shall Be Monitored)
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::process_state::*;
//!
//! let process_state_path = FilePath::new(b"process_state_file").unwrap();
//!
//! // monitoring is enabled as soon as the guard object is created
//! let guard = match ProcessGuardBuilder::new().create(&process_state_path) {
//!     Ok(guard) => guard,
//!     Err(ProcessGuardCreateError::AlreadyExists) => {
//!         // process is dead and we have to cleanup all resources
//!         match ProcessCleaner::new(&process_state_path) {
//!             Ok(cleaner) => {
//!                 // we own the stale resources and have to remove them
//!                 // as soon as the guard goes out of scope the process state file is removed
//!                 drop(cleaner);
//!
//!                 match ProcessGuardBuilder::new().create(&process_state_path) {
//!                     Ok(guard) => guard,
//!                     Err(_) => {
//!                         panic!("Perform here some error handling");
//!                     }
//!                 }
//!             }
//!             Err(ProcessCleanerCreateError::OwnedByAnotherProcess) => {
//!                 // cool, someone else has instantiated it and is already cleaning up all resources
//!                 // wait a bit and try again
//!                 std::thread::sleep(core::time::Duration::from_millis(500));
//!                 match ProcessGuardBuilder::new().create(&process_state_path) {
//!                     Ok(guard) => guard,
//!                     Err(_) => {
//!                         panic!("Perform here some error handling");
//!                     }
//!                 }
//!             }
//!             Err(_) => {
//!                 panic!("Perform here some error handling");
//!             }
//!         }
//!     }
//!     Err(_) => {
//!         panic!("Perform here some error handling");
//!     }
//! };
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
//! # extern crate iceoryx2_bb_loggers;
//!
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
//!
//! ## Cleanup (Process That Removes Stale Resources)
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_posix::process_state::*;
//!
//! let process_state_path = FilePath::new(b"process_state_file").unwrap();
//!
//! match ProcessCleaner::new(&process_state_path) {
//!     Ok(_guard) => {
//!         // we own the stale resources and have to remove them
//!         // as soon as the _guard goes out of scope the process state file is removed
//!     }
//!     Err(ProcessCleanerCreateError::OwnedByAnotherProcess) => {
//!         // cool, someone else has instantiated it and is already cleaning up all resources
//!     }
//!     Err(_) => (),
//! }
//! ```

use alloc::format;
use core::fmt::Debug;
use core::ptr::NonNull;
use iceoryx2_bb_elementary_traits::non_null::NonNullCompat;
use iceoryx2_bb_elementary_traits::testing::abandonable::Abandonable;
use iceoryx2_bb_elementary_traits::zeroable::Zeroable;

pub use iceoryx2_bb_container::semantic_string::SemanticString;
pub use iceoryx2_bb_system_types::file_path::FilePath;

use iceoryx2_bb_container::semantic_string::SemanticStringError;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_log::{fail, fatal_panic, trace, warn};
use iceoryx2_pal_posix::posix::{self, Errno, MemZeroedStruct};

use crate::{
    access_mode::AccessMode,
    directory::{Directory, DirectoryAccessError, DirectoryCreateError},
    file::{File, FileBuilder, FileCreationError, FileOpenError, FileRemoveError, FileStatError},
    file_descriptor::{FileDescriptorBased, FileDescriptorManagement},
    file_lock::LockType,
    permission::Permission,
    process::{Process, UniqueProcessId},
    unix_datagram_socket::CreationMode,
};

const INIT_PERMISSION: Permission = Permission::OWNER_WRITE;
const OWNER_LOCK_SUFFIX: &[u8] = b"_owner_lock";
const CONTEXT_SUFFIX: &[u8] = b"_context";

/// Defines the current state of a process.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProcessState {
    Alive,
    Dead,
    DoesNotExist,
    Starting,
    CleaningUp,
}

enum_gen! {
    /// Defines all errors that can occur when a new [`ProcessGuard`] is created.
    ProcessGuardCreateError
  entry:
    InsufficientPermissions,
    IsDirectory,
    InvalidDirectory,
    AlreadyExists,
    NoSpaceLeft,
    ReadOnlyFilesystem,
    ContractViolation,
    Interrupt,
    InvalidCleanerPathName,
    FailedToWriteUniqueProcessIdIntoContextFile,
    FailedToAcquireUniqueProcessId,
    FailedToAcquireStateFileMetaData,
    SystemCorrupted,
    UnknownError(i32)
}

enum_gen! { ProcessGuardLockError
  entry:
    OwnedByAnotherProcess,
    FailedToAcquireStateFileMetaData,
    FileRemovedFromFileSystem,
    Interrupt,
    UnknownError(i32)
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

enum_gen! {
    /// Defines all errors that can occur when a new [`ProcessMonitor`] is created.
    ProcessMonitorCreateError
  entry:
    InvalidCleanerPathName
}

enum_gen! {
    /// Defines all errors that can occur when a new [`ProcessMonitor`] opens a management file.
    ProcessMonitorOpenError
  entry:
    InsufficientPermissions,
    Interrupt,
    IsDirectory,
    UnknownError
}

enum_gen! {
    /// Defines all errors that can occur in [`ProcessMonitor::state()`].
    ProcessMonitorStateError
  entry:
    CorruptedState,
    Interrupt,
    FailedToAcquireUniqueProcessIdFromContextFile,
    FailedToAcquireUniqueProcessId,
    UnknownError(i32)

  mapping:
    ProcessMonitorOpenError,
    FileStatError
}

enum_gen! {
    /// Defines all errors that can occur when a new [`ProcessCleaner`] is created.
    ProcessCleanerCreateError
  entry:
    ProcessIsStillAlive,
    ProcessIsInitializedOrCrashedDuringInitialization,
    ProcessIsBeingCleanedUpOrCrashedDuringCleanup,
    OwnedByAnotherProcess,
    Interrupt,
    FailedToAcquireLockState,
    FailedToAcquireUniqueProcessId,
    FailedToAcquireUniqueProcessIdFromContextFile,
    UnableToOpenContextFile,
    UnableToOpenStateFile,
    UnableToOpenOwnerLockFile,
    InvalidCleanerPathName,
    DoesNotExist,
    UnknownError

  mapping:
    ProcessMonitorStateError
}

/// The builder of the [`ProcessGuard`]
/// ```
/// # extern crate iceoryx2_bb_loggers;
///
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"process_state_file").unwrap();
///
/// // start monitoring from this point on
/// let guard = ProcessGuardBuilder::new().create(&process_state_path).expect("");
///
/// // normal application code
///
/// // stop monitoring
/// drop(guard);
/// ```
#[derive(Debug)]
pub struct ProcessGuardBuilder {
    directory_permissions: Permission,
    guard_permissions: Permission,
}

impl Default for ProcessGuardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessGuardBuilder {
    /// Creates a new instance
    pub fn new() -> Self {
        Self {
            directory_permissions: Permission::OWNER_ALL
                | Permission::GROUP_READ
                | Permission::GROUP_EXEC
                | Permission::OTHERS_READ
                | Permission::OTHERS_EXEC,
            guard_permissions: Permission::OWNER_READ | Permission::OWNER_WRITE,
        }
    }

    /// Defines the [`Directory`] [`Permission`]s of the [`Directory`] that
    /// will be created when the [`Directory`] from the provided [`FilePath`]
    /// does not exist.
    pub fn directory_permissions(mut self, value: Permission) -> Self {
        self.directory_permissions = value;
        self
    }

    /// Defines the [`Permission`]s of the [`ProcessGuard`].
    pub fn guard_permissions(mut self, value: Permission) -> Self {
        self.guard_permissions = value;
        self
    }

    fn create_context_permissions(&self) -> Permission {
        let mut permission = Permission::OWNER_READ;
        if self.guard_permissions.has(Permission::GROUP_READ) {
            permission |= Permission::GROUP_READ;
        }
        if self.guard_permissions.has(Permission::OTHERS_READ) {
            permission |= Permission::OTHERS_READ;
        }

        permission
    }

    /// Creates a new [`ProcessGuard`]. As soon as it is created successfully another process can
    /// monitor the state of the process. One cannot create multiple [`ProcessGuard`]s that use the
    /// same `path`. But one can create multiple [`ProcessGuard`]s that are using different
    /// `path`s.
    ///
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
    /// use iceoryx2_bb_posix::process_state::*;
    ///
    /// let process_state_path = FilePath::new(b"process_state_file").unwrap();
    ///
    /// // start monitoring from this point on
    /// let guard = ProcessGuardBuilder::new().create(&process_state_path).expect("");
    /// ```
    pub fn create(self, path: &FilePath) -> Result<ProcessGuard, ProcessGuardCreateError> {
        let origin = "ProcessGuard::new()";
        let msg = format!("Unable to create new ProcessGuard with the file \"{path}\"");

        let context_path = generate_context_path(path)
            .map_err(|_| ProcessGuardCreateError::InvalidCleanerPathName)?;
        let owner_lock_path = generate_owner_lock_path(path)
            .map_err(|_| ProcessGuardCreateError::InvalidCleanerPathName)?;

        fail!(from origin, when Self::create_directory(path, self.directory_permissions),
            "{} since the directory \"{}\" of the process guard could not be created", msg, path);

        let mut context_file = fail!(from origin, when Self::create_file(&context_path, INIT_PERMISSION),
                                "{msg} since the context file \"{context_path}\" could not be created.");

        let mut state_file = fail!(from origin, when Self::create_file(path, INIT_PERMISSION),
                                "{} since the state file \"{}\" could not be created.", msg, path);

        let mut owner_lock_file = fail!(from origin, when Self::create_file(&owner_lock_path, INIT_PERMISSION),
                                    "{} since the owner_lock file \"{}\" could not be created.", msg, owner_lock_path);

        let unique_process_id = fail!(from origin,
                                      when Process::unique_id(),
                                      with ProcessGuardCreateError::FailedToAcquireUniqueProcessId,
                                      "{msg} since the unique process id could not be acquired.");

        fail!(from origin,
              when context_file.write_val(&unique_process_id.value()),
              with ProcessGuardCreateError::FailedToWriteUniqueProcessIdIntoContextFile,
              "{msg} since the unique process could not be written to the owner file.");

        match Self::set_lock(&state_file, LockType::Write) {
            Ok(()) => (),
            Err(lock_error) => match lock_error {
                ProcessGuardLockError::Interrupt => {
                    fail!(from origin, with ProcessGuardCreateError::Interrupt,
                            "{} since an interrupt signal was received while locking the file.", msg);
                }
                ProcessGuardLockError::FailedToAcquireStateFileMetaData => {
                    fail!(from origin, with ProcessGuardCreateError::FailedToAcquireStateFileMetaData,
                        "{msg} since the state file's metadata could not be acquired.");
                }
                ProcessGuardLockError::FileRemovedFromFileSystem => {
                    fail!(from origin, with ProcessGuardCreateError::SystemCorrupted,
                        "{msg} since an external process removed the state file while it was being initialized.");
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

        if let Err(e) = owner_lock_file.set_permission(self.guard_permissions) {
            fail!(from origin, with ProcessGuardCreateError::UnknownError(0),
                "{msg} since the final permissions could not be applied to the owner file \"{owner_lock_path}\" due to an internal failure. [{e:?}]");
        }

        if let Err(e) = state_file.set_permission(self.guard_permissions) {
            fail!(from origin, with ProcessGuardCreateError::UnknownError(0),
                "{msg} since the final permissions could not be applied to the state file \"{path}\" due to an internal failure. [{e:?}]");
        }

        match context_file.set_permission(self.create_context_permissions()) {
            Ok(()) => {
                trace!(from "ProcessGuard::new()", "create process state \"{}\" for monitoring", path);
                Ok(ProcessGuard {
                    files: StateFiles {
                        state: Some(state_file),
                        owner_lock: Some(owner_lock_file),
                        context: Some(context_file),
                    },
                })
            }
            Err(v) => {
                fail!(from origin, with ProcessGuardCreateError::UnknownError(0),
                    "{} since the final permissions could not be applied to the context file \"{context_path}\" due to an internal failure ({:?}).", msg, v);
            }
        }
    }

    fn create_directory(
        path: &FilePath,
        permissions: Permission,
    ) -> Result<(), ProcessGuardCreateError> {
        let origin = "ProcessGuard::create_directory()";
        let msg = format!(
            "Unable to create directory \"{}\" for new ProcessGuard state with the file \"{}\"",
            path.path(),
            path
        );

        let dir_path = path.path();

        if dir_path.is_empty() {
            return Ok(());
        }

        match Directory::does_exist(&dir_path) {
            Ok(true) => Ok(()),
            Ok(false) => match Directory::create(&dir_path, permissions) {
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
        let msg = format!("Unable to create new ProcessGuard state file \"{path}\"");

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

    fn set_lock(file: &File, lock_type: LockType) -> Result<(), ProcessGuardLockError> {
        let origin = "ProcessState::set_lock()";
        let msg = format!("Unable to lock process state file {file:?}");
        let mut new_lock_state = posix::flock::new_zeroed();
        new_lock_state.l_type = lock_type as _;
        new_lock_state.l_whence = posix::SEEK_SET as _;

        if unsafe {
            posix::fcntl(
                file.file_descriptor().native_handle(),
                posix::F_SETLK,
                &mut new_lock_state,
            )
        } != -1
        {
            let metadata = match file.metadata() {
                Ok(metadata) => metadata,
                Err(e) => {
                    fail!(from origin, with ProcessGuardLockError::FailedToAcquireStateFileMetaData,
                        "{msg} since the metadata of the state file could not be acquired. [{e:?}]");
                }
            };

            if metadata.number_of_links() == 0 {
                fail!(from origin, with ProcessGuardLockError::FileRemovedFromFileSystem,
                    "{msg} since the state file was removed from the file system.");
            }

            return Ok(());
        }

        handle_errno!(ProcessGuardLockError, from origin,
            Errno::EACCES => (OwnedByAnotherProcess, "{} since the lock is owned by another process.", msg),
            Errno::EAGAIN => (OwnedByAnotherProcess, "{} since the lock is owned by another process.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            v => (UnknownError(v as i32), "{} due to an unknown failure (errno code: {}).", msg, v)
        );
    }
}

#[derive(Debug)]
struct StateFiles {
    state: Option<File>,
    owner_lock: Option<File>,
    context: Option<File>,
}

impl StateFiles {
    fn release_ownership(self) {
        self.context
            .as_ref()
            .expect("contains always a value, only removed on destruction")
            .release_ownership();
        self.state
            .as_ref()
            .expect("contains always a value, only removed on destruction")
            .release_ownership();
        self.owner_lock
            .as_ref()
            .expect("contains always a value, only removed on destruction")
            .release_ownership();
    }
}

impl Abandonable for StateFiles {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        for file in [&mut this.state, &mut this.owner_lock, &mut this.context] {
            if let Some(f) = file.take() {
                f.abandon();
            }
        }
    }
}

impl Drop for StateFiles {
    fn drop(&mut self) {
        let msg = "Unable to remove the ProcessGuard";
        let origin = format!("{:?}", self);

        // The drop order is important, the last entry must be the context file so that we can also recover if
        // the cleaner process dies in the middle of cleaning up the resources
        //
        // Therefore, we check explicitly if a file has the ownership and remove it explicitly instead of
        // handling this via RAII
        for file in [&mut self.state, &mut self.owner_lock, &mut self.context] {
            let Some(mut file) = file.take() else {
                continue;
            };

            if file.has_ownership() {
                let file_path = *file
                    .path()
                    .expect("created from path therefore it always contains a value");
                if let Err(e) = file.set_permission(Permission::OWNER_ALL) {
                    warn!(from origin,
                        "{msg} since the access rights of the file \"{:?}\" could not be elevated to grant permissions to remove the file. [{e:?}]",
                        file_path);
                }
                if let Err(e) = file.remove_self() {
                    warn!(from origin, "{msg} since the file \"{:?}\" could not be removed. [{e:?}]", file_path)
                }
            }
        }
    }
}

/// A guard for a process that makes the process monitorable by a [`ProcessMonitor`] as long as it
/// is in scope. When it goes out of scope the process is no longer monitorable.
///
/// ```
/// # extern crate iceoryx2_bb_loggers;
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"process_state_file").unwrap();
///
/// // start monitoring from this point on
/// let guard = ProcessGuardBuilder::new()
///     .create(&process_state_path).expect("");
///
/// // normal application code
///
/// // stop monitoring
/// drop(guard);
/// ```
#[derive(Debug)]
pub struct ProcessGuard {
    files: StateFiles,
}

impl Abandonable for ProcessGuard {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };
        let msg = "Unable to stage death";

        if let Err(e) = this
            .files
            .context
            .as_mut()
            .expect("contains always a value, only removed on destruction")
            .write_val_at(0, &UniqueProcessId::new_zeroed())
        {
            // The usage of `fatal_panic` is safe since the API shall be used only for testing.
            fatal_panic!(from this, "{msg} since the state file could not be overridden with zeros. [{e:?}]");
        }

        if let Some(f) = &this.files.state {
            if let Err(e) = ProcessGuardBuilder::set_lock(f, LockType::Unlock) {
                warn!(from this,
                    "{msg} since the lock of the state file could not be released. [{e:?}]");
            }
        }

        unsafe { StateFiles::abandon_in_place(NonNull::iox2_from_mut(&mut this.files)) };
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        let msg = "Unable to remove the ProcessGuard";

        if let Some(f) = &self.files.state {
            if let Err(e) = ProcessGuardBuilder::set_lock(f, LockType::Unlock) {
                warn!(from self,
                    "{msg} since the lock of the state file could not be released. [{e:?}]");
            }
        }
    }
}

fn generate_owner_lock_path(path: &FilePath) -> Result<FilePath, SemanticStringError> {
    let mut owner_lock_path = *path;
    fail!(from "generate_owner_lock_path()",
          when owner_lock_path.push_bytes(OWNER_LOCK_SUFFIX),
          "Unable to construct a valid file path from \"{path}\" and the suffix and the owner lock suffix.");
    Ok(owner_lock_path)
}

fn generate_context_path(path: &FilePath) -> Result<FilePath, SemanticStringError> {
    let mut context_path = *path;
    fail!(from "generate_context_path()",
          when context_path.push_bytes(CONTEXT_SUFFIX),
          "Unable to construct a valid file path from \"{path}\" and the suffix and the context suffix.");
    Ok(context_path)
}

impl ProcessGuard {
    /// Removes by force an existing [`ProcessGuard`]. This is useful when stale resources of
    /// a dead process need to be cleaned up.
    ///
    /// # Safety
    ///
    ///  - Users must ensure that no [`Process`] currently has
    ///    an instance of [`ProcessGuard`], [`ProcessCleaner`] and [`ProcessMonitor`] that
    ///    will be removed.
    pub unsafe fn remove(file: &FilePath) -> Result<bool, FileRemoveError> {
        let msg = "Unable to remove process guard resources";
        let origin = "ProcessGuard::remove()";
        let owner_lock_path = generate_owner_lock_path(file)
            .map_err(|_| FileRemoveError::MaxSupportedPathLengthExceeded)?;

        let mut result = match File::remove(file) {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin, with e,
                "{msg} since the underlying file \"{file}\" could not be removed.");
            }
        };

        result &= match File::remove(&owner_lock_path) {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin, with e,
                "{msg} since the underlying owner lock file \"{owner_lock_path}\" could not be removed.");
            }
        };

        Ok(result)
    }

    /// Returns the [`FilePath`] under which the underlying file is stored.
    pub fn path(&self) -> &FilePath {
        self.files
            .state
            .as_ref()
            .expect("contains always a value, only removed on destruction")
            .path()
            .expect("file is created from path and contains always a path")
    }
}

/// Monitor processes that have created a [`ProcessGuard`]. If the process dies, shutdowns or is
/// alive the monitor will detect it.
///
/// # Example
///
/// ```
/// # extern crate iceoryx2_bb_loggers;
///
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"process_state_file").unwrap();
///
/// let mut monitor = ProcessMonitor::new(&process_state_path).unwrap();
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
#[derive(Debug)]
pub struct ProcessMonitor {
    state_path: FilePath,
    owner_lock_path: FilePath,
    context_path: FilePath,
}

impl ProcessMonitor {
    /// Creates a new [`ProcessMonitor`] that can obtain the state of the process that will be
    /// monitored.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
    /// use iceoryx2_bb_posix::process_state::*;
    ///
    /// let process_state_path = FilePath::new(b"process_state_file").unwrap();
    ///
    /// let mut monitor = ProcessMonitor::new(&process_state_path).expect("");
    /// ```
    pub fn new(path: &FilePath) -> Result<Self, ProcessMonitorCreateError> {
        let owner_lock_path = generate_owner_lock_path(path)
            .map_err(|_| ProcessMonitorCreateError::InvalidCleanerPathName)?;
        let context_path = generate_context_path(path)
            .map_err(|_| ProcessMonitorCreateError::InvalidCleanerPathName)?;

        let new_self = Self {
            state_path: *path,
            owner_lock_path,
            context_path,
        };

        Ok(new_self)
    }

    /// Returns the path of the underlying file of the [`ProcessGuard`].
    pub fn path(&self) -> &FilePath {
        &self.state_path
    }

    /// Returns the current state of the process that is monitored.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate iceoryx2_bb_loggers;
    ///
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

        let my_process_id = match Process::unique_id() {
            Ok(v) => v,
            Err(e) => {
                fail!(from self, with ProcessMonitorStateError::FailedToAcquireUniqueProcessId,
                    "{msg} since the unique process id could not be acquired. [{e:?}]");
            }
        };

        // first we need to open the context_file with AccessMode::Write only since it could
        // be in init mode. After the initialization we can open it with AccessMode::Read
        match Self::open_file(&self.context_path, AccessMode::Write) {
            Ok(Some(context_file)) => {
                if context_file.permission().unwrap() == INIT_PERMISSION {
                    return Ok(ProcessState::Starting);
                }
            }
            Ok(None) => {
                return Ok(ProcessState::DoesNotExist);
            }
            Err(ProcessMonitorOpenError::InsufficientPermissions) => {
                // if the process state is initialized the permissions are read only and the file cannot be opened
                // with `AccessMode::Write`
            }
            Err(e) => return Err(e.into()),
        }

        let other_process_id = if let Some(context_file) =
            Self::open_file(&self.context_path, AccessMode::Read)?
        {
            let other_process_id: UniqueProcessId = match context_file.read_val() {
                Ok(v) => v,
                Err(e) => {
                    fail!(from self, with ProcessMonitorStateError::FailedToAcquireUniqueProcessIdFromContextFile,
                              "{msg} since the unique process id contained in the owner lock file could not be read. [{e:?}]");
                }
            };

            if my_process_id == other_process_id {
                return Ok(ProcessState::Alive);
            }

            other_process_id
        } else {
            return Ok(ProcessState::DoesNotExist);
        };

        if let Some(owner_lock_file) = Self::open_file(&self.owner_lock_path, AccessMode::Write)? {
            let lock_state = fail!(from self,
                                    when Self::get_lock_state(&owner_lock_file),
                                    "{} since the lock state of the owner_lock file could not be acquired.", msg);
            if lock_state == posix::F_WRLCK as _ {
                return Ok(ProcessState::CleaningUp);
            }
        } else {
            if File::does_exist(&self.state_path).unwrap() {
                return Err(ProcessMonitorStateError::CorruptedState);
            }
            return Ok(ProcessState::CleaningUp);
        }

        // IMPORTANT: it is essential that the state file is only then opened when it is ensured that the
        // ProcessGuard is not hold by the same process. Otherwise, the lock is released as soon as the
        // state file is closed. This is a weird case in some OSes that release a file lock as soon as
        // any file descriptor is closed to that file, even when other file descriptors to that same file
        // are still open.
        match Self::open_file(&self.state_path, AccessMode::Write)? {
            Some(state_file) => {
                let lock_state = fail!(from self, when Self::get_lock_state(&state_file),
                                    "{} since the lock state of the state file could not be acquired.", msg);
                match lock_state as _ {
                    posix::F_WRLCK => {
                        // It is possible that the file system is not yet completely synced and the
                        // file lock cannot be acquired despite the process is already dead.
                        // Therefore, we check again manually.
                        if Process::from_pid(other_process_id.pid()).is_alive() {
                            Ok(ProcessState::Alive)
                        } else {
                            Ok(ProcessState::Dead)
                        }
                    }
                    _ => Ok(ProcessState::Dead),
                }
            }
            None => Ok(ProcessState::CleaningUp),
        }
    }

    fn get_lock_state(file: &File) -> Result<i64, ProcessMonitorStateError> {
        let msg = format!("Unable to acquire lock on file {file:?}");
        let mut current_state = posix::flock::new_zeroed();
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

    fn open_file(
        path: &FilePath,
        access_mode: AccessMode,
    ) -> Result<Option<File>, ProcessMonitorOpenError> {
        let origin = "ProcessMonitor::new()";
        let msg = format!("Unable to open ProcessMonitor state file \"{path}\"");

        match FileBuilder::new(path).open_existing(access_mode) {
            Ok(f) => Ok(Some(f)),
            Err(FileOpenError::FileDoesNotExist) => Ok(None),
            Err(FileOpenError::IsDirectory) => {
                fail!(from origin, with ProcessMonitorOpenError::IsDirectory,
                    "{} since the path is a directory.", msg);
            }
            Err(FileOpenError::InsufficientPermissions) => {
                fail!(from origin, with ProcessMonitorOpenError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
            }
            Err(FileOpenError::Interrupt) => {
                fail!(from origin, with ProcessMonitorOpenError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(v) => {
                fail!(from origin, with ProcessMonitorOpenError::UnknownError,
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
/// # extern crate iceoryx2_bb_loggers;
///
/// use iceoryx2_bb_posix::process_state::*;
///
/// let process_state_path = FilePath::new(b"process_state_file").unwrap();
///
/// match ProcessCleaner::new(&process_state_path) {
///     Ok(guard) => {/* cleanup all process resources */},
///     Err(_) => (),
/// }
/// ```
#[derive(Debug)]
pub struct ProcessCleaner {
    files: StateFiles,
}

impl Abandonable for ProcessCleaner {
    unsafe fn abandon_in_place(mut this: NonNull<Self>) {
        let this = unsafe { this.as_mut() };

        if let Some(f) = &this.files.owner_lock {
            if let Err(e) = ProcessGuardBuilder::set_lock(f, LockType::Unlock) {
                warn!(from this,
                    "Unable to abandon ProcessCleaner since the lock of the owner file could not be released. [{e:?}]");
            }
        }

        unsafe { StateFiles::abandon_in_place(NonNull::iox2_from_mut(&mut this.files)) };
    }
}

impl ProcessCleaner {
    /// Creates a new [`ProcessCleaner`]. Succeeds when the process that creates the state files
    /// with the [`ProcessGuard`] died an no other process has acquired the resources for cleanup
    /// with [`ProcessCleaner::new()`].
    pub fn new(path: &FilePath) -> Result<Self, ProcessCleanerCreateError> {
        let msg = format!("Unable to instantiate ProcessCleaner \"{path}\"");
        let origin = "ProcessCleaner::new()";

        let owner_lock_path = generate_owner_lock_path(path)
            .map_err(|_| ProcessCleanerCreateError::InvalidCleanerPathName)?;
        let context_path = generate_context_path(path)
            .map_err(|_| ProcessCleanerCreateError::InvalidCleanerPathName)?;

        match (ProcessMonitor {
            state_path: *path,
            context_path,
            owner_lock_path,
        }
        .state())
        {
            Ok(ProcessState::Dead) => (),
            Ok(ProcessState::Alive) => {
                fail!(from origin, with ProcessCleanerCreateError::ProcessIsStillAlive,
                    "{} since the corresponding process is still alive.", msg);
            }
            Ok(ProcessState::DoesNotExist) => {
                fail!(from origin, with ProcessCleanerCreateError::DoesNotExist,
                    "{} since the process state file does not exist.", msg);
            }
            Ok(ProcessState::CleaningUp) => {
                fail!(from origin, with ProcessCleanerCreateError::ProcessIsBeingCleanedUpOrCrashedDuringCleanup,
                    "{} since another process already has instantiated a ProcessCleaner.", msg);
            }
            Ok(ProcessState::Starting) => {
                fail!(from origin, with ProcessCleanerCreateError::ProcessIsInitializedOrCrashedDuringInitialization,
                    "{} since it seems that the ProcessGuard is still being created or it has crashed during initialization.", msg);
            }
            Err(e) => {
                fail!(from origin, with e.into(),
                    "{} since the process state could not be acquired. [{e:?}]", msg);
            }
        };

        let context_file = match ProcessMonitor::open_file(&context_path, AccessMode::Read) {
            Ok(Some(file)) => file,
            Ok(None) => {
                fail!(from origin, with ProcessCleanerCreateError::DoesNotExist,
                    "{} since the process context file does not exist.", msg);
            }
            Err(e) => {
                fail!(from origin, with ProcessCleanerCreateError::UnableToOpenContextFile,
                    "{} since the context file \"{context_path}\" could not be opened. [{e:?}]", msg);
            }
        };

        let other_process_id: UniqueProcessId = match context_file.read_val() {
            Ok(v) => v,
            Err(e) => {
                fail!(from origin, with ProcessCleanerCreateError::FailedToAcquireUniqueProcessIdFromContextFile,
                          "{msg} since the unique process id contained in the owner lock file could not be read. [{e:?}]");
            }
        };

        let owner_lock_file = match ProcessMonitor::open_file(&owner_lock_path, AccessMode::Write) {
            Ok(Some(file)) => file,
            Ok(None) => {
                fail!(from origin, with ProcessCleanerCreateError::ProcessIsBeingCleanedUpOrCrashedDuringCleanup,
                    "{} since the process state is either being cleaned up or crashed during cleanup.", msg);
            }
            Err(e) => {
                fail!(from origin, with ProcessCleanerCreateError::UnableToOpenOwnerLockFile,
                    "{} since the owner lock file \"{owner_lock_path}\" could not be opened. [{e:?}]", msg);
            }
        };

        // IMPORTANT: it is essential that the state file is only then opened when it is ensured that the
        // ProcessGuard is not hold by the same process. Otherwise, the lock is released as soon as the
        // state file is closed. This is a weird case in some OSes that release a file lock as soon as
        // any file descriptor is closed to that file, even when other file descriptors to that same file
        // are still open.
        let state_file = match ProcessMonitor::open_file(path, AccessMode::Write) {
            Ok(Some(file)) => file,
            Ok(None) => {
                fail!(from origin, with ProcessCleanerCreateError::ProcessIsBeingCleanedUpOrCrashedDuringCleanup,
                    "{} since the process state is either being cleaned up or crashed during cleanup.", msg);
            }
            Err(e) => {
                fail!(from origin, with ProcessCleanerCreateError::UnableToOpenStateFile,
                    "{} since the state file \"{path}\" could not be opened. [{e:?}]", msg);
            }
        };

        let lock_state = fail!(from origin, when ProcessMonitor::get_lock_state(&state_file),
            with ProcessCleanerCreateError::FailedToAcquireLockState,
            "{} since the lock state could not be acquired.", msg);

        if lock_state == posix::F_WRLCK as _ && Process::from_pid(other_process_id.pid()).is_alive()
        {
            fail!(from origin, with ProcessCleanerCreateError::ProcessIsStillAlive,
                "{} since the corresponding process is still alive.", msg);
        }

        match ProcessGuardBuilder::set_lock(&owner_lock_file, LockType::Write) {
            Ok(()) => {
                context_file.acquire_ownership();
                state_file.acquire_ownership();
                owner_lock_file.acquire_ownership();
                Ok(Self {
                    files: StateFiles {
                        state: Some(state_file),
                        owner_lock: Some(owner_lock_file),
                        context: Some(context_file),
                    },
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
            Err(ProcessGuardLockError::FileRemovedFromFileSystem) => {
                fail!(from origin, with ProcessCleanerCreateError::DoesNotExist,
                    "{} since the process state file was cleaned up right before acquiring the lock.", msg);
            }
            Err(e) => {
                fail!(from origin, with ProcessCleanerCreateError::UnknownError,
                    "{} due to an unknown failure ({:?}).", msg, e);
            }
        }
    }

    /// Abandons the [`ProcessCleaner`] without removing the underlying resources. This is useful
    /// when another process tried to cleanup the stale resources of the dead process but is unable
    /// to due to insufficient permissions.
    pub fn abandon(self) {
        self.files.release_ownership();
    }
}
