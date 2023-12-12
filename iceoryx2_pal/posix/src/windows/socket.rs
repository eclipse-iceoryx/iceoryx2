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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use std::cell::OnceCell;

use windows_sys::Win32::Networking::WinSock::WSAEWOULDBLOCK;
use windows_sys::Win32::Networking::WinSock::{INVALID_SOCKET, SOCKADDR, SOCKET_ERROR, WSADATA};

use crate::posix::htons;
use crate::posix::ntohs;
use crate::posix::types::*;
use crate::posix::SockAddrIn;
use crate::posix::{constants::*, fcntl_int};
use crate::posix::{htonl, select};
use crate::posix::{Errno, Struct};

use crate::win32call;

use super::win32_handle_translator::UdsDatagramSocketHandle;
use super::win32_handle_translator::{FdHandleEntry, HandleTranslator, SocketHandle};

struct GlobalWsaInitializer {
    _wsa_data: WSADATA,
}

impl Struct for WSADATA {}

impl GlobalWsaInitializer {
    unsafe fn init() {
        static mut WSA_INSTANCE: OnceCell<GlobalWsaInitializer> = OnceCell::new();

        WSA_INSTANCE.get_or_init(||{
            let mut _wsa_data = WSADATA::new();
            win32call! {winsock windows_sys::Win32::Networking::WinSock::WSAStartup(2, &mut _wsa_data)};
            GlobalWsaInitializer { _wsa_data }
        });
    }
}

impl Drop for GlobalWsaInitializer {
    fn drop(&mut self) {
        unsafe {
            win32call! {winsock windows_sys::Win32::Networking::WinSock::WSACleanup()}
        };
    }
}

unsafe impl Sync for GlobalWsaInitializer {}
unsafe impl Send for GlobalWsaInitializer {}

pub unsafe fn setsockopt(
    socket: int,
    mut level: int,
    option_name: int,
    option_value: *const void,
    option_len: socklen_t,
) -> int {
    let socket_handle = match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(mut s)) => {
            level = SOL_SOCKET;
            if option_name == SO_SNDTIMEO {
                return 0;
            }
            if option_name == SO_RCVTIMEO {
                fcntl_int(socket, F_SETFL, O_NONBLOCK);
                s.recv_timeout = Some(*(option_value as *const timeval));
                HandleTranslator::get_instance().update(FdHandleEntry::UdsDatagramSocket(s));
                return 0;
            }
            s.fd
        }
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    if win32call! {winsock windows_sys::Win32::Networking::WinSock::setsockopt(socket_handle, level, option_name, option_value as *const u8, option_len as _)}
        == SOCKET_ERROR
    {
        return -1;
    }
    0
}

pub unsafe fn getsockopt(
    socket: int,
    level: int,
    option_name: int,
    option_value: *mut void,
    option_len: *mut socklen_t,
) -> int {
    let socket_handle = match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(s)) => s.fd,
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    if win32call! {winsock windows_sys::Win32::Networking::WinSock::getsockopt(socket_handle, level, option_name, option_value as *mut u8, option_len as *mut i32)}
        == SOCKET_ERROR
    {
        return -1;
    }

    0
}

unsafe fn create_uds_address(port: u16) -> sockaddr_in {
    let mut udp_address = sockaddr_in::new();
    udp_address.sin_family = AF_INET as _;
    let localhost: u32 = 127 << 24 | 1;
    udp_address.set_s_addr(htonl(localhost));
    udp_address.sin_port = htons(port);
    udp_address
}

pub unsafe fn bind(socket: int, address: *const sockaddr, address_len: socklen_t) -> int {
    match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => {
            if win32call! {winsock windows_sys::Win32::Networking::WinSock::bind(s.fd, address as *const SOCKADDR, address_len as _)}
                == SOCKET_ERROR
            {
                return -1;
            }

            0
        }
        Some(FdHandleEntry::UdsDatagramSocket(s)) => {
            let name = address as *const sockaddr_un;
            if HandleTranslator::get_instance().contains_uds((*name).sun_path.as_ptr()) {
                Errno::set(Errno::EEXIST);
                return -1;
            }

            let udp_address = create_uds_address(0);

            if win32call! {winsock windows_sys::Win32::Networking::WinSock::bind(s.fd, (&udp_address as *const sockaddr_in) as *const SOCKADDR, core::mem::size_of::<sockaddr_in>() as _)}
                == SOCKET_ERROR
            {
                return -1;
            }

            let mut client_address = sockaddr_in::new();
            let mut client_len = core::mem::size_of::<sockaddr_in>() as socklen_t;

            if getsockname(
                socket,
                (&mut client_address as *mut sockaddr_in) as *mut sockaddr,
                &mut client_len,
            ) == -1
            {
                Errno::set(Errno::EINVAL);
                return -1;
            }

            let port = ntohs(client_address.sin_port);
            HandleTranslator::get_instance().set_uds_name(socket, client_address, address);

            0
        }
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn connect(socket: int, address: *const sockaddr, address_len: socklen_t) -> int {
    match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => {
            if win32call! {winsock windows_sys::Win32::Networking::WinSock::connect(s.fd, address as *const SOCKADDR, address_len as _)}
                == SOCKET_ERROR
            {
                return -1;
            }
            0
        }
        Some(FdHandleEntry::UdsDatagramSocket(s)) => {
            if !HandleTranslator::get_instance()
                .contains_uds((*(address as *const sockaddr_un)).sun_path.as_ptr().cast())
            {
                Errno::set(Errno::ENOENT);
                return -1;
            }

            let port = HandleTranslator::get_instance().get_uds_port(address);
            let udp_address = create_uds_address(port);

            if win32call! {winsock windows_sys::Win32::Networking::WinSock::connect(s.fd, (&udp_address as *const sockaddr_in) as *const SOCKADDR, core::mem::size_of::<sockaddr_in>() as _)}
                == SOCKET_ERROR
            {
                return -1;
            }
            0
        }
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn socket(domain: int, socket_type: int, protocol: int) -> int {
    GlobalWsaInitializer::init();

    if domain == PF_UNIX as _ && socket_type == SOCK_DGRAM {
        let socket = win32call! { winsock windows_sys::Win32::Networking::WinSock::socket(PF_INET as _, SOCK_DGRAM, protocol)};

        if socket == INVALID_SOCKET {
            return -1;
        }

        HandleTranslator::get_instance().add(FdHandleEntry::UdsDatagramSocket(
            UdsDatagramSocketHandle {
                fd: socket,
                address: None,
                recv_timeout: None,
            },
        ))
    } else {
        let socket = win32call! { winsock windows_sys::Win32::Networking::WinSock::socket(domain, socket_type, protocol)};

        if socket == INVALID_SOCKET {
            return -1;
        }

        HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle { fd: socket }))
    }
}

pub unsafe fn sendmsg(socket: int, message: *const msghdr, flags: int) -> ssize_t {
    Errno::set(Errno::ENOTSUP);
    -1
}

pub unsafe fn sendto(
    socket: int,
    message: *const void,
    length: size_t,
    flags: int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => {
            let bytes_sent = win32call! {winsock windows_sys::Win32::Networking::WinSock::sendto(s.fd, message as *const u8, length as _, flags, dest_addr as *const SOCKADDR, dest_len as _) };

            if bytes_sent == SOCKET_ERROR {
                return -1;
            }
            bytes_sent as _
        }
        Some(FdHandleEntry::UdsDatagramSocket(s)) => {
            let bytes_sent = win32call! {winsock windows_sys::Win32::Networking::WinSock::send(s.fd, message as *const u8, length as _, flags)};

            if bytes_sent == SOCKET_ERROR {
                return -1;
            }
            bytes_sent as _
        }
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn recvmsg(socket: int, message: *mut msghdr, flags: int) -> ssize_t {
    Errno::set(Errno::ENOTSUP);
    -1
}

pub unsafe fn recvfrom(
    socket: int,
    buffer: *mut void,
    length: size_t,
    flags: int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> ssize_t {
    match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => {
            let bytes_received = win32call! {winsock windows_sys::Win32::Networking::WinSock::recvfrom(s.fd, buffer as *mut u8, length as _, flags, address as *mut SOCKADDR, address_len as _) };

            if bytes_received == SOCKET_ERROR {
                return -1;
            }
            bytes_received as _
        }
        Some(FdHandleEntry::UdsDatagramSocket(s)) => {
            if let Some(mut timeout) = s.recv_timeout {
                let mut read_set = fd_set::new();
                read_set.fd_count = 1;
                read_set.fd_array[0] = s.fd;

                win32call! {select(
                    (s.fd + 1) as _,
                    &mut read_set,
                    core::ptr::null_mut::<fd_set>(),
                    core::ptr::null_mut::<fd_set>(),
                    &mut timeout,
                ) };

                (win32call! {winsock windows_sys::Win32::Networking::WinSock::recv(s.fd, buffer as *mut u8, length as _, flags), ignore WSAEWOULDBLOCK })
                    as _
            } else {
                let bytes_received = win32call! {winsock windows_sys::Win32::Networking::WinSock::recv(s.fd, buffer as *mut u8, length as _, flags), ignore WSAEWOULDBLOCK };

                if bytes_received == SOCKET_ERROR {
                    return -1;
                }
                bytes_received as _
            }
        }
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn getsockname(socket: int, address: *mut sockaddr, address_len: *mut socklen_t) -> int {
    let socket_fd = match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(s)) => s.fd,
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    if win32call! {winsock windows_sys::Win32::Networking::WinSock::getsockname(socket_fd, address as *mut SOCKADDR, address_len as _) }
        == SOCKET_ERROR
    {
        return -1;
    }
    0
}

pub unsafe fn send(socket: int, message: *const void, length: size_t, flags: int) -> ssize_t {
    match HandleTranslator::get_instance().get_socket(socket) {
        Some(s) => {
            let bytes_sent = win32call! {winsock windows_sys::Win32::Networking::WinSock::send(s.fd, message as *const u8, length as _, flags)};
            if bytes_sent == SOCKET_ERROR {
                return -1;
            }
            bytes_sent as _
        }
        None => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}

pub unsafe fn recv(socket: int, buffer: *mut void, length: size_t, flags: int) -> ssize_t {
    match HandleTranslator::get_instance().get_socket(socket) {
        Some(s) => {
            let bytes_received = win32call! {winsock windows_sys::Win32::Networking::WinSock::recv(s.fd, buffer as *mut u8, length as _, flags) };
            if bytes_received == SOCKET_ERROR {
                return -1;
            }
            bytes_received as _
        }
        None => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}
