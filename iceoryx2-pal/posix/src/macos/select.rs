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

use crate::posix::types::*;
use crate::posix::MemZeroedStruct;

pub unsafe fn select(
    nfds: int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    errorfds: *mut fd_set,
    timeout: *mut timeval,
) -> int {
    crate::internal::select(nfds, readfds, writefds, errorfds, timeout)
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
    if (*sub_header).cmsg_len < core::mem::size_of::<cmsghdr>() as _ {
        return core::ptr::null_mut::<cmsghdr>();
    };

    let next_sub_header =
        (sub_header as usize + CMSG_ALIGN((*sub_header).cmsg_len as _)) as *mut cmsghdr;
    let end_of_message = (*header).msg_control as usize + (*header).msg_controllen as usize;

    if (next_sub_header.offset(1)) as usize > end_of_message {
        return core::ptr::null_mut::<cmsghdr>();
    }

    if next_sub_header as usize + CMSG_ALIGN((*next_sub_header).cmsg_len as _) > end_of_message {
        return core::ptr::null_mut::<cmsghdr>();
    }

    next_sub_header as *mut cmsghdr
}

pub const unsafe fn CMSG_LEN(length: uint) -> uint {
    CMSG_ALIGN(core::mem::size_of::<cmsghdr>()) as uint + length
}

pub unsafe fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut uchar {
    (cmsg as usize + CMSG_ALIGN(core::mem::size_of::<cmsghdr>())) as *mut uchar
}

pub unsafe fn FD_CLR(fd: int, set: *mut fd_set) {
    let fd = fd as usize;
    const BITS: usize = core::mem::size_of::<i32>() * 8;
    (*set).fds_bits[fd / BITS] &= !(1i32 << (fd % BITS));
}

pub unsafe fn FD_ISSET(fd: int, set: *const fd_set) -> bool {
    let fd = fd as usize;
    const BITS: usize = core::mem::size_of::<i32>() * 8;
    (*set).fds_bits[fd / BITS] & (1i32 << (fd % BITS)) != 0
}

pub unsafe fn FD_SET(fd: int, set: *mut fd_set) {
    let fd = fd as usize;
    const BITS: usize = core::mem::size_of::<i32>() * 8;
    (*set).fds_bits[fd / BITS] |= 1i32 << (fd % BITS);
}

pub unsafe fn FD_ZERO(set: *mut fd_set) {
    (*set) = fd_set::new_zeroed();
}
