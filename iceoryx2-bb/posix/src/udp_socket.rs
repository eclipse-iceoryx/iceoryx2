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

//! Abstraction of an UDP socket.
//!
//! The [`UdpServerBuilder`] creates an [`UdpServer`] that can
//! [send](UdpServer::send_to()) to and [receive](UdpServer::try_receive_from()) from a
//! network client.
//!
//! The [`UdpClientBuilder`] creates an [`UdpClient`] that can
//! [send](UdpServer::send_to()) to and [receive](UdpServer::try_receive_from()) from a
//! network server.
//!
//! # Example
//!
//! ```ignore
//! use iceoryx2_bb_posix::udp_socket::*;
//!
//! let server = UdpServerBuilder::new().listen()
//!                     .expect("Failed to start server");
//!
//! println!("Server started on {}:{}", server.address(), server.port());
//!
//! let client = UdpClientBuilder::new(server.address()).connect_to(server.port())
//!                     .expect("Failed to connect client to server");
//!
//! println!("Client connected to {}:{}", client.address(), client.port());
//!
//! // send data from client to server
//! let send_buffer = [1u8, 2u8, 3u8];
//! let bytes_sent = client.send(&send_buffer)
//!                        .expect("failed to send data");
//!
//! // receive data from client
//! let mut recv_buffer = [0u8; 16];
//! let recv_state = server.blocking_receive_from(&mut recv_buffer)
//!                        .expect("failed to receive data").unwrap();
//!
//! // send answer to client
//! let bytes_sent = server.send_to(&send_buffer, recv_state.source_ip, recv_state.source_port)
//!                        .expect("failed to send message back to client");
//!
//! // receive answer on client
//! let bytes_received = client.try_receive(&mut recv_buffer)
//!                            .expect("failed to receive answer");
//! ```

use core::fmt::Debug;
use core::sync::atomic::Ordering;
use core::time::Duration;
use iceoryx2_bb_log::{fail, fatal_panic, trace};
use iceoryx2_bb_system_types::ipv4_address::{self, Ipv4Address};
use iceoryx2_bb_system_types::port::{self, Port};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;
use iceoryx2_pal_posix::posix::{self, MemZeroedStruct};
use iceoryx2_pal_posix::posix::{Errno, SockAddrIn};

use crate::file_descriptor::{FileDescriptor, FileDescriptorBased};
use crate::file_descriptor_set::{
    FileDescriptorSet, FileDescriptorSetWaitError, FileEvent, SynchronousMultiplexing,
};
use crate::handle_errno;

/// Describes errors when creating and [`UdpServer`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UdpServerCreateError {
    InsufficientMemory,
    InsufficientResources,
    InsufficientPermissions,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    UdpProtocolNotSupported,
    InetSocketsNotSupported,
    AddressAlreadyInUse,
    AddressNotAvailable,
    AddressFamilyNotSupported,
    UnknownError(i32),
}

/// Describes errors when creating and [`UdpClient`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UdpClientCreateError {
    InsufficientResources,
    InsufficientPermissions,
    PerProcessFileHandleLimitReached,
    SystemWideFileHandleLimitReached,
    UdpProtocolNotSupported,
    InetSocketsNotSupported,
    AddressNotAvailable,
    ConnectionRefused,
    Interrupt,
    NoRouteToHost,
    ConnectionTimeout,
    HostUnreachable,
    NetworkInterfaceDown,
    AddressFamilyNotSupported,
    UnknownError(i32),
}

/// Describes errors when receiving data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UdpReceiveError {
    ConnectionReset,
    Interrupt,
    NotConnected,
    IOerror,
    InsufficientResources,
    InsufficientMemory,
    UnknownError(i32),
}

/// Describes errors when sending data.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UdpSendError {
    ConnectionReset,
    Interrupt,
    MessageTooLarge,
    HostUnreachable,
    IOerror,
    NetworkInterfaceDown,
    NoRouteToHost,
    InsufficientResources,
    InsufficientMemory,
    UnknownError(i32),
}

fn create_sockaddr(address: Ipv4Address, port: Port) -> posix::sockaddr_in {
    let mut addr = posix::sockaddr_in::new_zeroed();
    addr.sin_family = posix::AF_INET as _;
    addr.set_s_addr(address.as_u32().to_be());
    addr.sin_port = port.as_u16().to_be();
    addr
}

/// Contains the number of bytes received as well as the origin of the data.
#[derive(Debug)]
pub struct ReceiveDetails {
    pub number_of_bytes: usize,
    pub source_ip: Ipv4Address,
    pub source_port: Port,
}

impl ReceiveDetails {
    fn new(number_of_bytes: usize, source: posix::sockaddr_in) -> Self {
        Self {
            number_of_bytes,
            source_ip: unsafe {
                core::mem::transmute::<u32, Ipv4Address>(u32::from_be(source.get_s_addr()))
            },
            source_port: Port::new(u16::from_be(source.sin_port)),
        }
    }
}

/// Builder for the [`UdpClient`]
#[derive(Debug)]
pub struct UdpClientBuilder {
    address: Ipv4Address,
}

impl UdpClientBuilder {
    /// Creates a new [`UdpClientBuilder`]. Requires the address of the [`UdpServer`].
    pub fn new(address: Ipv4Address) -> Self {
        Self { address }
    }

    /// Connects to a given port of the [`UdpServer`].
    pub fn connect_to(self, port: Port) -> Result<UdpClient, UdpClientCreateError> {
        let raw_fd = unsafe {
            posix::socket(
                posix::PF_INET as posix::int,
                posix::SOCK_DGRAM,
                posix::IPPROTO_UDP,
            )
        };

        let msg = "Unable to create UdpClient socket";
        if raw_fd < 0 {
            handle_errno!(UdpClientCreateError, from self,
                Errno::EAFNOSUPPORT => (AddressFamilyNotSupported, "{} since the address family is not supported by the system.", msg),
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process limit of file descriptors was reached.", msg),
                Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since system-wide limit of file descriptors was reached.", msg),
                Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
                Errno::EPROTOTYPE => (InetSocketsNotSupported, "{} since PF_INET socket type is not supported.", msg),
                Errno::EPROTONOSUPPORT => (UdpProtocolNotSupported, "{} since the udp protocol is not supported by the system.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        let server_address = create_sockaddr(self.address, port);

        let msg = "Unable to connect UdpClient socket";
        if unsafe {
            posix::connect(
                raw_fd,
                (&server_address as *const posix::sockaddr_in) as *const posix::sockaddr,
                core::mem::size_of::<posix::sockaddr_in>() as u32,
            )
        } == -1
        {
            handle_errno!(UdpClientCreateError, from self,
                Errno::EAFNOSUPPORT => (AddressFamilyNotSupported, "{} since the address family is not supported by the system.", msg),
                Errno::EADDRNOTAVAIL => (AddressNotAvailable, "{} since the address is not available.", msg),
                Errno::ECONNREFUSED => (ConnectionRefused, "{} since the connection was refused.", msg),
                Errno::EINTR => (Interrupt, "{} due to an interrupt signal.", msg),
                Errno::ENETUNREACH => (NoRouteToHost, "{} since there is no route to the host.", msg),
                Errno::ETIMEDOUT => (ConnectionTimeout, "{} since timed out.", msg),
                Errno::ENETDOWN => (NetworkInterfaceDown, "{} since the required network interface is down.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        Ok(UdpClient::new(
            unsafe { FileDescriptor::new_unchecked(raw_fd) },
            server_address,
        ))
    }
}

/// Abstraction of an UDP client that can communicate with one specific network server.
#[derive(Debug)]
pub struct UdpClient {
    socket: UdpSocket,
}

impl Drop for UdpClient {
    fn drop(&mut self) {
        trace!(from self, "disconnected");
    }
}

impl UdpClient {
    fn new(socket_fd: FileDescriptor, server: posix::sockaddr_in) -> Self {
        let new_self = Self {
            socket: UdpSocket::new(socket_fd, server),
        };
        trace!(from new_self, "connected");
        new_self
    }

    /// Returns the [`Ipv4Address`] of the corresponding UDP server
    pub fn address(&self) -> Ipv4Address {
        self.socket.address()
    }

    /// Returns the [`Port`] of the corresponding UDP server
    pub fn port(&self) -> Port {
        self.socket.port()
    }

    /// Sends a message to the corresponding UDP server. Returns the amount of bytes sent.
    pub fn send(&self, data: &[u8]) -> Result<usize, UdpSendError> {
        self.socket.send(data)
    }

    /// Tries to receive a message from the corresponding UDP server. If no message was received
    /// the method returns 0 otherwise the number of bytes received.
    pub fn try_receive(&self, buffer: &mut [u8]) -> Result<usize, UdpReceiveError> {
        fail!(from self, when self.socket.set_non_blocking(true),
            "Unable to try receive on socket since the socket could not activate the non-blocking mode.");

        self.socket.receive(buffer)
    }

    /// Blocks until either a message from the server was received or the timeout has passed. If no
    /// message was received the method returns 0 otherwise the number of bytes received.
    pub fn timed_receive(
        &self,
        buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<usize, UdpReceiveError> {
        let msg = "Failed to timed receive";

        fail!(from self, when self.socket.set_non_blocking(false),
            "{} since the socket could not activate the blocking mode.", msg);

        let fd_set = FileDescriptorSet::new();
        let _guard = fatal_panic!(from self, when fd_set.add(&self.socket),
                            "This should never happen! {} since the socket could not be attached to a fd set.", msg);

        let mut received_bytes = Ok(0);
        let receive_call = |_: &FileDescriptor| {
            received_bytes = self.socket.receive(buffer);
        };

        match fd_set.timed_wait(timeout, FileEvent::Read, receive_call) {
            Err(FileDescriptorSetWaitError::Interrupt) => {
                fail!(from self, with UdpReceiveError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(_) => {
                fail!(from self, with UdpReceiveError::UnknownError(-1),
                    "{} since an unknown failure occurred.", msg);
            }
            Ok(_) => received_bytes,
        }
    }

    /// Blocks until a message from the server was received. Returns the number of bytes received.
    pub fn blocking_receive(&self, buffer: &mut [u8]) -> Result<usize, UdpReceiveError> {
        fail!(from self, when self.socket.set_non_blocking(false),
            "Unable to blocking receive on socket since the socket could not activate the blocking mode.");

        self.socket.receive(buffer)
    }
}

/// Builder for the [`UdpServer`].
#[derive(Debug)]
pub struct UdpServerBuilder {
    address: Ipv4Address,
    port: Port,
}

impl Default for UdpServerBuilder {
    fn default() -> Self {
        Self {
            address: ipv4_address::UNSPECIFIED,
            port: port::UNSPECIFIED,
        }
    }
}

impl UdpServerBuilder {
    /// Creates a new [`UdpServerBuilder`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Can be set optionally. If no address is set the [`UdpServer`] listens on all available
    /// addresses.
    pub fn address(mut self, address: Ipv4Address) -> Self {
        self.address = address;
        self
    }

    /// Can be set optionally. If no port is given the operating system will choose a free port on
    /// which the [`UdpServer`] will listen.
    pub fn port(mut self, port: Port) -> Self {
        self.port = port;
        self
    }

    /// Creates a socket that listens on the specified address/port.
    pub fn listen(self) -> Result<UdpServer, UdpServerCreateError> {
        let raw_fd = unsafe {
            posix::socket(
                posix::PF_INET as posix::int,
                posix::SOCK_DGRAM,
                posix::IPPROTO_UDP,
            )
        };

        let msg = "Unable to create UdpServer socket";
        if raw_fd < 0 {
            handle_errno!(UdpServerCreateError, from self,
                Errno::EAFNOSUPPORT => (AddressFamilyNotSupported, "{} since the address family is not supported by the system.", msg),
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::EMFILE => (PerProcessFileHandleLimitReached, "{} since the per-process limit of file descriptors was reached.", msg),
                Errno::ENFILE => (SystemWideFileHandleLimitReached, "{} since system-wide limit of file descriptors was reached.", msg),
                Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
                Errno::EPROTOTYPE => (InetSocketsNotSupported, "{} since PF_INET socket type is not supported.", msg),
                Errno::EPROTONOSUPPORT => (UdpProtocolNotSupported, "{} since the udp protocol is not supported by the system.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        let server_address = create_sockaddr(self.address, self.port);

        let msg = "Unable to create and bind UdpServer socket";
        if unsafe {
            posix::bind(
                raw_fd,
                (&server_address as *const posix::sockaddr_in) as *const posix::sockaddr,
                core::mem::size_of::<posix::sockaddr_in>() as u32,
            ) == -1
        } {
            handle_errno!(UdpServerCreateError, from self,
                Errno::EAFNOSUPPORT => (AddressFamilyNotSupported, "{} since the address family is not supported by the system.", msg),
                Errno::EACCES => (InsufficientPermissions, "{} due to insufficient permissions.", msg),
                Errno::EADDRINUSE => (AddressAlreadyInUse, "{} since the address is already in use.", msg),
                Errno::EADDRNOTAVAIL => (AddressNotAvailable, "{} since the address is not available.", msg),
                Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        let mut client_address = posix::sockaddr_in::new_zeroed();
        let mut client_len = core::mem::size_of::<posix::sockaddr_in>() as posix::socklen_t;

        let msg = "Unable to read newly created UdpServer socket details";
        if unsafe {
            posix::getsockname(
                raw_fd,
                (&mut client_address as *mut posix::sockaddr_in) as *mut posix::sockaddr,
                &mut client_len,
            )
        } == -1
        {
            handle_errno!(UdpServerCreateError, from self,
                v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
            );
        }

        Ok(UdpServer::new(
            unsafe { FileDescriptor::new_unchecked(raw_fd) },
            client_address,
        ))
    }
}

/// Abstraction for an UDP network server.
#[derive(Debug)]
pub struct UdpServer {
    socket: UdpSocket,
}

impl Drop for UdpServer {
    fn drop(&mut self) {
        trace!(from self, "stop listen");
    }
}

impl UdpServer {
    fn new(socket_fd: FileDescriptor, server: posix::sockaddr_in) -> Self {
        let new_self = Self {
            socket: UdpSocket::new(socket_fd, server),
        };
        trace!(from new_self, "listen");
        new_self
    }

    /// Returns the [`Ipv4Address`] of the [`UdpServer`]
    pub fn address(&self) -> Ipv4Address {
        self.socket.address()
    }

    /// Returns the [`Port`] of the [`UdpServer`]
    pub fn port(&self) -> Port {
        self.socket.port()
    }

    /// Sends data to a specific [`UdpClient`]. Returns the number of bytes sent.
    pub fn send_to(
        &self,
        buffer: &[u8],
        address: Ipv4Address,
        port: Port,
    ) -> Result<usize, UdpSendError> {
        self.socket.send_to(buffer, address, port)
    }

    /// Tries to receive a message from any UDP client. If no message was received
    /// the method returns [`None`] otherwise [`ReceiveDetails`] that contain the number of bytes
    /// received as well as the origin of the data.
    pub fn try_receive_from(
        &self,
        buffer: &mut [u8],
    ) -> Result<Option<ReceiveDetails>, UdpReceiveError> {
        fail!(from self, when self.socket.set_non_blocking(true),
            "Unable to try receive from socket since the socket could not activate the non-blocking mode.");

        self.socket.receive_from(buffer)
    }

    /// Blocks until either a message was received or the timeout has passed. If no message was received
    /// the method returns [`None`] otherwise [`ReceiveDetails`] that contain the number of bytes
    /// received as well as the origin of the data.
    pub fn timed_receive_from(
        &self,
        buffer: &mut [u8],
        timeout: Duration,
    ) -> Result<Option<ReceiveDetails>, UdpReceiveError> {
        let msg = "Failed to timed receive from";
        fail!(from self, when self.socket.set_non_blocking(false),
            "{} since the socket could not activate the blocking mode.", msg);

        let fd_set = FileDescriptorSet::new();
        let _guard = fatal_panic!(from self, when fd_set.add(&self.socket),
                            "This should never happen! {} since the socket could not be attached to a fd set.", msg);

        let mut received_bytes = Ok(None);
        match fd_set.timed_wait(timeout, FileEvent::Read, |_| {
            received_bytes = self.socket.receive_from(buffer)
        }) {
            Err(FileDescriptorSetWaitError::Interrupt) => {
                fail!(from self, with UdpReceiveError::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(_) => {
                fail!(from self, with UdpReceiveError::UnknownError(-1),
                    "{} since an unknown failure occurred.", msg);
            }
            Ok(_) => received_bytes,
        }
    }

    /// Blocks until either a message was received. If no message was received
    /// the method returns [`None`] otherwise [`ReceiveDetails`] that contain the number of bytes
    /// received as well as the origin of the data.
    pub fn blocking_receive_from(
        &self,
        buffer: &mut [u8],
    ) -> Result<Option<ReceiveDetails>, UdpReceiveError> {
        fail!(from self, when self.socket.set_non_blocking(false),
            "Unable to blocking receive from socket since the socket could not activate the blocking mode.");

        self.socket.receive_from(buffer)
    }
}

struct UdpSocket {
    socket_fd: FileDescriptor,
    details: posix::sockaddr_in,
    is_non_blocking: IoxAtomicBool,
}

impl Debug for UdpSocket {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "UdpSocket {{ socket_fd: {:?}, details: posix::sockaddr_in {{ sin_addr: {}, sin_family: {}, sin_port: {} }}, is_non_blocking: {:?} }}",
            self.socket_fd,
            self.details.get_s_addr(),
            self.details.sin_family,
            self.details.sin_port,
            self.is_non_blocking.load(Ordering::Relaxed)
        )
    }
}

impl FileDescriptorBased for UdpSocket {
    fn file_descriptor(&self) -> &FileDescriptor {
        &self.socket_fd
    }
}

impl SynchronousMultiplexing for UdpSocket {}

impl UdpSocket {
    fn new(socket_fd: FileDescriptor, details: posix::sockaddr_in) -> Self {
        Self {
            socket_fd,
            details,
            is_non_blocking: IoxAtomicBool::new(false),
        }
    }

    fn address(&self) -> Ipv4Address {
        unsafe { core::mem::transmute::<u32, Ipv4Address>(u32::from_be(self.details.get_s_addr())) }
    }

    pub fn port(&self) -> Port {
        Port::new(u16::from_be(self.details.sin_port))
    }

    fn fcntl(&self, command: i32, value: i32, msg: &str) -> Result<i32, UdpReceiveError> {
        let result = unsafe { posix::fcntl_int(self.socket_fd.native_handle(), command, value) };

        if result >= 0 {
            return Ok(result);
        }

        handle_errno!(UdpReceiveError, from self,
            fatal Errno::EBADF => ("This should never happen! {} since the file descriptor is invalid.", msg);
            fatal Errno::EINVAL => ("This should never happen! {} since an internal argument was invalid.", msg),
            Errno::EINTR => (Interrupt, "{} due to an interrupt signal.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn set_non_blocking(&self, value: bool) -> Result<(), UdpReceiveError> {
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

    fn receive_from(&self, buffer: &mut [u8]) -> Result<Option<ReceiveDetails>, UdpReceiveError> {
        let mut client = posix::sockaddr_in::new_zeroed();
        let mut client_len = core::mem::size_of::<posix::sockaddr_in>() as u32;
        let bytes_received = unsafe {
            posix::recvfrom(
                self.socket_fd.native_handle(),
                buffer.as_mut_ptr() as *mut posix::void,
                buffer.len(),
                0,
                (&mut client as *mut posix::sockaddr_in) as *mut posix::sockaddr,
                &mut client_len,
            )
        };

        if bytes_received >= 0 {
            return Ok(Some(ReceiveDetails::new(bytes_received as usize, client)));
        }

        let msg = "Unable to receive data";
        handle_errno!(UdpReceiveError, from self,
            success Errno::EAGAIN => None,
            Errno::ECONNRESET => (ConnectionReset, "{} since connection was forcibly closed.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::ENOTCONN => (NotConnected, "{} since the socket is not connected.", msg),
            Errno::EIO => (IOerror, "{} since an I/O error occurred while reading from the file system.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} due to an unknown error({}).", msg, v)
        );
    }

    fn receive(&self, buffer: &mut [u8]) -> Result<usize, UdpReceiveError> {
        let bytes_received = unsafe {
            posix::recv(
                self.socket_fd.native_handle(),
                buffer.as_mut_ptr() as *mut posix::void,
                buffer.len(),
                0,
            )
        };

        if bytes_received >= 0 {
            return Ok(bytes_received as usize);
        }

        let msg = "Unable to receive data";
        handle_errno!(UdpReceiveError, from self,
            success Errno::EAGAIN => 0,
            Errno::ECONNRESET => (ConnectionReset, "{} since connection was forcibly closed.", msg),
            Errno::EINTR => (Interrupt, "{} since an interrupt signal was received.", msg),
            Errno::ENOTCONN => (NotConnected, "{} since the socket is not connected.", msg),
            Errno::EIO => (IOerror, "{} since an I/O error occurred while reading from the file system.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} due to an unknown error({}).", msg, v)
        );
    }

    fn send_to(
        &self,
        data: &[u8],
        address: Ipv4Address,
        port: Port,
    ) -> Result<usize, UdpSendError> {
        let addr = create_sockaddr(address, port);
        let number_of_bytes_sent = unsafe {
            posix::sendto(
                self.socket_fd.native_handle(),
                data.as_ptr() as *const posix::void,
                data.len(),
                0,
                (&addr as *const posix::sockaddr_in) as *mut posix::sockaddr,
                core::mem::size_of::<posix::sockaddr_in>() as u32,
            )
        };

        if number_of_bytes_sent >= 0 {
            return Ok(number_of_bytes_sent as usize);
        }

        let msg = format!("Unable to send message to {address}:{port}");
        handle_errno!(UdpSendError, from self,
            Errno::ECONNRESET => (ConnectionReset, "{} since the connection was reset.", msg),
            Errno::EINTR => (Interrupt, "{} due to an interrupt signal.", msg),
            Errno::EMSGSIZE => (MessageTooLarge, "{} since the message is too large to be sent.", msg),
            Errno::EHOSTUNREACH => (HostUnreachable, "{} since the host is unreachable.", msg),
            Errno::EIO => (IOerror, "{} due to an IO failure.", msg),
            Errno::ENETDOWN => (NetworkInterfaceDown, "{} since the required network interface is down.", msg),
            Errno::ENETUNREACH => (NoRouteToHost, "{} since there is no route to the specified host.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }

    fn send(&self, data: &[u8]) -> Result<usize, UdpSendError> {
        let number_of_bytes_sent = unsafe {
            posix::send(
                self.socket_fd.native_handle(),
                data.as_ptr() as *const posix::void,
                data.len(),
                0,
            )
        };

        if number_of_bytes_sent >= 0 {
            return Ok(number_of_bytes_sent as usize);
        }

        let msg = "Unable to send message";
        handle_errno!(UdpSendError, from self,
            Errno::ECONNRESET => (ConnectionReset, "{} since the connection was reset.", msg),
            Errno::EINTR => (Interrupt, "{} due to an interrupt signal.", msg),
            Errno::EMSGSIZE => (MessageTooLarge, "{} since the message is too large to be sent.", msg),
            Errno::EHOSTUNREACH => (HostUnreachable, "{} since the host is unreachable.", msg),
            Errno::EIO => (IOerror, "{} due to an IO failure.", msg),
            Errno::ENETDOWN => (NetworkInterfaceDown, "{} since the required network interface is down.", msg),
            Errno::ENETUNREACH => (NoRouteToHost, "{} since there is no route to the specified host.", msg),
            Errno::ENOBUFS => (InsufficientResources, "{} due to insufficient resources.", msg),
            Errno::ENOMEM => (InsufficientMemory, "{} due to insufficient memory.", msg),
            v => (UnknownError(v as i32), "{} since an unknown error occurred ({}).", msg, v)
        );
    }
}
