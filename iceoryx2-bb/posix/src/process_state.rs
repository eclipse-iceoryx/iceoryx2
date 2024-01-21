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

use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_log::{debug, error, fail, fatal_panic, trace, warn};
use iceoryx2_bb_system_types::file_path::FilePath;
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProcessState {
    Alive,
    Dead,
    DoesNotExist,
    InInitialization,
}

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ProcessWatcherCreateError {
    InsufficientPermissions,
    Interrupt,
    IsDirectory,
    UnknownError,
}

enum_gen! {ProcessWatcherStateError
  entry:
    CorruptedState,
    Interrupt,
    UnknownError(i32)

  mapping:
    ProcessWatcherCreateError
}

#[derive(Debug)]
pub struct ProcessGuard {
    file: Option<File>,
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        let path = self.path().clone();
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
                return Ok(Self { file: Some(file) });
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

    pub fn remove(path: &FilePath) -> Result<bool, FileRemoveError> {
        Ok(
            fail!(from "ProcessGuard::remove()", when File::remove(path),
            "Unable to remove ProcessStateGuard file."),
        )
    }

    pub fn path(&self) -> &FilePath {
        match self.file.as_ref() {
            Some(ref f) => match f.path() {
                Some(ref path) => path,
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

#[derive(Debug)]
pub struct ProcessWatcher {
    file: Option<File>,
    path: FilePath,
}

impl ProcessWatcher {
    pub fn new(path: &FilePath) -> Result<Self, ProcessWatcherCreateError> {
        let mut new_self = Self {
            file: None,
            path: *path,
        };

        new_self.open_file()?;
        Ok(new_self)
    }

    pub fn path(&self) -> &FilePath {
        &self.path
    }

    pub fn reset(&mut self) {
        self.file = None;
    }

    pub fn state(&mut self) -> Result<ProcessState, ProcessWatcherStateError> {
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
                    fail!(from self, with ProcessWatcherStateError::UnknownError(0),
                        "{} since an unknown failure occurred while checking if the file exists ({:?}).", msg, v);
                }
            },
        }
    }

    fn read_state_from_file(&mut self) -> Result<ProcessState, ProcessWatcherStateError> {
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
            handle_errno!(ProcessWatcherStateError, from self,
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
                    fail!(from self, with ProcessWatcherStateError::UnknownError(0),
                            "{} since an unknown failure occurred while checking if the process state file exists ({:?}).", msg, v);
                }
            },
        }
    }

    fn open_file(&mut self) -> Result<(), ProcessWatcherCreateError> {
        let origin = "ProcessWatcher::new()";
        let msg = format!(
            "Unable to open ProcessWatcher with the file \"{}\"",
            self.path
        );
        self.file = match FileBuilder::new(&self.path).open_existing(AccessMode::Read) {
            Ok(f) => Some(f),
            Err(FileOpenError::FileDoesNotExist) => None,
            Err(FileOpenError::IsDirectory) => {
                fail!(from origin, with ProcessWatcherCreateError::IsDirectory,
                    "{} since the path is a directory.", msg);
            }
            Err(FileOpenError::InsufficientPermissions) => {
                if FileBuilder::new(&self.path)
                    .open_existing(AccessMode::Write)
                    .is_ok()
                {
                    None
                } else {
                    fail!(from origin, with ProcessWatcherCreateError::InsufficientPermissions,
                    "{} due to insufficient permissions.", msg);
                }
            }
            Err(FileOpenError::Interrupt) => {
                fail!(from origin, with ProcessWatcherCreateError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(v) => {
                fail!(from origin, with ProcessWatcherCreateError::UnknownError,
                    "{} since an unknown failure occurred ({:?}).", msg, v);
            }
        };

        Ok(())
    }
}
