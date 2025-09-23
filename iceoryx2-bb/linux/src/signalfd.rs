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

use std::fmt::Debug;

use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::{
    file_descriptor::FileDescriptor, process::ProcessId, signal::FetchableSignal,
    signal_set::FetchableSignalSet, user::Uid,
};
use iceoryx2_pal_os_api::linux;
use iceoryx2_pal_posix::posix::{self, MemZeroedStruct};

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SignalFdCreationError {
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    InsufficientMemory,
    UnableToMountInodeDevice,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum SignalFdReadError {
    SystemBreaksReadContract,
    Interrupt,
    IOerror,
    InsufficientResources,
    InsufficientMemory,
    UnknownError(i32),
}

#[derive(Debug)]
pub struct SignalFdBuilder {
    signal_set: FetchableSignalSet,
    close_on_exec: bool,
}

impl SignalFdBuilder {
    pub fn new(signal_set: FetchableSignalSet) -> Self {
        Self {
            signal_set,
            close_on_exec: false,
        }
    }

    pub fn set_close_on_exec(mut self) -> Self {
        self.close_on_exec = true;
        self
    }

    pub fn create_non_blocking(self) -> Result<SignalFd, SignalFdCreationError> {
        Ok(SignalFd {
            file_descriptor: self.create(false)?,
        })
    }

    pub fn create_blocking(self) -> Result<BlockingSignalFd, SignalFdCreationError> {
        Ok(BlockingSignalFd {
            file_descriptor: self.create(true)?,
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
                "This should never happen! {msg} since the signalfd returned a broked file descriptor (fd)."),
        };

        Ok(file_descriptor)
    }
}

pub struct SignalInfo {
    signal_info: linux::signalfd_siginfo,
}

impl SignalInfo {
    pub fn signal(&self) -> FetchableSignal {
        (self.signal_info.ssi_signo as i32).into()
    }

    pub fn signal_origin_pid(&self) -> ProcessId {
        ProcessId::new(self.signal_info.ssi_pid as _)
    }

    pub fn signal_origin_uid(&self) -> Uid {
        Uid::new_from_native(self.signal_info.ssi_uid)
    }
}

#[derive(Debug)]
pub struct SignalFd {
    file_descriptor: FileDescriptor,
}

fn read_from_fd<T: Debug>(
    this: &T,
    fd: &FileDescriptor,
) -> Result<Option<SignalInfo>, SignalFdReadError> {
    let msg = "Unable to read signal from SignalFd";
    let mut signal_info = linux::signalfd_siginfo::new_zeroed();

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
        posix::Errno::EAGAIN => return Ok(None),
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
    pub fn try_read(&self) -> Result<Option<SignalInfo>, SignalFdReadError> {
        read_from_fd(self, &self.file_descriptor)
    }
}

#[derive(Debug)]
pub struct BlockingSignalFd {
    file_descriptor: FileDescriptor,
}

impl BlockingSignalFd {
    pub fn blocking_read(&self) -> Result<Option<SignalInfo>, SignalFdReadError> {
        read_from_fd(self, &self.file_descriptor)
    }
}
