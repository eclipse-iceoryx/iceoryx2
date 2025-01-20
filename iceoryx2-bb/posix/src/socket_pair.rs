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

//! Abstraction of a unix streaming socket pair. Every [`StreamingSocket`] can send and receive
//! data on distinct channels, meaning that a [`StreamingSocket`] will never acquire the data
//! it has sent via a receive call.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_posix::socket_pair::*;
//!
//! let (socket_1, socket_2) = StreamingSocket::create_pair().unwrap();
//! socket_1.try_send(b"hello world").unwrap();
//!
//! let mut buffer = vec![];
//! buffer.resize(128, 0);
//! socket_2.try_receive(&mut buffer).unwrap();
//! ```
use core::sync::atomic::Ordering;
use core::time::Duration;
use iceoryx2_bb_log::fail;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;
use iceoryx2_pal_posix::posix::{self, Errno};

use crate::{
    clock::AsTimeval,
    file_descriptor::{FileDescriptor, FileDescriptorBased},
    file_descriptor_set::SynchronousMultiplexing,
    handle_errno,
};

const BLOCKING_TIMEOUT: Duration = Duration::from_secs(i16::MAX as _);

/// Defines the errors that can occur when a socket pair is created with
/// [`StreamingSocket::create_pair()`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamingSocketDuplicateError {
    PerProcessFileHandleLimitReached,
    Interrupt,
    FileDescriptorBroken,
    UnknownError(i32),
}

impl From<FcntlError> for StreamingSocketDuplicateError {
    fn from(value: FcntlError) -> Self {
        match value {
            FcntlError::Interrupt => StreamingSocketDuplicateError::Interrupt,
            FcntlError::UnknownError(v) => StreamingSocketDuplicateError::UnknownError(v),
        }
    }
}

/// Defines the errors that can occur when a socket pair is created with
/// [`StreamingSocket::create_pair()`].
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamingSocketPairCreationError {
    FileDescriptorBroken,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    InsufficientPermissions,
    InsufficientResources,
    InsufficientMemory,
    Interrupt,
    UnknownError(i32),
}

impl From<FcntlError> for StreamingSocketPairCreationError {
    fn from(value: FcntlError) -> Self {
        match value {
            FcntlError::Interrupt => StreamingSocketPairCreationError::Interrupt,
            FcntlError::UnknownError(v) => StreamingSocketPairCreationError::UnknownError(v),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum FcntlError {
    Interrupt,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum SetSockoptError {
    InsufficientResources,
    InsufficientMemory,
    UnknownError(i32),
}

/// Defines the errors that can occur when a [`StreamingSocket`] sends data via
/// * [`StreamingSocket::try_send()`]
/// * [`StreamingSocket::timed_send()`]
/// * [`StreamingSocket::blocking_send()`]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamingSocketPairSendError {
    InsufficientMemory,
    InsufficientResources,
    Interrupt,
    ConnectionReset,
    Disconnected,
    UnknownError(i32),
}

impl From<SetSockoptError> for StreamingSocketPairSendError {
    fn from(value: SetSockoptError) -> Self {
        match value {
            SetSockoptError::UnknownError(v) => StreamingSocketPairSendError::UnknownError(v),
            SetSockoptError::InsufficientResources => {
                StreamingSocketPairSendError::InsufficientResources
            }
            SetSockoptError::InsufficientMemory => StreamingSocketPairSendError::InsufficientMemory,
        }
    }
}

impl From<FcntlError> for StreamingSocketPairSendError {
    fn from(value: FcntlError) -> Self {
        match value {
            FcntlError::Interrupt => StreamingSocketPairSendError::Interrupt,
            FcntlError::UnknownError(v) => StreamingSocketPairSendError::UnknownError(v),
        }
    }
}

/// Defines the errors that can occur when a [`StreamingSocket`] receives data via
/// * [`StreamingSocket::try_receive()`]
/// * [`StreamingSocket::timed_receive()`]
/// * [`StreamingSocket::blocking_receive()`]
/// * [`StreamingSocket::peek()`]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StreamingSocketPairReceiveError {
    InsufficientMemory,
    InsufficientResources,
    ConnectionReset,
    Interrupt,
    UnknownError(i32),
}

impl From<FcntlError> for StreamingSocketPairReceiveError {
    fn from(value: FcntlError) -> Self {
        match value {
            FcntlError::Interrupt => StreamingSocketPairReceiveError::Interrupt,
            FcntlError::UnknownError(v) => StreamingSocketPairReceiveError::UnknownError(v),
        }
    }
}

impl From<SetSockoptError> for StreamingSocketPairReceiveError {
    fn from(value: SetSockoptError) -> Self {
        match value {
            SetSockoptError::UnknownError(v) => StreamingSocketPairReceiveError::UnknownError(v),
            SetSockoptError::InsufficientResources => {
                StreamingSocketPairReceiveError::InsufficientResources
            }
            SetSockoptError::InsufficientMemory => {
                StreamingSocketPairReceiveError::InsufficientMemory
            }
        }
    }
}

/// A single socket in a [`StreamingSocket`] pair.
#[derive(Debug)]
pub struct StreamingSocket {
    file_descriptor: FileDescriptor,
    is_non_blocking: IoxAtomicBool,
}

impl FileDescriptorBased for StreamingSocket {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.file_descriptor
    }
}

impl SynchronousMultiplexing for StreamingSocket {}

unsafe impl Send for StreamingSocket {}

impl StreamingSocket {
    fn create_type_safe_fd(
        raw_fd: i32,
        origin: &str,
        msg: &str,
    ) -> Result<FileDescriptor, StreamingSocketPairCreationError> {
        match FileDescriptor::new(raw_fd) {
            Some(fd) => Ok(fd),
            None => {
                fail!(from origin,
                    with StreamingSocketPairCreationError::FileDescriptorBroken,
                    "This should never happen! {msg} since the socketpair implementation returned a broken file descriptor.");
            }
        }
    }

    /// Creates a new [`StreamingSocket`] pair.
    pub fn create_pair(
    ) -> Result<(StreamingSocket, StreamingSocket), StreamingSocketPairCreationError> {
        let msg = "Unable to create streaming socket pair";
        let origin = "StreamingSocket::create_pair()";
        let mut fd_values = [0, 0];

        if unsafe {
            posix::socketpair(
                posix::AF_UNIX as _,
                posix::SOCK_STREAM,
                0,
                fd_values.as_mut_ptr(),
            )
        } == 0
        {
            let fd_1 = Self::create_type_safe_fd(fd_values[0], origin, msg)?;
            let fd_2 = Self::create_type_safe_fd(fd_values[1], origin, msg)?;
            let socket_1 = StreamingSocket {
                file_descriptor: fd_1,
                is_non_blocking: IoxAtomicBool::new(false),
            };
            fail!(from origin, when socket_1.set_non_blocking(true),
                "{msg} since the first file descriptor of the socket pair could not be set to non-blocking.");
            let socket_2 = StreamingSocket {
                file_descriptor: fd_2,
                is_non_blocking: IoxAtomicBool::new(false),
            };
            fail!(from origin, when socket_2.set_non_blocking(true),
                "{msg} since the second file descriptor of the socket pair could not be set to non-blocking.");
            return Ok((socket_1, socket_2));
        };

        handle_errno!(StreamingSocketPairCreationError, from origin,
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{msg} since the processes file descriptor limit was reached."),
            Errno::ENFILE => (SystemWideFileHandleLimitReached, "{msg} since the system wide file descriptor limit was reached."),
            Errno::EACCES => (InsufficientPermissions, "{msg} due to insufficient permissions."),
            Errno::ENOBUFS => (InsufficientResources, "{msg} due to insufficient resources."),
            Errno::ENOMEM => (InsufficientResources, "{msg} due to insufficient memory."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }

    /// Duplicates a [`StreamingSocket`]. It is connected to all existing sockets.
    pub fn duplicate(&self) -> Result<StreamingSocket, StreamingSocketDuplicateError> {
        let origin = "StreamingSocket::duplicate()";
        let msg = "Unable to duplicate StreamingSocket";
        let duplicated_fd = unsafe { posix::dup(self.file_descriptor.native_handle()) };
        if duplicated_fd != -1 {
            let new_socket = StreamingSocket {
                file_descriptor: Self::create_type_safe_fd(duplicated_fd, origin, msg)
                    .map_err(|_| StreamingSocketDuplicateError::FileDescriptorBroken)?,
                is_non_blocking: IoxAtomicBool::new(false),
            };

            fail!(from origin, when new_socket.set_non_blocking(true),
                "{msg} since the duplicated streaming socket could not be set to non-blocking.");

            return Ok(new_socket);
        }

        handle_errno!(StreamingSocketDuplicateError, from origin,
            Errno::EMFILE => (PerProcessFileHandleLimitReached, "{msg} since the processes file descriptor limit was reached."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }

    fn fcntl(&self, command: i32, value: i32, msg: &str) -> Result<i32, FcntlError> {
        let result =
            unsafe { posix::fcntl_int(self.file_descriptor.native_handle(), command, value) };

        if result >= 0 {
            return Ok(result);
        }

        handle_errno!(FcntlError, from self,
            fatal Errno::EBADF => ("This should never happen! {} since the file descriptor is invalid.", msg);
            fatal Errno::EINVAL => ("This should never happen! {} since an internal argument was invalid.", msg),
            Errno::EINTR => (Interrupt, "{} due to an interrupt signal.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn set_non_blocking(&self, value: bool) -> Result<(), FcntlError> {
        if self.is_non_blocking.load(Ordering::Relaxed) == value {
            return Ok(());
        }

        let current_flags = self.fcntl(
            posix::F_GETFL,
            0,
            "Unable to acquire current socket filedescriptor flags",
        )?;
        let new_flags = match value {
            true => current_flags | posix::O_NONBLOCK,
            false => current_flags & !posix::O_NONBLOCK,
        };

        self.fcntl(posix::F_SETFL, new_flags, "Unable to set blocking mode")?;
        self.is_non_blocking.store(value, Ordering::Relaxed);
        Ok(())
    }

    fn set_socket_option<T>(
        &self,
        msg: &str,
        value: &T,
        socket_option: posix::int,
    ) -> Result<(), SetSockoptError> {
        if unsafe {
            posix::setsockopt(
                self.file_descriptor.native_handle(),
                posix::SOL_SOCKET,
                socket_option,
                (value as *const T) as *const posix::void,
                core::mem::size_of::<T>() as u32,
            )
        } == 0
        {
            return Ok(());
        }

        handle_errno!(SetSockoptError, from self,
            fatal Errno::EBADF => ("This should never happen! {} since the file descriptor is invalid", msg);
            fatal Errno::EINVAL => ("This should never happen! {} since an argument is invalid", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            v => (UnknownError(v as i32), "{} caused by an unknown error ({}).", msg, v)
        );
    }

    fn set_send_timeout(&self, timeout: Duration) -> Result<(), SetSockoptError> {
        self.set_socket_option(
            "Unable to set send timeout",
            &timeout.as_timeval(),
            posix::SO_SNDTIMEO,
        )
    }

    fn set_receive_timeout(&self, timeout: Duration) -> Result<(), SetSockoptError> {
        self.set_socket_option(
            "Unable to set receive timeout",
            &timeout.as_timeval(),
            posix::SO_RCVTIMEO,
        )
    }

    fn send_impl(&self, msg: &str, buf: &[u8]) -> Result<usize, StreamingSocketPairSendError> {
        let number_of_bytes_written = unsafe {
            posix::send(
                self.file_descriptor.native_handle(),
                buf.as_ptr().cast(),
                buf.len(),
                0,
            )
        };

        if 0 <= number_of_bytes_written {
            return Ok(number_of_bytes_written as _);
        }

        handle_errno!(StreamingSocketPairSendError, from self,
            success Errno::EAGAIN => 0,
            fatal Errno::EBADF => ("This should never happen! {msg} since the internal file descriptor was invalid..");
            fatal Errno::EINVAL => ("This should never happen! {msg} since an internal argument was invalid."),
            Errno::EINTR => (Interrupt, "{msg} since an interrupt signal was received."),
            Errno::ECONNRESET => (ConnectionReset, "{msg} since the connection was reset."),
            Errno::EPIPE => (Disconnected, "{msg} since the socket is no longer connected."),
            Errno::ENOBUFS => (InsufficientResources, "{msg} due to insufficient resources."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }

    /// Tries to send the given buffer. It does not block, when the buffer is full it
    /// returns `0`, otherwise it returns the number of bytes that were sent.
    pub fn try_send(&self, buffer: &[u8]) -> Result<usize, StreamingSocketPairSendError> {
        let msg = "Unable to try sending message";
        fail!(from self, when self.set_non_blocking(true),
            "{msg} since the socket could not be set into non-blocking mode");
        self.send_impl(msg, buffer)
    }

    /// Blocks until either the timeout has passed or until the data could be delivered.
    /// If the timeout passed it returns `0`, otherwise the number of bytes that were sent.
    pub fn timed_send(
        &self,
        buffer: &[u8],
        timeout: Duration,
    ) -> Result<usize, StreamingSocketPairSendError> {
        let msg = "Unable to send message with a timeout";
        fail!(from self, when self.set_non_blocking(false),
            "{msg} ({timeout:?}) since the socket could not be set into blocking mode");
        fail!(from self, when self.set_send_timeout(timeout),
            "{msg} ({timeout:?}) since the socket send timeout could not be set.");
        self.send_impl(msg, buffer)
    }

    /// Blocks until the data could be delivered.
    /// Despite the name, the function may not block indefinitely and spurious wakeups can cause
    /// to return `0` when no data could be delivered.
    pub fn blocking_send(&self, buffer: &[u8]) -> Result<usize, StreamingSocketPairSendError> {
        let msg = "Unable to send message with blocking behavior";
        fail!(from self, when self.set_non_blocking(false),
            "{msg} since the socket could not be set into blocking mode");
        fail!(from self, when self.set_send_timeout(BLOCKING_TIMEOUT),
            "{msg} since the socket blocking send timeout could not be set.");
        self.send_impl(msg, buffer)
    }

    fn receive_impl(
        &self,
        msg: &str,
        buf: &mut [u8],
        flags: i32,
    ) -> Result<usize, StreamingSocketPairReceiveError> {
        let number_of_bytes_read = unsafe {
            posix::recv(
                self.file_descriptor.native_handle(),
                buf.as_mut_ptr().cast(),
                buf.len(),
                flags,
            )
        };

        if 0 <= number_of_bytes_read {
            return Ok(number_of_bytes_read as _);
        }

        handle_errno!(StreamingSocketPairReceiveError, from self,
            success Errno::EAGAIN => 0;
            success Errno::ETIMEDOUT => 0,
            fatal Errno::EBADF => ("This should never happen! {msg} since the internal file descriptor was invalid.");
            fatal Errno::EINVAL => ("This should never happen! {msg} since an internal argument was invalid."),
            Errno::EINTR => (Interrupt, "{msg} since an interrupt signal was received."),
            Errno::ECONNRESET => (ConnectionReset, "{msg} since the connection was reset."),
            Errno::ENOBUFS => (InsufficientResources, "{msg} due to insufficient resources."),
            Errno::ENOMEM => (InsufficientMemory, "{msg} due to insufficient memory."),
            v => (UnknownError(v as i32), "{msg} since an unknown error occurred ({v}).")
        )
    }

    /// Tries to receive date. It does not block, when the buffer is empty it
    /// returns `0`, otherwise it returns the number of bytes that were received.
    pub fn try_receive(&self, buf: &mut [u8]) -> Result<usize, StreamingSocketPairReceiveError> {
        let msg = "Unable to try receiving message";
        fail!(from self, when self.set_non_blocking(true),
            "{msg} since the socket could not be set into non-blocking mode");
        self.receive_impl(msg, buf, 0)
    }

    /// Blocks until either the timeout has passed or until the data could be received.
    /// If the timeout passed it returns `0`, otherwise the number of bytes that were received.
    pub fn timed_receive(
        &self,
        buf: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, StreamingSocketPairReceiveError> {
        let msg = "Unable to receive message with a timeout";
        fail!(from self, when self.set_non_blocking(false),
            "{msg} ({timeout:?}) since the socket could not be set into blocking mode");
        fail!(from self, when self.set_receive_timeout(timeout),
            "{msg} ({timeout:?}) since the socket receive timeout could not be set.");
        self.receive_impl("Unable to try receiving message", buf, 0)
    }

    /// Blocks until the data could be received.
    /// Despite the name, the function may not block indefinitely and spurious wakeups can cause
    /// to return `0` when no data could be received.
    pub fn blocking_receive(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, StreamingSocketPairReceiveError> {
        let msg = "Unable to receive message with blocking behavior";
        fail!(from self, when self.set_non_blocking(false),
            "{msg} since the socket could not be set into blocking mode");
        fail!(from self, when self.set_receive_timeout(BLOCKING_TIMEOUT),
            "{msg} since the socket blocking receive timeout could not be set.");

        self.receive_impl("Unable to try receiving message", buf, 0)
    }

    /// Tries to peek date without removing it from the internal buffer. It does not block, when
    /// the buffer is empty it returns `0`, otherwise it returns the number of bytes that were
    /// received.
    pub fn peek(&self, buf: &mut [u8]) -> Result<usize, StreamingSocketPairReceiveError> {
        let msg = "Unable to peek message";
        fail!(from self, when self.set_non_blocking(true),
            "{msg} since the socket could not be set into non-blocking mode");
        self.receive_impl(msg, buf, posix::MSG_PEEK)
    }
}
