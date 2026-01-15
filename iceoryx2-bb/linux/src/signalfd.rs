// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

//! The [`SignalFd`] is a safe abstraction over the linux signal fd api. It
//! allows users to receive a [`FetchableSignal`] via a [`FileDescriptor`] that can be attached
//! to a
//! [`FileDescriptorSet`](iceoryx2_bb_posix::file_descriptor_set::FileDescriptorSet)
//! or via [`Epoll`](crate::epoll::Epoll).
//!
//! # Example
//!
//! ```
//! # extern crate iceoryx2_bb_loggers;
//!
//! use iceoryx2_bb_linux::signalfd::SignalFdBuilder;
//! use iceoryx2_bb_posix::signal_set::FetchableSignalSet;
//! use iceoryx2_bb_posix::signal::FetchableSignal;
//!
//! # fn main() -> Result<(), Box<dyn core::error::Error>> {
//!
//! let mut registered_signals = FetchableSignalSet::new_empty();
//! registered_signals.add(FetchableSignal::UserDefined1);
//!
//! let signal_fd = SignalFdBuilder::new(registered_signals).create_non_blocking()?;
//!
//! match signal_fd.try_read()? {
//!     Some(signal) => println!("signal was raised: {signal:?}"),
//!     None => println!("no signal was raised")
//! }
//!
//! # Ok(())
//! # }
//! ```

use core::fmt::Debug;

use iceoryx2_bb_posix::{
    file_descriptor::{FileDescriptor, FileDescriptorBased},
    file_descriptor_set::SynchronousMultiplexing,
    process::ProcessId,
    signal::FetchableSignal,
    signal_set::FetchableSignalSet,
    user::Uid,
};
use iceoryx2_log::{fail, fatal_panic};
use iceoryx2_pal_os_api::linux;
use iceoryx2_pal_posix::posix::{self};

/// Error emitted when creating a new [`SignalFd`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SignalFdCreationError {
    /// The process wide file handle limit is reached
    PerProcessFileHandleLimitReached,
    /// The system wide file handle limit is reached
    SystemWideFileHandleLimitReached,
    /// Insufficient memory available
    InsufficientMemory,
    /// The underlying inode device could not be mounted
    UnableToMountInodeDevice,
    /// An error that was not documented in the POSIX API was reported
    UnknownError(i32),
}

impl core::fmt::Display for SignalFdCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SignalFdCreationError::{self:?}")
    }
}

impl core::error::Error for SignalFdCreationError {}

/// Error emitted from [`BlockingSignalFd::blocking_read()`] or [`SignalFd::try_read()`].
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SignalFdReadError {
    /// The amount of bytes read were less than the size of the internal siginfo struct
    SystemBreaksReadContract,
    /// An interrupt signal was raised
    Interrupt,
    /// An input/output error occurred
    IOerror,
    /// Insufficient resources available
    InsufficientResources,
    /// Insufficient memory available
    InsufficientMemory,
    /// An error that was not documented in the POSIX API was reported
    UnknownError(i32),
}

impl core::fmt::Display for SignalFdReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SignalFdReadError::{self:?}")
    }
}

impl core::error::Error for SignalFdReadError {}

/// The builder that creates a [`SignalFd`] or a [`BlockingSignalFd`].
#[derive(Debug)]
pub struct SignalFdBuilder {
    signal_set: FetchableSignalSet,
    close_on_exec: bool,
}

impl SignalFdBuilder {
    /// Creates a new builder and the [`FetchableSignalSet`] defines which [`FetchableSignal`]s
    /// the [`SignalFd`] shall receive.
    pub fn new(signal_set: FetchableSignalSet) -> Self {
        Self {
            signal_set,
            close_on_exec: false,
        }
    }

    /// Defines if the underlying [`FileDescriptor`] shall be closed when the
    /// [`Process`](iceoryx2_bb_posix::process::Process) is forked.
    pub fn set_close_on_exec(mut self, value: bool) -> Self {
        self.close_on_exec = value;
        self
    }

    /// Create the non-blocking version of the [`SignalFd`].
    pub fn create_non_blocking(self) -> Result<SignalFd, SignalFdCreationError> {
        Ok(SignalFd {
            file_descriptor: self.create(true)?,
        })
    }

    /// Create the blocking version [`BlockingSignalFd`]
    pub fn create_blocking(self) -> Result<BlockingSignalFd, SignalFdCreationError> {
        Ok(BlockingSignalFd {
            file_descriptor: self.create(false)?,
        })
    }

    fn create(self, is_non_blocking: bool) -> Result<FileDescriptor, SignalFdCreationError> {
        let msg = "Unable to create SignalFd";
        let mut flags = 0;
        if self.close_on_exec {
            flags |= linux::SFD_CLOEXEC;
        }

        if is_non_blocking {
            flags |= linux::SFD_NONBLOCK;
        }

        let fd = unsafe { linux::signalfd(-1, self.signal_set.native_handle(), flags as _) };

        if fd == -1 {
            match posix::Errno::get() {
                posix::Errno::EMFILE => {
                    fail!(from self,
                        with SignalFdCreationError::PerProcessFileHandleLimitReached,
                        "{msg} since the per process file descriptor limit is exceeded.");
                }
                posix::Errno::ENFILE => {
                    fail!(from self,
                        with SignalFdCreationError::SystemWideFileHandleLimitReached,
                        "{msg} since the system wide file descriptor limit is exceeded.");
                }
                posix::Errno::ENODEV => {
                    fail!(from self,
                        with SignalFdCreationError::UnableToMountInodeDevice,
                        "{msg} since anonymous inode device could not be mapped.");
                }
                posix::Errno::ENOMEM => {
                    fail!(from self,
                        with SignalFdCreationError::InsufficientMemory,
                        "{msg} due to insufficient memory.");
                }
                e => {
                    fail!(from self,
                        with SignalFdCreationError::UnknownError(e as i32),
                        "{msg} due to an unknown error {e:?}.");
                }
            }
        }

        let file_descriptor = match FileDescriptor::new(fd) {
            Some(fd) => fd,
            None => fatal_panic!(from self,
                "This should never happen! {msg} since the signalfd returned a broken file descriptor (fd)."),
        };

        Ok(file_descriptor)
    }
}

/// Contains all details about the [`FetchableSignal`] that was raised.
pub struct SignalInfo {
    signal_info: linux::signalfd_siginfo,
}

impl Debug for SignalInfo {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "SignalInfo {{ signal: {:?}, origin_pid: {}, origin_uid: {} }}",
            self.signal(),
            self.origin_pid(),
            self.origin_uid()
        )
    }
}

impl SignalInfo {
    /// Returns the [`FetchableSignal`] that was received.
    pub fn signal(&self) -> FetchableSignal {
        (self.signal_info.ssi_signo as i32).into()
    }

    /// Returns the [`ProcessId`] of the origin that sent the signal.
    pub fn origin_pid(&self) -> ProcessId {
        ProcessId::new(self.signal_info.ssi_pid as _)
    }

    /// Returns the [`Uid`] of the origin that sent the signal.
    pub fn origin_uid(&self) -> Uid {
        Uid::new_from_native(self.signal_info.ssi_uid)
    }
}

/// Non-blocking version of a signalfd
#[derive(Debug)]
pub struct SignalFd {
    file_descriptor: FileDescriptor,
}

fn read_from_fd<T: Debug>(
    this: &T,
    fd: &FileDescriptor,
) -> Result<Option<SignalInfo>, SignalFdReadError> {
    let msg = "Unable to read signal from SignalFd";
    let mut signal_info: linux::signalfd_siginfo = unsafe { core::mem::zeroed() };

    let number_of_bytes = unsafe {
        posix::read(
            fd.native_handle(),
            ((&mut signal_info) as *mut linux::signalfd_siginfo).cast(),
            core::mem::size_of::<linux::signalfd_siginfo>(),
        )
    };

    if number_of_bytes == core::mem::size_of::<linux::signalfd_siginfo>() as _ {
        return Ok(Some(SignalInfo { signal_info }));
    }

    if number_of_bytes != -1 {
        fail!(from this,
            with SignalFdReadError::SystemBreaksReadContract,
            "{msg} since only {number_of_bytes} bytes were read but {} bytes were expected. This breaks the contract with the system.",
            core::mem::size_of::<linux::signalfd_siginfo>());
    }

    match posix::Errno::get() {
        posix::Errno::EAGAIN => Ok(None),
        posix::Errno::EINTR => {
            fail!(from this,
                with SignalFdReadError::Interrupt,
                "{msg} since an interrupt signal was raised.");
        }
        posix::Errno::EIO => {
            fail!(from this,
                with SignalFdReadError::IOerror,
                "{msg} due to an i/o error.");
        }
        posix::Errno::ENOBUFS => {
            fail!(from this,
                with SignalFdReadError::InsufficientResources,
                "{msg} due insufficient resources.");
        }
        posix::Errno::ENOMEM => {
            fail!(from this,
                with SignalFdReadError::InsufficientMemory,
                "{msg} due insufficient memory.");
        }
        e => {
            fail!(from this,
                with SignalFdReadError::UnknownError(e as _),
                "{msg} due to an unknown error ({e:?}).");
        }
    }
}

impl SignalFd {
    /// Tries to read the raised [`FetchableSignal`]. If no signal was raised it returns
    /// [`None`].
    pub fn try_read(&self) -> Result<Option<SignalInfo>, SignalFdReadError> {
        read_from_fd(self, &self.file_descriptor)
    }
}

impl FileDescriptorBased for SignalFd {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl SynchronousMultiplexing for SignalFd {}

/// Blocking version of the signal fd
#[derive(Debug)]
pub struct BlockingSignalFd {
    file_descriptor: FileDescriptor,
}

impl BlockingSignalFd {
    /// Blocks until either the raised [`FetchableSignal`] was received or an error was
    /// reported. It might have spurious wake ups.
    pub fn blocking_read(&self) -> Result<Option<SignalInfo>, SignalFdReadError> {
        read_from_fd(self, &self.file_descriptor)
    }
}

impl FileDescriptorBased for BlockingSignalFd {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl SynchronousMultiplexing for BlockingSignalFd {}
