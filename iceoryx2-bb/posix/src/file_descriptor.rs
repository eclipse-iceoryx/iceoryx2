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
use crate::user::Uid;
use iceoryx2_bb_log::{error, fail, fatal_panic};
use iceoryx2_pal_posix::posix::errno::Errno;
use iceoryx2_pal_posix::*;

/// Represents a FileDescriptor in a POSIX system. Contains always a value greater or equal zero,
/// a valid file descriptor. It takes the ownership of the provided file descriptor and calls
/// [`posix::close`] on destruction.
///
/// # Example
///
/// ```ignore
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
        // TODO: [#223] START: rewrite lines to: let *self = source.clone()
        let temp = source.clone();
        *self = temp;
        // TODO: [#223] END
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
    ///  * it must be a valid file descriptor
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
        Ok(())
    }

    /// Truncates to the file descriptor corresponding construct
    fn truncate(&mut self, size: usize) -> Result<(), FileTruncateError> {
        fail!(from self, when File::truncate(self, size),
                    "Unable to truncate to {}.", size);
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
