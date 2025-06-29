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

//! Abstraction of datagram based unix domain sockets. The [`UnixDatagramReceiver`] creates a
//! socket and the [`UnixDatagramSender`] can connect to it and send messages.
//!
//! # Example
//!
//! ## Transfer data
//!
//! ```
//! use iceoryx2_bb_posix::unix_datagram_socket::*;
//! use iceoryx2_bb_posix::permission::*;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let socket_name = FilePath::new(b"mySocket").unwrap();
//! let receiver = UnixDatagramReceiverBuilder::new(&socket_name)
//!                         .permission(Permission::OWNER_ALL)
//!                         .creation_mode(CreationMode::PurgeAndCreate)
//!                         .create().unwrap();
//!
//! let sender = UnixDatagramSenderBuilder::new(&socket_name)
//!                         .create().unwrap();
//!
//! // send some data
//! let data: Vec<u8> = vec![1u8, 2u8, 3u8, 4u8, 5u8];
//! sender.try_send(data.as_slice()).unwrap();
//!
//! // receive some data
//! let mut recv_data: Vec<u8> = vec![];
//! recv_data.resize(5, 0);
//! receiver.try_receive(recv_data.as_mut_slice()).unwrap();
//! ```
//!
//! ## Transfer [`SocketCred`]s
//!
//! ```ignore
//! use iceoryx2_bb_posix::unix_datagram_socket::*;
//! use iceoryx2_bb_posix::socket_ancillary::*;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let socket_name = FilePath::new(b"myFunSocket").unwrap();
//! let receiver = UnixDatagramReceiverBuilder::new(&socket_name)
//!                         .creation_mode(CreationMode::PurgeAndCreate)
//!                         .create().unwrap();
//!
//! let sender = UnixDatagramSenderBuilder::new(&socket_name)
//!                         .create().unwrap();
//!
//! // send credentials (pid, uid and gid of the connected sender)
//! let mut msg = SocketAncillary::new();
//! msg.set_creds(&SocketCred::new());
//! sender.try_send_msg(&mut msg).unwrap();
//!
//! // receive credentials
//! let mut recv_msg = SocketAncillary::new();
//! receiver.try_receive_msg(&mut recv_msg).unwrap();
//!
//! match recv_msg.get_creds() {
//!     Some(cred) => { println!("received credentials {}", cred); }
//!     None => { println!("No message received."); }
//! };
//! ```
//!
//! ## Transfer [`FileDescriptor`]s
//!
//! ```no_run
//! use iceoryx2_bb_posix::unix_datagram_socket::*;
//! use iceoryx2_bb_posix::socket_ancillary::*;
//! use iceoryx2_bb_posix::file::*;
//! use iceoryx2_bb_posix::file_descriptor::*;
//! use iceoryx2_bb_system_types::file_path::FilePath;
//! use iceoryx2_bb_container::semantic_string::SemanticString;
//!
//! let socket_name = FilePath::new(b"mySocket").unwrap();
//! let file_name = FilePath::new(b"udsExampleFile").unwrap();
//! let receiver = UnixDatagramReceiverBuilder::new(&socket_name)
//!                         .creation_mode(CreationMode::PurgeAndCreate)
//!                         .create().unwrap();
//!
//! let sender = UnixDatagramSenderBuilder::new(&socket_name)
//!                         .create().unwrap();
//!
//! let file = FileBuilder::new(&file_name)
//!                     .creation_mode(CreationMode::PurgeAndCreate)
//!                     .create().unwrap();
//!
//! // send credentials (pid, uid and gid of the connected sender)
//! let mut msg = SocketAncillary::new();
//! if msg.add_fd(file.file_descriptor().clone()) {
//!     println!("No more space left in message for another file descriptor.");
//! }
//! sender.try_send_msg(&mut msg).unwrap();
//!
//! // receive credentials
//! let mut recv_msg = SocketAncillary::new();
//! receiver.blocking_receive_msg(&mut recv_msg).unwrap();
//!
//! let mut fd_vec = recv_msg.extract_fds();
//! if fd_vec.is_empty() {
//!     println!("No file descriptors received.");
//! }
//! let recv_file = File::from_file_descriptor(fd_vec.remove(0));
//!
//! // cleanup
//! File::remove(&file_name);
//! ```

use crate::clock::AsTimeval;
use crate::file_descriptor::{FileDescriptor, FileDescriptorBased, FileDescriptorManagement};
use crate::file_descriptor_set::SynchronousMultiplexing;
use crate::socket_ancillary::*;
use core::mem::MaybeUninit;
use core::sync::atomic::Ordering;
use core::{mem::size_of, time::Duration};
use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_elementary::enum_gen;
use iceoryx2_bb_elementary::scope_guard::ScopeGuardBuilder;
use iceoryx2_bb_log::{fail, fatal_panic, trace};
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;
use iceoryx2_pal_posix::posix::{errno::Errno, MemZeroedStruct};

use crate::{config::UNIX_DOMAIN_SOCKET_PATH_LENGTH, file::*, permission::Permission};

pub use crate::creation_mode::CreationMode;
use iceoryx2_pal_posix::*;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum UnixDatagramCreationError {
    SocketNameTooLong,
    InsufficientPermissions,
    InsufficientResources,
    InsufficientMemory,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    DatagramProtocolNotSupported,
    UnixDomainSocketsNotSupported,
    UnknownError(i32),
}

enum_gen! {
    UnixDatagramSenderCreationError
  entry:
    InsufficientPermissions,
    InsufficientResources,
    AlreadyConnected,
    ConnectionRefused,
    Interrupt,
    ConnectionReset,
    WouldBlock,
    DoesNotExist,
    UnknownError(i32)
  mapping:
    UnixDatagramCreationError
}

enum_gen! {
    UnixDatagramReceiverCreationError
  entry:
    SocketFileAlreadyExists,
    InsufficientResources,
    InsufficientPermissions,
    AddressAlreadyInUse,
    PathDoesNotExist,
    ReadOnlyFileSytem,
    UnknownError(i32)
  mapping:
    UnixDatagramSetSocketOptionError,
    UnixDatagramCreationError,
    FileAccessError,
    FileRemoveError
}

enum_gen! {
    UnixDatagramSendError
  entry:
    MessageTooLarge,
    ConnectionReset,
    ConnectionRefused,
    Interrupt,
    IOerror,
    InsufficientPermissions,
    InsufficientResources,
    InsufficientMemory,
    NotConnected,
    MessagePartiallySend(u64),
    UnknownError(i32)
  mapping:
    UnixDatagramSetPropertyError,
    UnixDatagramSetSocketOptionError
}

enum_gen! {
    UnixDatagramSendMsgError
  entry:
    MessageTooLarge,
    ConnectionReset,
    Interrupt,
    IOerror,
    InsufficientPermissions,
    InsufficientResources,
    InsufficientMemory,
    NotConnected,
    MaximumSupportedMessagesExceeded,
    MessagePartiallySend(u64),
    UnknownError(i32)
  mapping:
    UnixDatagramSetPropertyError,
    UnixDatagramSetSocketOptionError
}

enum_gen! {
    UnixDatagramReceiveError
  entry:
    ConnectionReset,
    Interrupt,
    NotConnected,
    IOerror,
    InsufficientResources,
    InsufficientMemory,
    UnknownError(i32)
  mapping:
    UnixDatagramSetPropertyError,
    UnixDatagramSetSocketOptionError
}

enum_gen! {
    UnixDatagramReceiveFdError
  entry:
    WouldBlock,
    ConnectionReset,
    Interrupt,
    NotConnected,
    IOerror,
    InsufficientResources,
    InsufficientMemory,
    ReceivedUnexpectedMessage,
    ReceivedInvalidFileDescriptor,
    UnknownError(i32)
  mapping:
    UnixDatagramSetPropertyError,
    UnixDatagramSetSocketOptionError
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum UnixDatagramSetSocketOptionError {
    InsufficientMemory,
    InsufficientResources,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum UnixDatagramGetSocketOptionError {
    InsufficientPermissions,
    InsufficientResources,
    SocketHasBeenShutDown,
    UnknownError(i32),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum UnixDatagramSetPropertyError {
    Interrupt,
    WouldCauseOverflow,
    UnknownError(i32),
}

enum_gen! {
    /// The UnixDatagramError enum is a generalization when one doesn't require the fine-grained error
    /// handling enums. One can forward UnixDatagramError as more generic return value when a method
    /// returns a UnixDatagram***Error.
    /// On a higher level it is again convertable to [`crate::Error`].
    UnixDatagramError
  generalization:
    CreationFailed <= UnixDatagramSenderCreationError; UnixDatagramReceiverCreationError,
    SetupFailure <= UnixDatagramSetSocketOptionError; UnixDatagramSetPropertyError,
    SendFailed <= UnixDatagramSendError,
    ReceiveFailed <= UnixDatagramReceiveError
}

const BLOCKING_TIMEOUT: Duration = Duration::from_secs(i16::MAX as _);

#[derive(Debug)]
struct UnixDatagramSocket {
    name: FilePath,
    is_non_blocking: IoxAtomicBool,
    file_descriptor: FileDescriptor,
}

impl UnixDatagramSocket {
    fn fcntl(
        &self,
        command: i32,
        value: i32,
        msg: &str,
    ) -> Result<i32, UnixDatagramSetPropertyError> {
        let result =
            unsafe { posix::fcntl_int(self.file_descriptor.native_handle(), command, value) };

        if result >= 0 {
            return Ok(result);
        }

        handle_errno!(UnixDatagramSetPropertyError, from self,
            fatal Errno::EBADF => ("This should never happen! {} since the file descriptor is invalid.", msg);
            fatal Errno::EINVAL => ("This should never happen! {} since an internal argument was invalid.", msg),
            Errno::EOVERFLOW => (WouldCauseOverflow, "{} since the operation would cause an overflow.", msg),
            Errno::EINTR => (Interrupt, "{} due to an interrupt signal.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn set_non_blocking(&self, value: bool) -> Result<(), UnixDatagramSetPropertyError> {
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
    ) -> Result<(), UnixDatagramSetSocketOptionError> {
        if unsafe {
            posix::setsockopt(
                self.file_descriptor.native_handle(),
                CMSG_SOCKET_LEVEL,
                socket_option,
                (value as *const T) as *const posix::void,
                core::mem::size_of::<T>() as u32,
            )
        } == 0
        {
            return Ok(());
        }

        handle_errno!(UnixDatagramSetSocketOptionError, from self,
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            v => (UnknownError(v as i32), "{} caused by an unknown error ({}).", msg, v)
        );
    }

    fn get_socket_option<T>(
        &self,
        msg: &str,
        socket_option: posix::int,
    ) -> Result<T, UnixDatagramGetSocketOptionError> {
        let mut value: MaybeUninit<T> = MaybeUninit::uninit();
        let mut value_len: posix::socklen_t = core::mem::size_of::<T>() as posix::socklen_t;

        if unsafe {
            posix::getsockopt(
                self.file_descriptor.native_handle(),
                CMSG_SOCKET_LEVEL,
                socket_option,
                value.as_mut_ptr() as *mut posix::void,
                &mut value_len,
            )
        } == 0
        {
            return Ok(unsafe { value.assume_init() });
        }

        handle_errno!(UnixDatagramGetSocketOptionError, from self,
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::EINVAL => (SocketHasBeenShutDown, "{} since the socket has been shut down.", msg),
            v => (UnknownError(v as i32), "{} caused by an unknown error ({}).", msg, v)
        );
    }

    fn create_socket_address(&self) -> posix::sockaddr_un {
        let mut socket_address = posix::sockaddr_un::new_zeroed();
        socket_address.sun_family = posix::AF_UNIX;

        unsafe {
            posix::strncpy(
                socket_address.sun_path.as_mut_ptr(),
                self.name.as_c_str(),
                self.name.len(),
            );
        }

        socket_address
    }

    fn bind(&self, permission: Permission) -> Result<(), UnixDatagramReceiverCreationError> {
        let socket_address = self.create_socket_address();
        let ptr: *const posix::sockaddr_un = &socket_address;

        {
            let _mask = ScopeGuardBuilder::new(0 as posix::mode_t)
                .on_init(|mask| -> Result<(), ()> {
                    *mask = unsafe { posix::umask((!permission).bits()) };
                    Ok(())
                })
                .on_drop(|mask| unsafe {
                    posix::umask(*mask);
                })
                .create();

            if unsafe {
                posix::bind(
                    self.file_descriptor.native_handle(),
                    ptr as *const posix::sockaddr,
                    size_of::<posix::sockaddr_un>() as u32,
                )
            } == 0
            {
                return Ok(());
            }
        }

        let msg = "Failed to bind socket";
        handle_errno!(UnixDatagramReceiverCreationError, from self,
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EADDRINUSE => (AddressAlreadyInUse, "{} since the address is already in use.", msg),
            Errno::ENOENT => (PathDoesNotExist, "{} since the path does not exist.", msg),
            Errno::ENOTDIR => (PathDoesNotExist, "{} since the path does not exist.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::EROFS => (ReadOnlyFileSytem, "{} since it would reside on an read-only file system.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error has occurred ({}).", msg, v)
        );
    }

    fn connect(&self) -> Result<(), UnixDatagramSenderCreationError> {
        let socket_address = self.create_socket_address();
        let ptr: *const posix::sockaddr_un = &socket_address;
        if unsafe {
            posix::connect(
                self.file_descriptor.native_handle(),
                ptr as *const posix::sockaddr,
                size_of::<posix::sockaddr_un>() as u32,
            )
        } == 0
        {
            return Ok(());
        }

        let msg = "Failed to connect";
        handle_errno!(UnixDatagramSenderCreationError, from self,
            Errno::ENOENT => (DoesNotExist, "{} since the unix datagram receiver does not exist.", msg),
            Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
            Errno::EADDRINUSE => (AlreadyConnected, "{} since it is already connected.", msg),
            Errno::ECONNREFUSED => (ConnectionRefused, "{} since the connection was refused.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt was received.", msg),
            Errno::ECONNRESET => (ConnectionReset, "{} since the host reset the connection request.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} since there is no buffer space available.", msg),
            Errno::EINPROGRESS => (WouldBlock, "{} since the operation would block the process. Allow blocking and the connection can may be established.", msg),
            v => (UnknownError(v as i32), "{} caused by an unknown error ({}).", msg, v)
        );
    }

    fn new(name: &FilePath) -> Result<Self, UnixDatagramCreationError> {
        if name.len() > UNIX_DOMAIN_SOCKET_PATH_LENGTH {
            fail!(with UnixDatagramCreationError::SocketNameTooLong,
                "The name \"{}\" is too long for a UnixDatagramSocket name. Maximum supported length is {}.", name, UNIX_DOMAIN_SOCKET_PATH_LENGTH);
        }

        let raw_fd = unsafe { posix::socket(posix::PF_UNIX as posix::int, posix::SOCK_DGRAM, 0) };

        let msg = format!("Unable to create UnixDatagramSocket named \"{name}\"");
        if raw_fd < 0 {
            handle_errno!(UnixDatagramCreationError, from "UnixDatagramSocket::new",
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process limit of file descriptors was reached.", msg),
                Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since system-wide limit of file descriptors was reached.", msg),
                Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
                Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                Errno::EPROTONOSUPPORT => (DatagramProtocolNotSupported, "{} since the datagram protocol is not supported by the system.", msg),
                Errno::EPROTOTYPE => (UnixDomainSocketsNotSupported, "{} since UnixDomainSockets are not supported by the system.", msg),
                v => (UnknownError(v as i32), "Unable to create socket since an unknown error occurred ({}).", v)
            );
        }

        Ok(Self {
            name: name.clone(),
            is_non_blocking: IoxAtomicBool::new(false),
            file_descriptor: FileDescriptor::new(raw_fd).unwrap(),
        })
    }
}

/// Creates a [`UnixDatagramSender`]. It requires that a [`UnixDatagramReceiver`] has already
/// created the socket, otherwise a connection failure will occur.
#[derive(Debug)]
pub struct UnixDatagramSenderBuilder {
    name: FilePath,
}

impl UnixDatagramSenderBuilder {
    pub fn new(name: &FilePath) -> Self {
        Self { name: name.clone() }
    }

    /// Creates a new [`UnixDatagramSender`].
    pub fn create(self) -> Result<UnixDatagramSender, UnixDatagramSenderCreationError> {
        UnixDatagramSender::new(self)
    }
}

/// Created by the [`UnixDatagramSenderBuilder`]. Connect to an existing socket and sends data with
/// [`UnixDatagramSender::try_send()`] or message which contain [`FileDescriptor`]s or [`SocketCred`]
/// with [`UnixDatagramSender::try_send_msg()`].
#[derive(Debug)]
pub struct UnixDatagramSender {
    socket: UnixDatagramSocket,
}

impl Drop for UnixDatagramSender {
    fn drop(&mut self) {
        trace!(from self, "disconnected");
    }
}

impl UnixDatagramSender {
    fn new(config: UnixDatagramSenderBuilder) -> Result<Self, UnixDatagramSenderCreationError> {
        let msg = "Failed to created UnixDatagramSender";
        let new_socket = UnixDatagramSender {
            socket: fail!(from config, when UnixDatagramSocket::new(&config.name), "{}.", msg),
        };

        match new_socket.socket.connect() {
            Err(UnixDatagramSenderCreationError::DoesNotExist) => {
                fail!(from config, with UnixDatagramSenderCreationError::DoesNotExist,
                    "{} since the connection could not be established.", msg);
            }
            Err(v) => {
                return Err(v);
            }
            Ok(_) => (),
        };

        trace!(from new_socket, "connected");

        Ok(new_socket)
    }

    /// Returns the name of the socket
    pub fn name(&self) -> &FilePath {
        &self.socket.name
    }

    fn set_non_blocking(&self, value: bool) -> Result<(), UnixDatagramSetPropertyError> {
        self.socket.set_non_blocking(value)
    }

    fn set_timeout(&self, timeout: Duration) -> Result<(), UnixDatagramSetSocketOptionError> {
        self.socket.set_socket_option(
            "Unable to set send timeout",
            &timeout.as_timeval(),
            posix::SO_SNDTIMEO,
        )
    }

    /// Sets the send buffer minimum size. The operating system is allowed to increase the value
    /// but it is guaranteed that the buffer size is at least the provided value.
    pub fn set_send_buffer_min_size(
        &mut self,
        value: usize,
    ) -> Result<(), UnixDatagramSetSocketOptionError> {
        let temp = value as posix::int;
        self.socket
            .set_socket_option("Unable to set send buffer size", &temp, posix::SO_SNDBUF)
    }

    /// Returns the send buffer size
    pub fn get_send_buffer_size(&self) -> Result<usize, UnixDatagramGetSocketOptionError> {
        Ok(self.socket.get_socket_option::<posix::int>(
            "Unable to acquire send buffer size",
            posix::SO_SNDBUF,
        )? as usize)
    }

    fn send_msg(&self, uds_msg: &mut SocketAncillary) -> Result<bool, UnixDatagramSendMsgError> {
        uds_msg.prepare_for_send();

        let msg = "Unable to send unix domain socket message";
        const FLAGS: i32 = 0;
        let bytes_sent = unsafe {
            posix::sendmsg(
                self.socket.file_descriptor.native_handle(),
                uds_msg.get(),
                FLAGS,
            )
        };

        if bytes_sent > 0 {
            if bytes_sent as usize == uds_msg.len() {
                fail!(from self, with UnixDatagramSendMsgError::MessagePartiallySend(bytes_sent as u64),
                    "{} since only {} bytes were sent. {} bytes remain unsent.", msg, bytes_sent, uds_msg.len() - bytes_sent as usize );
            }

            return Ok(true);
        }

        handle_errno!(UnixDatagramSendMsgError, from self,
            success Errno::EAGAIN => false,
            fatal Errno::EINVAL => ("{} {} due to an implementation error. The msghdr.msg_controllen size does not fit the used cmsghdrs.", msg, uds_msg),
            Errno::ECONNRESET => (ConnectionReset, "{} {} since the connection was reset by peer.", msg, uds_msg),
            Errno::EINTR => (Interrupt, "{} {} since an interrupt signal was received.", msg, uds_msg),
            Errno::EMSGSIZE => (MessageTooLarge, "{} {} since the message size of {} bytes is too large to be send in one package.", msg, uds_msg, uds_msg.len()),
            Errno::EIO => (IOerror, "{} {} since an I/O error occurred while writing to the file system.", msg, uds_msg),
            Errno::EACCES => (InsufficientPermissions, "{} {} due to insufficient permissions.", msg, uds_msg),
            Errno::EPERM => (InsufficientPermissions, "{} {} due to insufficient permissions.", msg, uds_msg),
            Errno::ENOBUFS => (InsufficientResources, "{} {} due to insufficient resources.", msg, uds_msg),
            Errno::ENOMEM => (InsufficientMemory, "{} {} due to insufficient memory.", msg, uds_msg),
            Errno::ENOTCONN => (NotConnected, "{} {} since the socket is not yet connected.", msg, uds_msg),
            v => (UnknownError(v as i32), "{} {} since an unknown error occurred ({}).", msg, uds_msg, v)
        );
    }

    /// Tries to send a [`SocketAncillary`] message that can contain [`FileDescriptor`]s and [`SocketCred`].
    /// Returns true if the message was sent, otherwise false.
    pub fn try_send_msg(
        &self,
        uds_msg: &mut SocketAncillary,
    ) -> Result<bool, UnixDatagramSendMsgError> {
        fail!(from self, when self.set_non_blocking(true),
                "Unable to try send message since the socket could not bet set into unblocking state.");
        self.send_msg(uds_msg)
    }

    /// Blocks until the [`SocketAncillary`] message can be sent or the timeout has passed.
    /// Returns true if the message was sent, otherwise false.
    pub fn timed_send_msg(
        &self,
        uds_msg: &mut SocketAncillary,
        timeout: Duration,
    ) -> Result<bool, UnixDatagramSendMsgError> {
        let msg = "Unable to timed send message";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(timeout),
                "{} since the socket timeout could not be set.", msg);
        self.send_msg(uds_msg)
    }

    /// Blocks until the [`SocketAncillary`] message can be sent.
    pub fn blocking_send_msg(
        &self,
        uds_msg: &mut SocketAncillary,
    ) -> Result<(), UnixDatagramSendMsgError> {
        let msg = "Unable to blocking send message";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(BLOCKING_TIMEOUT),
                "{} since the socket blocking timeout could not be set.", msg);
        self.send_msg(uds_msg)?;
        Ok(())
    }

    fn send(&self, data: &[u8]) -> Result<bool, UnixDatagramSendError> {
        let bytes_sent = unsafe {
            posix::sendto(
                self.socket.file_descriptor.native_handle(),
                data.as_ptr() as *const posix::void,
                data.len(),
                0,
                core::ptr::null::<posix::sockaddr>(),
                0,
            )
        };

        let msg = "Unable to send message";
        if bytes_sent < 0 {
            handle_errno!(UnixDatagramSendError, from self,
                success Errno::EAGAIN => false,
                Errno::ECONNRESET => (ConnectionReset, "{} since the connection was reset by peer.", msg),
                Errno::ECONNREFUSED => (ConnectionRefused, "{} since the connection was refused by peer.", msg),
                Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                Errno::EMSGSIZE => (MessageTooLarge, "{} since the message size of {} bytes is too large to be send in one package.", msg, data.len()),
                Errno::EIO => (IOerror, "{} since an I/O error occurred while writing to the file system.", msg),
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
                Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                Errno::ENOTCONN => (NotConnected, "{} since the socket is not yet connected.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg,v)
            );
        }

        if data.len() != bytes_sent as usize {
            fail!(from self, with UnixDatagramSendError::MessagePartiallySend(bytes_sent as u64),
                "{} since only parts of the message of length {}. {} bytes were sent and {} remain unsent.", msg, data.len(), bytes_sent, data.len() - bytes_sent as usize );
        }

        Ok(true)
    }

    /// Tries to sent data in a non-blocking way.
    /// If the data was sent it returns true, otherwise false.
    pub fn try_send(&self, data: &[u8]) -> Result<bool, UnixDatagramSendError> {
        fail!(from self, when self.set_non_blocking(true),
                "Unable to try send data since the socket could not bet set into unblocking state.");
        self.send(data)
    }

    /// Blocks until the data was sent or the timeout has passed.
    /// If the data was sent it returns true, otherwise false.
    pub fn timed_send(&self, data: &[u8], timeout: Duration) -> Result<(), UnixDatagramSendError> {
        let msg = "Unable to timed send data";
        fail!(from self, when self.set_non_blocking(true),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(timeout),
                "{} since the socket timeout could not be set.", msg);
        self.send(data)?;
        Ok(())
    }

    /// Blocks until the data was sent.
    pub fn blocking_send(&self, data: &[u8]) -> Result<(), UnixDatagramSendError> {
        let msg = "Unable to blocking send data";
        fail!(from self, when self.set_non_blocking(true),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(BLOCKING_TIMEOUT),
                "{} since the socket blocking timeout could not be set.", msg);
        self.send(data)?;
        Ok(())
    }
}

impl FileDescriptorBased for UnixDatagramSender {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.socket.file_descriptor
    }
}

impl FileDescriptorManagement for UnixDatagramSender {}

impl SynchronousMultiplexing for UnixDatagramSender {}

/// Creates a [`UnixDatagramReceiver`]. Must be created before the [`UnixDatagramSender`] since the
/// sender connects to the receiver.
#[derive(Debug)]
pub struct UnixDatagramReceiverBuilder {
    name: FilePath,
    permission: Permission,
    creation_mode: CreationMode,
}

impl UnixDatagramReceiverBuilder {
    pub fn new(name: &FilePath) -> Self {
        Self {
            name: name.clone(),
            permission: Permission::OWNER_ALL,
            creation_mode: CreationMode::CreateExclusive,
        }
    }

    /// Sets the permission of the corresponding socket file
    pub fn permission(mut self, permission: Permission) -> Self {
        self.permission = permission;
        self
    }

    /// Defines the creation mode
    pub fn creation_mode(mut self, value: CreationMode) -> Self {
        self.creation_mode = value;
        self
    }

    pub fn create(self) -> Result<UnixDatagramReceiver, UnixDatagramReceiverCreationError> {
        UnixDatagramReceiver::new(self)
    }
}

/// Created by the [`UnixDatagramReceiverBuilder`]. Creates a new socket to which a sender can
/// connect. It can either receive data with [`UnixDatagramReceiver::try_receive()`] or
/// [`SocketAncillary`] messages which can contain [`SocketCred`] or [`FileDescriptor`] with
/// [`UnixDatagramReceiver::try_receive_msg()`].
#[derive(Debug)]
pub struct UnixDatagramReceiver {
    socket: UnixDatagramSocket,
}

impl Drop for UnixDatagramReceiver {
    fn drop(&mut self) {
        fatal_panic!(from self, when File::remove(&self.socket.name), "Failed to remove socket file.");
        trace!(from self, "stop listening and remove");
    }
}

impl UnixDatagramReceiver {
    fn new(config: UnixDatagramReceiverBuilder) -> Result<Self, UnixDatagramReceiverCreationError> {
        let msg = "Unable to create new socket";
        let new_socket = Self {
            socket: fail!(from config, when UnixDatagramSocket::new(&config.name), "{}.", msg),
        };

        let does_file_exist = fail!(from new_socket, when File::does_exist(&config.name), "Unable to determine if socket exists.");

        if config.creation_mode == CreationMode::PurgeAndCreate && does_file_exist {
            fail!(from new_socket, when File::remove(&config.name), "{} since the already existing socket could not be removed.", msg);
        } else if config.creation_mode == CreationMode::CreateExclusive && does_file_exist {
            fail!(from new_socket, with UnixDatagramReceiverCreationError::SocketFileAlreadyExists, "{} since it already exists.", msg);
        }

        fail!(from new_socket, when new_socket.socket.bind(config.permission), "{} since the socket could not be bind.", msg);

        if posix::POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA {
            fail!(from new_socket, when new_socket.socket.set_socket_option("Unable to activate credential support", &1u32, posix::SO_PASSCRED),
                "{} since the credential support could not be activated.", msg);
        }

        trace!(from new_socket, "create and listening");
        Ok(new_socket)
    }

    /// Returns the name of the socket
    pub fn name(&self) -> &FilePath {
        &self.socket.name
    }

    fn set_non_blocking(&self, value: bool) -> Result<(), UnixDatagramSetPropertyError> {
        self.socket.set_non_blocking(value)
    }

    fn set_timeout(&self, timeout: Duration) -> Result<(), UnixDatagramSetSocketOptionError> {
        self.socket.set_socket_option(
            "Unable to set receive timeout",
            &timeout.as_timeval(),
            posix::SO_RCVTIMEO,
        )
    }

    /// Sets the receive buffer minimum size. The operating system is allowed to increase the value
    /// but it is guaranteed that the buffer size is at least the provided value.
    pub fn set_receive_buffer_min_size(
        &mut self,
        value: usize,
    ) -> Result<(), UnixDatagramSetSocketOptionError> {
        let temp = value as posix::int;
        self.socket
            .set_socket_option("Unable to set receive buffer size", &temp, posix::SO_RCVBUF)
    }

    /// Returns the receive buffer size
    pub fn get_receive_buffer_size(&self) -> Result<usize, UnixDatagramGetSocketOptionError> {
        Ok(self.socket.get_socket_option::<posix::int>(
            "Unable to acquire receive buffer size",
            posix::SO_RCVBUF,
        )? as usize)
    }

    /// Tries to receive data from a [`UnixDatagramSender`]. If no data is present it will not
    /// block and return 0.
    pub fn try_receive(&self, buffer: &mut [u8]) -> Result<u64, UnixDatagramReceiveError> {
        fail!(from self, when self.set_non_blocking(true),
                "Unable to try receive data since the socket could not bet set into unblocking state.");
        self.internal_receive(0, buffer)
    }

    /// Tries to receive data from a [`UnixDatagramSender`] and blocks until either the timeout has
    /// passed or data has been received. If no data was received it returns 0.
    pub fn timed_receive(
        &self,
        buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<u64, UnixDatagramReceiveError> {
        let msg = "Unable to timed receive data";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(timeout),
                "{} since the socket timeout could not be set.", msg);
        self.internal_receive(0, buffer)
    }

    /// Blocks until data was received from the [`UnixDatagramSender`].
    pub fn blocking_receive(&self, buffer: &mut [u8]) -> Result<u64, UnixDatagramReceiveError> {
        let msg = "Unable to blocking receive data";

        loop {
            fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
            fail!(from self, when self.set_timeout(BLOCKING_TIMEOUT),
                "{} since the socket blocking timeout could not be set.", msg);

            match self.internal_receive(0, buffer) {
                Ok(0) => (),
                Ok(v) => return Ok(v),
                Err(e) => return Err(e),
            }
        }
    }

    /// Tries to peek data. It is like [`UnixDatagramReceiver::try_receive()`] but the data is not removed from
    /// the queue and remains to be peeked or received again. If no data present it returns 0.
    pub fn try_peek(&self, buffer: &mut [u8]) -> Result<u64, UnixDatagramReceiveError> {
        fail!(from self, when self.set_non_blocking(true),
                "Unable to try peek data since the socket could not bet set into unblocking state.");
        self.internal_receive(posix::MSG_PEEK, buffer)
    }

    /// Tries to peek data. It is like [`UnixDatagramReceiver::timed_receive()`] but the data is not removed from
    /// the queue and remains to be peeked or received again. If no data present it returns 0.
    pub fn timed_peek(
        &self,
        buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<u64, UnixDatagramReceiveError> {
        let msg = "Unable to timed peek data";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(timeout),
                "{} since the socket timeout could not be set.", msg);
        self.internal_receive(posix::MSG_PEEK, buffer)
    }

    /// Blocks until data can be peeked. It is like [`UnixDatagramReceiver::blocking_receive()`] but the data
    /// is not removed from the queue and remains to be peeked or received again.
    pub fn blocking_peek(&self, buffer: &mut [u8]) -> Result<u64, UnixDatagramReceiveError> {
        let msg = "Unable to blocking peek data";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(BLOCKING_TIMEOUT),
                "{} since the socket blocking timeout could not be set.", msg);
        self.internal_receive(posix::MSG_PEEK, buffer)
    }

    fn receive_msg(
        &self,
        socket_msg: &mut SocketAncillary,
    ) -> Result<bool, UnixDatagramReceiveFdError> {
        socket_msg.clear();

        let msg = "Unable to receive file descriptor";
        match unsafe {
            posix::recvmsg(
                self.socket.file_descriptor.native_handle(),
                socket_msg.get_mut(),
                0,
            )
        } {
            1..=isize::MAX => {
                socket_msg.extract_received_data(self);
                Ok(true)
            }
            _ => {
                handle_errno!(UnixDatagramReceiveFdError, from self,
                    success Errno::ETIMEDOUT => false;
                    success Errno::EAGAIN => false,
                    Errno::ECONNRESET => (ConnectionReset, "{} since connection was forcibly closed.", msg),
                    Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
                    Errno::ENOTCONN => (NotConnected, "{} since socket is not connected.", msg),
                    Errno::EIO => (IOerror, "{} since an I/O error occurred while reading from the file system.", msg),
                    Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
                    Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
                    v => (UnknownError(v as i32), "{} due to an unknown error ({}).", msg, v)
                )
            }
        }
    }

    /// Tries to receives a [`SocketAncillary`] message which can contain [`FileDescriptor`]s or
    /// [`SocketCred`]. It returns true when a message was received, otherwise false.
    pub fn try_receive_msg(
        &self,
        socket_msg: &mut SocketAncillary,
    ) -> Result<bool, UnixDatagramReceiveFdError> {
        fail!(from self, when self.set_non_blocking(true),
                "Unable to try receive message since the socket could not bet set into unblocking state.");
        self.receive_msg(socket_msg)
    }

    /// Blocks until it receives a [`SocketAncillary`] message which can contain [`FileDescriptor`]s or
    /// [`SocketCred`] or the timeout has passed. It returns true when a message was received,
    /// otherwise false.
    pub fn timed_receive_msg(
        &self,
        socket_msg: &mut SocketAncillary,
        timeout: Duration,
    ) -> Result<bool, UnixDatagramReceiveFdError> {
        let msg = "Unable to timed receive message";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(timeout),
                "{} since the socket timeout could not be set.", msg);
        self.receive_msg(socket_msg)
    }

    /// Blocks until it receives a [`SocketAncillary`] message which can contain [`FileDescriptor`]s or
    /// [`SocketCred`]. It returns true when a message was received, otherwise false.
    pub fn blocking_receive_msg(
        &self,
        socket_msg: &mut SocketAncillary,
    ) -> Result<bool, UnixDatagramReceiveFdError> {
        let msg = "Unable to blocking receive message";
        fail!(from self, when self.set_non_blocking(false),
                "{} since the socket could not bet set into blocking state.", msg);
        fail!(from self, when  self.set_timeout(BLOCKING_TIMEOUT),
                "{} since the socket blocking timeout could not be set.", msg);
        self.receive_msg(socket_msg)
    }

    fn internal_receive(
        &self,
        flags: posix::int,
        buffer: &mut [u8],
    ) -> Result<u64, UnixDatagramReceiveError> {
        let bytes_received = unsafe {
            posix::recvfrom(
                self.socket.file_descriptor.native_handle(),
                buffer.as_mut_ptr() as *mut posix::void,
                buffer.len(),
                flags,
                core::ptr::null_mut::<posix::sockaddr>(),
                core::ptr::null_mut::<u32>(),
            )
        };

        if bytes_received >= 0 {
            return Ok(bytes_received as u64);
        }

        let msg = "Unable to receive data";
        handle_errno!(UnixDatagramReceiveError, from self,
            success Errno::ETIMEDOUT => 0;
            success Errno::EAGAIN => 0,
            Errno::ECONNRESET => (ConnectionReset, "{} since connection was forcibly closed.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::EIO => (IOerror, "{} since an I/O error occurred while reading from the file system.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} due to an unknown error({}).", msg, v)
        );
    }
}

impl FileDescriptorBased for UnixDatagramReceiver {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.socket.file_descriptor
    }
}

impl FileDescriptorManagement for UnixDatagramReceiver {}

impl SynchronousMultiplexing for UnixDatagramReceiver {}
