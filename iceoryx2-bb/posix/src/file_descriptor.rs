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

//! Provides a [`FileDescriptor`] abstraction which takes the ownership of low-level POSIX
//! file descriptors and the [`FileDescriptorBased`] & [`FileDescriptorManagement`] traits
//! which provide advanced functionalities to all [`FileDescriptorBased`] constructs.
//!
//! # Examples
//! ## Use [`FileDescriptorManagement`] to extend a type
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//! use iceoryx2_bb_posix::file_descriptor::*;
//!
//! // required for FileDescriptorManagement
//! #[derive(Debug)]
//! pub struct SomeConstructBasedOnFileDescriptor {
//!   fd: FileDescriptor
//! }
//!
//! // implement FileDescriptorBased trait
//! impl FileDescriptorBased for SomeConstructBasedOnFileDescriptor {
//!     fn file_descriptor(&self) -> &FileDescriptor {
//!         &self.fd
//!     }
//! }
//!
//!
//! // auto implement the FileDescriptorManagement trait to gain more file descriptor management
//! // features
//! impl FileDescriptorManagement for SomeConstructBasedOnFileDescriptor {}
//! ```
//!
//! ## Work with [`FileDescriptorManagement`]
//!
//! ```no_run
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//! use iceoryx2_bb_posix::file_descriptor::*;
//! use iceoryx2_bb_posix::file::*;
//! use iceoryx2_bb_posix::ownership::*;
//! use iceoryx2_bb_posix::user::UserExt;
//! use iceoryx2_bb_posix::group::GroupExt;
//!
//! let file_name = FilePath::new(b"/tmp/someFile").unwrap();
//! let mut file = FileBuilder::new(&file_name).creation_mode(CreationMode::PurgeAndCreate)
//!                              .create().expect("failed to create file");
//!
//! println!("owner: {:?}", file.ownership().unwrap());
//! println!("permission: {}", file.permission().unwrap());
//! println!("metadata: {:?}", file.metadata().unwrap());
//!
//! // set new owner
//! file.set_ownership(OwnershipBuilder::new()
//!         .uid("testuser1".as_user().unwrap().uid())
//!         .gid("testgroup1".as_group().unwrap().gid()).create());
//!
//! // set new permissions
//! file.set_permission(Permission::ALL);
//! ```

use core::fmt::Debug;

use crate::config::EINTR_REPETITIONS;
use crate::file::*;
use crate::group::Gid;
use crate::metadata::Metadata;
use crate::ownership::*;
use crate::permission::{Permission, PermissionExt};
use crate::process::{Process, ProcessId};
use crate::user::Uid;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_log::{error, fail, fatal_panic, trace, warn};
use iceoryx2_pal_posix::posix::MemZeroedStruct;
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::*;

/// Represents a FileDescriptor in a POSIX system. Contains always a value greater or equal zero,
/// a valid file descriptor. It takes the ownership of the provided file descriptor and calls
/// [`posix::close`] on destruction.
///
/// # Example
///
/// ```ignore
/// # extern crate iceoryx2_bb_loggers;
///
/// use iceoryx2_bb_posix::file_descriptor::*;
///
/// let valid_fd = FileDescriptor::new(2);
/// let invalid_fd = FileDescriptor::new(-4);
///
/// println!("Created FD: {:?}", valid_fd.unwrap());
/// ```
#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub struct FileDescriptor {
    value: i32,
    is_owned: bool,
}

impl Clone for FileDescriptor {
    fn clone_from(&mut self, source: &Self) {
        self.close();
        *self = source.clone();
    }

    fn clone(&self) -> Self {
        let fd_clone = unsafe { posix::dup(self.value) };
        if fd_clone < 0 {
            let msg = "Unable to clone file descriptor";
            match Errno::get() {
                Errno::EMFILE => {
                    fatal_panic!(from self, "{} since the maximum amount of open file descriptors for the process is reached.", msg)
                }
                v => fatal_panic!(from self, "{} since an unknown error occurred ({}).", msg, v),
            }
        }

        Self {
            value: fd_clone,
            is_owned: true,
        }
    }
}

impl FileDescriptor {
    /// Creates a FileDescriptor which does not hold the ownership of the file descriptor and will
    /// not call [`posix::close`] on destruction.
    pub fn non_owning_new(value: i32) -> Option<FileDescriptor> {
        if value < 0 {
            return None;
        }

        Self::new(value).map(|mut fd| {
            fd.is_owned = false;
            fd
        })
    }

    /// Creates a FileDescriptor which does not hold the ownership of the file descriptor and will
    /// not call [`posix::close`] on destruction.
    ///
    /// # Safety
    ///
    ///  * it must be a valid file descriptor for the lifetime of [`FileDescriptor`]
    ///
    pub unsafe fn non_owning_new_unchecked(value: i32) -> FileDescriptor {
        FileDescriptor {
            value,
            is_owned: false,
        }
    }

    /// Creates a new FileDescriptor. If the value is smaller than zero or it does not contain a
    /// valid file descriptor value it returns [`None`].
    pub fn new(value: i32) -> Option<FileDescriptor> {
        if value < 0 {
            return None;
        }

        if unsafe { posix::fcntl2(value, posix::F_GETFD) } < 0 {
            return None;
        }

        Some(FileDescriptor {
            value,
            is_owned: true,
        })
    }

    /// Creates a new FileDescriptor.
    ///
    /// # Safety
    ///
    ///  * it must be a valid file descriptor for the lifetime of [`FileDescriptor`]
    ///
    pub unsafe fn new_unchecked(value: i32) -> FileDescriptor {
        FileDescriptor {
            value,
            is_owned: true,
        }
    }

    /// Returns the underlying value of the FileDescriptor
    ///
    /// # Safety
    ///
    ///  * the user shall not store the value in a variable otherwise lifetime issues may be
    ///    encountered
    ///  * do not manually close the file descriptor with a sys call
    ///
    pub unsafe fn native_handle(&self) -> i32 {
        self.value
    }

    fn close(&mut self) {
        let mut counter = 0;
        loop {
            if unsafe { posix::close(self.value) } == 0 {
                break;
            }

            match Errno::get() {
                Errno::EBADF => {
                    fatal_panic!(from self, "This should never happen! Unable to close file due to an invalid file-descriptor.");
                }
                Errno::EINTR => {
                    counter += 1;
                    if counter > EINTR_REPETITIONS {
                        error!(from self, "Unable to close file since too many interrupt signals were received.");
                    }
                }
                Errno::EIO => {
                    error!(from self, "Unable to close file due to an I/O error.");
                    counter += 1;
                }
                v => {
                    fatal_panic!(from self, "This should never happen! Unable to close file since an unknown error occurred ({}).", v);
                }
            }

            if counter > EINTR_REPETITIONS {
                error!(from self, "Tried {} times to close the file but failed.", counter);
            }
        }
    }
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        if self.is_owned {
            self.close()
        }
    }
}

/// Every construct which is based on some [`FileDescriptor`] can implement this trait to gain
/// extended [`FileDescriptorManagement`] features.
pub trait FileDescriptorBased {
    /// Returns the file descriptor of the underlying construct
    fn file_descriptor(&self) -> &FileDescriptor;
}

impl FileDescriptorBased for FileDescriptor {
    fn file_descriptor(&self) -> &FileDescriptor {
        self
    }
}

enum_gen! { FileTryLockError
  entry:
    Interrupt,
    ExceedsMaximumNumberOfLockedRegionsInSystem,
    InvalidFileDescriptorOrWrongOpenMode,
    DeadlockConditionDetected,
    FileRemovedFromFileSystem,
    UnknownError(i32)
}

enum_gen! { FileGetLockStateError
  entry:
    Interrupt,
    UnknownError(i32)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i16)]
pub enum LockType {
    Read = posix::F_RDLCK as i16,
    Write = posix::F_WRLCK as i16,
}

#[derive(Debug)]
pub struct FileLockGuard<'a, T: Debug + FileDescriptorBased> {
    parent: &'a T,
}

impl<'a, T: Debug + FileDescriptorBased> FileLockGuard<'a, T> {
    /// Leaks the [`FileLockGuard`] without unlocking the corresponding file lock.
    /// This can be useful when the file lock is coupled to the lifetime of the file itself and one wants
    /// to avoid self-referencing structs.
    ///
    /// # Safety
    ///
    /// * Ensure that the underlying [`FileDescriptor`] is owned and closed, otherwise a deadlock might
    ///   occur.
    pub unsafe fn leak(self) {
        core::mem::forget(self);
    }
}

impl<'a, T: Debug + FileDescriptorBased> Drop for FileLockGuard<'a, T> {
    fn drop(&mut self) {
        let mut new_lock_state = posix::flock::new_zeroed();
        new_lock_state.l_type = posix::F_UNLCK as i16;
        new_lock_state.l_whence = posix::SEEK_SET as _;

        if unsafe {
            posix::fcntl(
                self.parent.file_descriptor().native_handle(),
                posix::F_SETLK,
                &mut new_lock_state,
            )
        } == -1
        {
            warn!(from self,
                "Failed to release file read lock. This can be caused by opening the same file multiple times and closing it before releasing the readlock. [{:?}]", Errno::get());
        }
    }
}

/// Contains what lock is owned by whom.
pub struct LockState {
    lock_type: LockType,
    process: Process,
}

impl LockState {
    /// Returns the [`LockType`]
    pub fn lock_type(&self) -> LockType {
        self.lock_type
    }

    /// Returns the owner [`Process`] of the lock.
    pub fn owning_process(&self) -> Process {
        self.process
    }
}

impl FileDescriptorManagement for FileDescriptor {}

/// Provides additional feature for every file descriptor based construct like
///  * ownership handling, [`ownership`](FileDescriptorManagement::ownership()),
///    [`set_ownership`](FileDescriptorManagement::set_ownership())
///  * permission handling, [`permission`](FileDescriptorManagement::permission()),
///    [`set_permission`](FileDescriptorManagement::set_permission())
///  * truncate size, [`truncate`](FileDescriptorManagement::truncate())
///  * accessing extended stats via [`Metadata`], [`metadata`](FileDescriptorManagement::metadata())
///
pub trait FileDescriptorManagement: FileDescriptorBased + Debug + Sized {
    /// Acquires a file lock based on the [`FileDescriptor`].
    ///
    /// # Safety
    ///
    /// Due to the nature of the different behaviors of file locks across POSIX platforms:
    ///
    /// * The write/read lock may be obtained multiple times for the same process.
    /// * Opening the same file and closing it before dropping the [`FileLockGuard`] may release any lock.
    ///
    unsafe fn try_lock<'a>(
        &'a self,
        lock_type: LockType,
    ) -> Result<Option<FileLockGuard<'a, Self>>, FileTryLockError> {
        let mut new_lock_state = posix::flock::new_zeroed();
        new_lock_state.l_type = lock_type as _;
        new_lock_state.l_whence = posix::SEEK_SET as _;

        if unsafe {
            posix::fcntl(
                self.file_descriptor().native_handle(),
                posix::F_SETLK,
                &mut new_lock_state,
            )
        } != -1
        {
            return Ok(Some(FileLockGuard { parent: self }));
        }

        let msg = match lock_type {
            LockType::Read => "Unable to acquire read file-lock",
            _ => "Unable to acquire write file-lock",
        };

        let errno = Errno::get();

        if let Ok(metadata) = self.metadata() {
            if metadata.number_of_links() == 0 {
                fail!(from self, with FileTryLockError::FileRemovedFromFileSystem,
                    "{msg} since the file was removed from the file system.");
            }
        }

        match errno {
            Errno::EACCES | Errno::EAGAIN => Ok(None),
            Errno::EBADF => {
                fail!(from self, with FileTryLockError::InvalidFileDescriptorOrWrongOpenMode,
                    "{msg} since the file-descriptor is invalid or not opened in the correct access mode.");
            }
            Errno::EINTR => {
                fail!(from self, with FileTryLockError::Interrupt,
                    "{msg} since an interrupt signal was raised.");
            }
            Errno::ENOLCK => {
                fail!(from self, with FileTryLockError::ExceedsMaximumNumberOfLockedRegionsInSystem,
                    "{msg} since it would exceed the maximum supported number of locked regions in the system.");
            }
            Errno::EDEADLK => {
                fail!(from self, with FileTryLockError::DeadlockConditionDetected,
                    "{msg} since a deadlock condition was detected.");
            }
            v => {
                fail!(from self, with FileTryLockError::UnknownError(v as i32),
                    "{msg} since an unknown error occurred ({v}).");
            }
        }
    }

    /// Acquires the file lock state. When the underlying [`FileDescriptor`] is not locked, it returns [`None`].
    fn get_lock_state(&self) -> Result<Option<LockState>, FileGetLockStateError> {
        let msg = "Unable to acquire lock state";
        let mut current_state = posix::flock::new_zeroed();
        current_state.l_type = LockType::Write as _;
        current_state.l_whence = posix::SEEK_SET as _;

        if unsafe {
            posix::fcntl(
                self.file_descriptor().native_handle(),
                posix::F_GETLK,
                &mut current_state,
            )
        } == -1
        {
            handle_errno!(FileGetLockStateError, from self,
                Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            )
        }

        let lock_type = match current_state.l_type as _ {
            posix::F_UNLCK => return Ok(None),
            posix::F_RDLCK => LockType::Read,
            posix::F_WRLCK => LockType::Write,
            n => {
                fail!(from self, with FileGetLockStateError::UnknownError(0),
                    "{msg} since the sys-call provided an invalid lock-state: {n}.");
            }
        };

        Ok(Some(LockState {
            lock_type,
            process: Process::from_pid(ProcessId::new(current_state.l_pid)),
        }))
    }

    /// Returns the current user and group owner of the file descriptor
    fn ownership(&self) -> Result<Ownership, FileStatError> {
        let attr =
            fail!(from self, when File::acquire_attributes(self), "Unable to read file owner.");
        Ok(OwnershipBuilder::new()
            .uid(Uid::new_from_native(attr.st_uid))
            .gid(Gid::new_from_native(attr.st_gid))
            .create())
    }

    /// Sets a new user and group owner
    fn set_ownership(&mut self, ownership: Ownership) -> Result<(), FileSetOwnerError> {
        fail!(from self, when File::set_ownership(self, ownership.uid(), ownership.gid()),
            "Unable to set owner of the file.");
        trace!(from self, "set ownership to: {ownership:?}");
        Ok(())
    }

    /// Returns the current permission of the file descriptor
    fn permission(&self) -> Result<Permission, FileStatError> {
        Ok(
            fail!(from self, when File::acquire_attributes(self), "Unable to read permissions.")
                .st_mode
                .as_permission(),
        )
    }

    /// Sets new permissions
    fn set_permission(&mut self, permission: Permission) -> Result<(), FileSetPermissionError> {
        fail!(from self, when File::set_permission(self, permission),
                    "Unable to update permission.");
        trace!(from self, "set permission to: {permission}");
        Ok(())
    }

    /// Truncates to the file descriptor corresponding construct
    fn truncate(&mut self, size: usize) -> Result<(), FileTruncateError> {
        fail!(from self, when File::truncate(self, size),
                    "Unable to truncate to {}.", size);
        trace!(from self, "truncate to: {size}");
        Ok(())
    }

    /// Requires all available [`Metadata`] for the file descriptor
    fn metadata(&self) -> Result<Metadata, FileStatError> {
        Ok(Metadata::create(
            &fail!(from self, when File::acquire_attributes(self),
                    "Unable to acquire attributes to create Metadata."),
        ))
    }
}
