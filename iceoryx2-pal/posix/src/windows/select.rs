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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use core::time::Duration;
use std::time::Instant;

use crate::posix::MemZeroedStruct;
use crate::{posix::types::*, win32call};

use super::win32_handle_translator::{FdHandleEntry, HandleTranslator};

pub unsafe fn select(
    nfds: int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    errorfds: *mut fd_set,
    timeout: *mut timeval,
) -> int {
    let now = Instant::now();

    // infinite blocking means on windows 10 years
    let mut timeout = if timeout.is_null() {
        timeval {
            tv_sec: 3600 * 24 * 365,
            tv_usec: 0,
        }
    } else {
        *timeout
    };

    let mut remaining_time =
        Duration::from_secs(timeout.tv_sec as _) + Duration::from_micros(timeout.tv_usec as _);
    let full_timeout = remaining_time;

    let readfds_copy = if readfds.is_null() {
        fd_set::new_zeroed()
    } else {
        *readfds
    };
    let writefds_copy = if writefds.is_null() {
        fd_set::new_zeroed()
    } else {
        *writefds
    };
    let errorfds_copy = if errorfds.is_null() {
        fd_set::new_zeroed()
    } else {
        *errorfds
    };

    loop {
        let (num_handles, _) = win32call! { winsock windows_sys::Win32::Networking::WinSock::select(nfds, readfds, writefds, errorfds, &timeout) };
        if num_handles > 0 {
            return num_handles;
        }

        let elapsed = now.elapsed();
        if elapsed > full_timeout {
            return num_handles;
        } else {
            remaining_time = full_timeout - elapsed;
            timeout.tv_sec = remaining_time.as_secs() as _;
            timeout.tv_usec = remaining_time.subsec_micros() as _;

            if !readfds.is_null() {
                *readfds = readfds_copy;
            }

            if !writefds.is_null() {
                *writefds = writefds_copy;
            }

            if !errorfds.is_null() {
                *errorfds = errorfds_copy;
            }
        }
    }
}

pub const fn CMSG_ALIGN(len: usize) -> usize {
    (len + core::mem::size_of::<usize>() - 1) & !(core::mem::size_of::<usize>() - 1)
}

pub const unsafe fn CMSG_SPACE(length: uint) -> uint {
    (CMSG_ALIGN(length as usize) + CMSG_ALIGN(core::mem::size_of::<cmsghdr>())) as uint
}

pub fn CMSG_SPACE_NON_CONST(length: uint) -> uint {
    (CMSG_ALIGN(length as usize) + CMSG_ALIGN(core::mem::size_of::<cmsghdr>())) as uint
}

pub unsafe fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    match ((*mhdr).msg_controllen as usize) < core::mem::size_of::<cmsghdr>() {
        true => core::ptr::null_mut::<cmsghdr>(),
        false => (*mhdr).msg_control as *mut cmsghdr,
    }
}

pub unsafe fn CMSG_NXTHDR(header: *const msghdr, sub_header: *const cmsghdr) -> *mut cmsghdr {
    // no header contained
    if (*sub_header).cmsg_len < core::mem::size_of::<cmsghdr>() {
        return core::ptr::null_mut::<cmsghdr>();
    };

    let next_sub_header =
        (sub_header as usize + CMSG_ALIGN((*sub_header).cmsg_len)) as *mut cmsghdr;
    let end_of_message = (*header).msg_control as usize + (*header).msg_controllen as usize;

    if (next_sub_header.offset(1)) as usize > end_of_message {
        return core::ptr::null_mut::<cmsghdr>();
    }

    if next_sub_header as usize + CMSG_ALIGN((*next_sub_header).cmsg_len) > end_of_message {
        return core::ptr::null_mut::<cmsghdr>();
    }

    next_sub_header
}

pub const unsafe fn CMSG_LEN(length: uint) -> uint {
    CMSG_ALIGN(core::mem::size_of::<cmsghdr>()) as uint + length
}

pub unsafe fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut uchar {
    cmsg.offset(1) as *mut uchar
}

pub unsafe fn FD_CLR(fd: int, set: *mut fd_set) {
    let socket = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(s)) => s.fd,
        Some(_) | None => return,
    };

    for i in 0..(*set).fd_count {
        if (*set).fd_array[i as usize] == socket {
            if i < (*set).fd_count - 1 {
                (*set).fd_array[i as usize] = (*set).fd_array[(*set).fd_count as usize - 1];
            } else {
                (*set).fd_array[i as usize] = 0;
            }
            (*set).fd_count -= 1;
            return;
        }
    }
}

pub unsafe fn FD_ISSET(fd: int, set: *const fd_set) -> bool {
    let socket = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(s)) => s.fd,
        Some(_) | None => return false,
    };

    for i in 0..(*set).fd_count {
        if (*set).fd_array[i as usize] == socket {
            return true;
        }
    }

    false
}

pub unsafe fn FD_SET(fd: int, set: *mut fd_set) {
    if FD_ISSET(fd, set) {
        return;
    }

    let socket = match HandleTranslator::get_instance().get(fd) {
        Some(FdHandleEntry::Socket(s)) => s.fd,
        Some(FdHandleEntry::UdsDatagramSocket(s)) => s.fd,
        Some(_) | None => return,
    };

    if (*set).fd_count as usize >= (*set).fd_array.len() {
        return;
    }

    (*set).fd_array[(*set).fd_count as usize] = socket;
    (*set).fd_count += 1;
}

pub unsafe fn FD_ZERO(set: *mut fd_set) {
    (*set).fd_count = 0;
    (*set).fd_array = [0; 64];
}
