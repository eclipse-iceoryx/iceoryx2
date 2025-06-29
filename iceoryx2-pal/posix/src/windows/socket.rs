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

extern crate alloc;
use alloc::ffi::CString;
use core::cell::OnceCell;
use core::sync::atomic::Ordering;
use core::time::Duration;
use iceoryx2_pal_concurrency_sync::iox_atomic::{IoxAtomicU64, IoxAtomicU8};
use std::time::Instant;
use windows_sys::Win32::Networking::WinSock::{INVALID_SOCKET, SOCKADDR, SOCKET_ERROR, WSADATA};
use windows_sys::Win32::Networking::WinSock::{SOCKADDR_UN, WSAEWOULDBLOCK};

use crate::posix::getpid;
use crate::posix::select;
use crate::posix::types::*;
use crate::posix::SockAddrIn;
use crate::posix::{constants::*, fcntl_int};
use crate::posix::{Errno, MemZeroedStruct};

use crate::win32call;

use super::win32_handle_translator::UdsDatagramSocketHandle;
use super::win32_handle_translator::{FdHandleEntry, HandleTranslator, SocketHandle};
use super::{close, remove};

struct GlobalWsaInitializer {
    _wsa_data: WSADATA,
}

impl MemZeroedStruct for WSADATA {}

impl GlobalWsaInitializer {
    unsafe fn init() {
        static mut WSA_INSTANCE: OnceCell<GlobalWsaInitializer> = OnceCell::new();
        static mut INITIALIZATION_STATE: IoxAtomicU8 = IoxAtomicU8::new(0);

        #[allow(static_mut_refs)] // only written here once when it is not initialized
        match INITIALIZATION_STATE.compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => {
                WSA_INSTANCE.get_or_init(||{
                    let mut _wsa_data = WSADATA::new_zeroed();
                    win32call! {winsock windows_sys::Win32::Networking::WinSock::WSAStartup(2, &mut _wsa_data)};
                    GlobalWsaInitializer { _wsa_data }
                });
                INITIALIZATION_STATE.store(2, Ordering::Relaxed);
            }
            Err(1) => while INITIALIZATION_STATE.load(Ordering::Relaxed) == 1 {},
            Err(_) => (),
        }
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

pub unsafe fn socketpair(
    domain: int,
    socket_type: int,
    protocol: int,
    socket_vector: *mut int, // actually it shall be [int; 2]
) -> int {
    static COUNTER: IoxAtomicU64 = IoxAtomicU64::new(0);
    let pid = getpid();
    let socket_listen = socket(domain, socket_type, protocol);
    if socket_listen == -1 {
        return -1;
    }
    let socket_data_1 = socket(domain, socket_type, protocol);
    if socket_data_1 == -1 {
        close(socket_listen);
        return -1;
    }

    let mut address = SOCKADDR_UN {
        sun_family: domain as _,
        sun_path: [0; 108],
    };

    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let socket_path = CString::new(format!("uds_stream_socket_{pid}_{counter}")).unwrap();
    core::ptr::copy_nonoverlapping(
        socket_path.as_ptr(),
        address.sun_path.as_mut_ptr().cast(),
        socket_path.to_bytes().len(),
    );

    if bind(
        socket_listen,
        (&address as *const SOCKADDR_UN).cast(),
        core::mem::size_of::<SOCKADDR_UN>() as _,
    ) == -1
    {
        close(socket_listen);
        close(socket_data_1);
        return -1;
    }

    if listen(socket_listen, 20) == -1 {
        close(socket_listen);
        close(socket_data_1);
        remove(socket_path.as_ptr().cast());
        return -1;
    }

    if connect(
        socket_data_1,
        (&address as *const SOCKADDR_UN).cast(),
        core::mem::size_of::<SOCKADDR_UN>() as _,
    ) == -1
    {
        close(socket_listen);
        close(socket_data_1);
        remove(socket_path.as_ptr().cast());
        return -1;
    }

    let socket_data_2 = accept(socket_listen, core::ptr::null_mut(), core::ptr::null_mut());
    if socket_data_2 == -1 {
        close(socket_listen);
        close(socket_data_1);
        remove(socket_path.as_ptr().cast());
        return -1;
    }

    close(socket_listen);
    socket_vector.write(socket_data_1);
    socket_vector.add(1).write(socket_data_2);
    remove(socket_path.as_ptr().cast());

    0
}

pub unsafe fn setsockopt(
    socket: int,
    mut level: int,
    option_name: int,
    option_value: *const void,
    option_len: socklen_t,
) -> int {
    let socket_handle = match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(mut s)) => {
            if option_name == SO_RCVTIMEO || option_name == SO_SNDTIMEO {
                fcntl_int(socket, F_SETFL, O_NONBLOCK);
                if option_name == SO_RCVTIMEO {
                    s.recv_timeout = Some(*(option_value as *const timeval));
                } else if option_name == SO_SNDTIMEO {
                    s.send_timeout = Some(*(option_value as *const timeval));
                }
                HandleTranslator::get_instance().update(FdHandleEntry::Socket(s));
                return 0;
            }
            s.fd
        }
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

    let (sock_opt_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::setsockopt(socket_handle, level, option_name, option_value as *const u8, option_len as _)};
    if sock_opt_result == SOCKET_ERROR {
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

    let (sock_opt_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::getsockopt(socket_handle, level, option_name, option_value as *mut u8, option_len as *mut i32)};
    if sock_opt_result == SOCKET_ERROR {
        return -1;
    }

    0
}

unsafe fn create_uds_address(port: u16) -> sockaddr_in {
    let mut udp_address = sockaddr_in::new_zeroed();
    udp_address.sin_family = AF_INET as _;
    let localhost: u32 = (127 << 24) | 1;
    udp_address.set_s_addr(localhost.to_be());
    udp_address.sin_port = port.to_be();
    udp_address
}

pub unsafe fn accept(socket: int, address: *mut sockaddr, address_len: *mut socklen_t) -> int {
    let socket_handle = match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(s)) => s.fd,
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    let (socket, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::accept(socket_handle, address.cast(), address_len.cast())};
    if socket == INVALID_SOCKET {
        return -1;
    }

    HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle {
        fd: socket,
        recv_timeout: None,
        send_timeout: None,
    }))
}

pub unsafe fn listen(socket: int, backlog: int) -> int {
    let socket_handle = match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(s)) => s.fd,
        None | Some(_) => {
            Errno::set(Errno::EBADF);
            return -1;
        }
    };

    let (listen_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::listen(socket_handle, backlog as _)};
    if listen_result == SOCKET_ERROR {
        return -1;
    }

    0
}

pub unsafe fn bind(socket: int, address: *const sockaddr, address_len: socklen_t) -> int {
    match HandleTranslator::get_instance().get(socket) {
        Some(FdHandleEntry::Socket(s)) => {
            let (bind_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::bind(s.fd, address as *const SOCKADDR, address_len as _)};
            if bind_result == SOCKET_ERROR {
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

            let (bind_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::bind(s.fd, (&udp_address as *const sockaddr_in) as *const SOCKADDR, core::mem::size_of::<sockaddr_in>() as _)};
            if bind_result == SOCKET_ERROR {
                return -1;
            }

            let mut client_address = sockaddr_in::new_zeroed();
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

            let port = u16::from_be(client_address.sin_port);
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
            let (connect_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::connect(s.fd, address as *const SOCKADDR, address_len as _)};
            if connect_result == SOCKET_ERROR {
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

            let (connect_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::connect(s.fd, (&udp_address as *const sockaddr_in) as *const SOCKADDR, core::mem::size_of::<sockaddr_in>() as _)};
            if connect_result == SOCKET_ERROR {
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
        let (socket, _) = win32call! { winsock windows_sys::Win32::Networking::WinSock::socket(PF_INET as _, SOCK_DGRAM, protocol)};

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
        let (socket, _) = win32call! { winsock windows_sys::Win32::Networking::WinSock::socket(domain, socket_type, protocol)};

        if socket == INVALID_SOCKET {
            return -1;
        }

        HandleTranslator::get_instance().add(FdHandleEntry::Socket(SocketHandle {
            fd: socket,
            recv_timeout: None,
            send_timeout: None,
        }))
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
            let (bytes_sent, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::sendto(s.fd, message as *const u8, length as _, flags, dest_addr as *const SOCKADDR, dest_len as _) };

            if bytes_sent == SOCKET_ERROR {
                return -1;
            }
            bytes_sent as _
        }
        Some(FdHandleEntry::UdsDatagramSocket(s)) => {
            let (bytes_sent, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::send(s.fd, message as *const u8, length as _, flags)};

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
            let (bytes_received, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::recvfrom(s.fd, buffer as *mut u8, length as _, flags, address as *mut SOCKADDR, address_len as _) };

            if bytes_received == SOCKET_ERROR {
                return -1;
            }
            bytes_received as _
        }
        Some(FdHandleEntry::UdsDatagramSocket(s)) => {
            if let Some(mut timeout) = s.recv_timeout {
                let mut remaining_time = Duration::from_secs(timeout.tv_sec as _)
                    + Duration::from_micros(timeout.tv_usec as _);
                let now = Instant::now();

                loop {
                    let mut read_set = fd_set::new_zeroed();
                    read_set.fd_count = 1;
                    read_set.fd_array[0] = s.fd;

                    let (number_of_triggered_fds, _) = win32call! {select(
                        (s.fd + 1) as _,
                        &mut read_set,
                        core::ptr::null_mut::<fd_set>(),
                        core::ptr::null_mut::<fd_set>(),
                        &mut timeout,
                    ) };

                    if number_of_triggered_fds == SOCKET_ERROR {
                        Errno::set(Errno::EINVAL);
                        return -1;
                    }

                    let elapsed_time = now.elapsed();
                    if remaining_time < elapsed_time {
                        return 0;
                    }

                    if 0 < number_of_triggered_fds {
                        let (received_bytes, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::recv(s.fd, buffer as *mut u8, length as _, flags), ignore WSAEWOULDBLOCK };
                        if 0 < received_bytes {
                            return received_bytes as _;
                        }
                    }

                    remaining_time -= elapsed_time;
                }
            } else {
                let (bytes_received, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::recv(s.fd, buffer as *mut u8, length as _, flags), ignore WSAEWOULDBLOCK };

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

    let (sock_name_result, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::getsockname(socket_fd, address as *mut SOCKADDR, address_len as _) };
    if sock_name_result == SOCKET_ERROR {
        return -1;
    }
    0
}

pub unsafe fn send(socket: int, message: *const void, length: size_t, flags: int) -> ssize_t {
    match HandleTranslator::get_instance().get_socket(socket) {
        Some(s) => {
            if let Some(mut timeout) = s.send_timeout {
                let mut remaining_time = Duration::from_secs(timeout.tv_sec as _)
                    + Duration::from_micros(timeout.tv_usec as _);
                let now = Instant::now();

                loop {
                    let mut write_set = fd_set::new_zeroed();
                    write_set.fd_count = 1;
                    write_set.fd_array[0] = s.fd;

                    let (number_of_triggered_fds, _) = win32call! {select((s.fd + 1) as _, core::ptr::null_mut::<fd_set>(), &mut write_set, core::ptr::null_mut::<fd_set>(), &mut timeout)};

                    if number_of_triggered_fds == SOCKET_ERROR {
                        Errno::set(Errno::EINVAL);
                        return -1;
                    }

                    let elapsed_time = now.elapsed();
                    if remaining_time < elapsed_time {
                        return 0;
                    }

                    if 0 < number_of_triggered_fds {
                        let (sent_bytes, _) = win32call! { winsock windows_sys::Win32::Networking::WinSock::send(s.fd, message as *const u8, length as _, flags), ignore WSAEWOULDBLOCK};
                        if 0 < sent_bytes {
                            return sent_bytes as _;
                        }
                    }

                    remaining_time -= elapsed_time;
                }
            } else {
                let (bytes_sent, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::send(s.fd, message as *const u8, length as _, flags), ignore WSAEWOULDBLOCK};
                if bytes_sent == SOCKET_ERROR {
                    return -1;
                }
                bytes_sent as _
            }
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
            if let Some(mut timeout) = s.recv_timeout {
                let mut remaining_time = Duration::from_secs(timeout.tv_sec as _)
                    + Duration::from_micros(timeout.tv_usec as _);
                let now = Instant::now();

                loop {
                    let mut read_set = fd_set::new_zeroed();
                    read_set.fd_count = 1;
                    read_set.fd_array[0] = s.fd;

                    let (number_of_triggered_fds, _) = win32call! {select((s.fd + 1) as _, &mut read_set, core::ptr::null_mut::<fd_set>(), core::ptr::null_mut::<fd_set>(), &mut timeout)};

                    if number_of_triggered_fds == SOCKET_ERROR {
                        Errno::set(Errno::EINVAL);
                        return -1;
                    }

                    let elapsed_time = now.elapsed();
                    if remaining_time < elapsed_time {
                        return 0;
                    }

                    if 0 < number_of_triggered_fds {
                        let (received_bytes, _) = win32call! { winsock windows_sys::Win32::Networking::WinSock::recv(s.fd, buffer as *mut u8, length as _, flags), ignore WSAEWOULDBLOCK};
                        if 0 < received_bytes {
                            return received_bytes as _;
                        }
                    }

                    remaining_time -= elapsed_time;
                }
            } else {
                let (bytes_received, _) = win32call! {winsock windows_sys::Win32::Networking::WinSock::recv(s.fd, buffer as *mut u8, length as _, flags), ignore WSAEWOULDBLOCK };
                if bytes_received == SOCKET_ERROR {
                    return -1;
                }
                bytes_received as _
            }
        }
        None => {
            Errno::set(Errno::EBADF);
            -1
        }
    }
}
