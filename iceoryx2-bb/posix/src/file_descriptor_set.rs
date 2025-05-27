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

//! Abstracts a POSIX file descriptor set based of FD_* and select.
//! Can be used to wait on multiple objects which implement the [`SynchronousMultiplexing`]
//! trait.
//!
//! # Example
//!
//! ```ignore
//! use iceoryx2_bb_posix::file_descriptor_set::*;
//! use iceoryx2_bb_posix::unix_datagram_socket::*;
//! use core::time::Duration;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let socket_name = FilePath::new(b"some_socket").unwrap();
//!
//! let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
//!     .creation_mode(CreationMode::PurgeAndCreate)
//!     .create()
//!     .unwrap();
//!
//! let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
//!     .create()
//!     .unwrap();
//!
//! let fd_set = FileDescriptorSet::new();
//! fd_set.add(&sut_receiver);
//! let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];
//! sut_sender.try_send(send_data.as_slice()).unwrap();
//!
//! // in some other process
//! let result = fd_set.timed_wait(Duration::from_secs(1), FileEvent::Read,
//!     |fd| println!("Fd was triggered {}", unsafe { fd.native_handle() })).unwrap();
//! ```

use core::{cell::UnsafeCell, fmt::Debug, time::Duration};

use crate::{
    clock::AsTimeval,
    file_descriptor::{FileDescriptor, FileDescriptorBased},
};
use iceoryx2_bb_log::fail;
use iceoryx2_pal_posix::posix::{errno::Errno, MemZeroedStruct};
use iceoryx2_pal_posix::*;

/// A trait which is implement by all objects which can be added to the [`FileDescriptorSet`].
pub trait SynchronousMultiplexing: FileDescriptorBased {}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileDescriptorSetWaitError {
    Interrupt,
    TooManyAttachedFileDescriptors,
    InsufficientPermissions,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileDescriptorSetAddError {
    AlreadyAttached,
    CapacityExceeded,
}

/// Defines the event type one wants to wait on in
/// [`FileDescriptorSet::timed_wait()`]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum FileEvent {
    Read,
    Write,
    Exceptional,
    ReadWrite,
    ReadExceptional,
    WriteExceptional,
    ReadWriteExceptional,
}

pub struct FileDescriptorSetGuard<'set, 'fd> {
    set: &'set FileDescriptorSet,
    fd: &'fd FileDescriptor,
}

impl<'fd> FileDescriptorSetGuard<'_, 'fd> {
    pub fn file_descriptor(&self) -> &'fd FileDescriptor {
        self.fd
    }
}

impl Drop for FileDescriptorSetGuard<'_, '_> {
    fn drop(&mut self) {
        self.set.remove(unsafe { self.fd.native_handle() })
    }
}

/// The POSIX abstraction file descriptor set to wait on multiple objects which implement
/// the [`SynchronousMultiplexing`] trait.
pub struct FileDescriptorSet {
    internals: UnsafeCell<Internals>,
}

struct Internals {
    fd_set: posix::fd_set,
    file_descriptors: Vec<i32>,
    max_fd: i32,
}

impl Debug for FileDescriptorSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "FileDescriptorSet {{ file_descriptors: {:?}, max_fd: {} }}",
            self.internals().file_descriptors,
            self.internals().max_fd
        )
    }
}

impl Default for FileDescriptorSet {
    fn default() -> Self {
        let fd_set = FileDescriptorSet {
            internals: UnsafeCell::new(Internals {
                fd_set: posix::fd_set::new_zeroed(),
                file_descriptors: vec![],
                max_fd: 0,
            }),
        };

        unsafe { posix::FD_ZERO(&mut fd_set.internals_mut().fd_set) };

        fd_set
    }
}

impl FileDescriptorSet {
    fn internals(&self) -> &Internals {
        unsafe { &*self.internals.get() }
    }

    #[allow(clippy::mut_from_ref)]
    fn internals_mut(&self) -> &mut Internals {
        unsafe { &mut *self.internals.get() }
    }

    pub fn new() -> FileDescriptorSet {
        FileDescriptorSet::default()
    }

    /// Adds a file descriptor
    pub fn add<'set, 'fd, F: SynchronousMultiplexing>(
        &'set self,
        fd: &'fd F,
    ) -> Result<FileDescriptorSetGuard<'set, 'fd>, FileDescriptorSetAddError> {
        self.add_impl(fd.file_descriptor())
    }

    fn add_impl<'set, 'fd>(
        &'set self,
        fd: &'fd FileDescriptor,
    ) -> Result<FileDescriptorSetGuard<'set, 'fd>, FileDescriptorSetAddError> {
        let msg = "Unable to add file descriptor";
        if self.internals().file_descriptors.len() >= Self::capacity() {
            fail!(from self, with FileDescriptorSetAddError::CapacityExceeded,
                "{msg} {:?} since the amount of file descriptors {} exceeds the maximum supported amount of file descriptors for a set {}.",
                fd.file_descriptor(), self.internals().file_descriptors.len(), Self::capacity());
        }

        if self.contains_impl(fd) {
            fail!(from self, with FileDescriptorSetAddError::AlreadyAttached,
                "{msg} {:?} since it is already attached.", fd);
        }

        unsafe {
            posix::FD_SET(
                fd.file_descriptor().native_handle(),
                &mut self.internals_mut().fd_set,
            )
        };
        self.internals_mut().max_fd = core::cmp::max(
            self.internals().max_fd,
            unsafe { fd.file_descriptor().native_handle() } + 1,
        );
        self.internals_mut()
            .file_descriptors
            .push(unsafe { fd.file_descriptor().native_handle() });

        Ok(FileDescriptorSetGuard { set: self, fd })
    }

    fn remove(&self, value: i32) {
        unsafe { posix::FD_CLR(value, &mut self.internals_mut().fd_set) };

        if self.internals_mut().max_fd == value + 1 {
            self.internals_mut().max_fd = 0;
            for fd in &self.internals().file_descriptors {
                self.internals_mut().max_fd = core::cmp::max(self.internals().max_fd, fd + 1);
            }
        }

        self.internals_mut()
            .file_descriptors
            .retain(|&v| value != v);
    }

    /// Returns the maximum capacity of the [`FileDescriptorSet`]
    pub const fn capacity() -> usize {
        posix::FD_SETSIZE
    }

    /// Returns the number of attached [`FileDescriptor`]s
    pub fn len(&self) -> usize {
        self.internals().file_descriptors.len()
    }

    /// Returns true if the [`FileDescriptorSet`] is empty, otherwise false
    pub fn is_empty(&self) -> bool {
        self.internals().file_descriptors.is_empty()
    }

    /// Returns true if the object is attached to the [`FileDescriptorSet`], otherwise false.
    pub fn contains<T: SynchronousMultiplexing>(&self, fd: &T) -> bool {
        self.contains_impl(fd.file_descriptor())
    }

    fn contains_impl(&self, fd: &FileDescriptor) -> bool {
        unsafe { posix::FD_ISSET(fd.native_handle(), &self.internals().fd_set) }
    }

    /// Blocks until the specified event has occurred. It
    /// returns a list with all [`FileDescriptor`]s which were triggered.
    pub fn blocking_wait<F: FnMut(&FileDescriptor)>(
        &self,
        event: FileEvent,
        fd_callback: F,
    ) -> Result<usize, FileDescriptorSetWaitError> {
        self.wait(core::ptr::null_mut(), event, fd_callback)
    }

    /// Waits until either the timeout has passed or the specified event has occurred. It
    /// returns a list with all [`FileDescriptor`]s which were triggered.
    pub fn timed_wait<F: FnMut(&FileDescriptor)>(
        &self,
        timeout: Duration,
        event: FileEvent,
        fd_callback: F,
    ) -> Result<usize, FileDescriptorSetWaitError> {
        let mut raw_timeout = timeout.as_timeval();
        self.wait(&mut raw_timeout, event, fd_callback)
    }

    fn wait<F: FnMut(&FileDescriptor)>(
        &self,
        timeout: *mut posix::timeval,
        event: FileEvent,
        mut fd_callback: F,
    ) -> Result<usize, FileDescriptorSetWaitError> {
        let mut fd_set: posix::fd_set = self.internals().fd_set;

        let read_fd: *mut posix::fd_set = match event {
            FileEvent::Read
            | FileEvent::ReadWrite
            | FileEvent::ReadExceptional
            | FileEvent::ReadWriteExceptional => &mut fd_set,
            _ => core::ptr::null_mut::<posix::fd_set>(),
        };
        let write_fd: *mut posix::fd_set = match event {
            FileEvent::Write
            | FileEvent::ReadWrite
            | FileEvent::WriteExceptional
            | FileEvent::ReadWriteExceptional => &mut fd_set,
            _ => core::ptr::null_mut::<posix::fd_set>(),
        };
        let exceptional_fd: *mut posix::fd_set = match event {
            FileEvent::Exceptional
            | FileEvent::ReadExceptional
            | FileEvent::WriteExceptional
            | FileEvent::ReadWriteExceptional => &mut fd_set,
            _ => core::ptr::null_mut::<posix::fd_set>(),
        };

        let msg = "Failure while waiting for file descriptor events";
        let number_of_notifications = unsafe {
            posix::select(
                self.internals().max_fd,
                read_fd,
                write_fd,
                exceptional_fd,
                timeout,
            )
        };

        if number_of_notifications == -1 {
            handle_errno!(FileDescriptorSetWaitError, from self,
                fatal Errno::EBADF => ("This should never happen! {} since at least one of the attached file descriptors is invalid.", msg),
                Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                Errno::EINVAL => (TooManyAttachedFileDescriptors,
                    "{} since the number of attached file descriptors exceed the system limit of ({}).",
                    msg, Self::capacity()),
                Errno::EPERM => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        for raw_fd in &self.internals().file_descriptors {
            if unsafe { posix::FD_ISSET(*raw_fd, &fd_set) } {
                let fd = FileDescriptor::non_owning_new(*raw_fd).unwrap();
                fd_callback(&fd);
            }
        }

        Ok(number_of_notifications as _)
    }
}
