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

pub unsafe fn select(
    nfds: int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    errorfds: *mut fd_set,
    timeout: *mut timeval,
) -> int {
    libc::select(nfds, readfds, writefds, errorfds, timeout)
}

pub unsafe fn CMSG_SPACE(length: size_t) -> size_t {
    libc::CMSG_SPACE(length as _) as _
}

pub unsafe fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    libc::CMSG_FIRSTHDR(mhdr)
}

pub unsafe fn CMSG_NXTHDR(header: *const msghdr, sub_header: *const cmsghdr) -> *mut cmsghdr {
    libc::CMSG_NXTHDR(header as *mut msghdr, sub_header as *mut cmsghdr)
}

pub unsafe fn CMSG_LEN(length: size_t) -> size_t {
    libc::CMSG_LEN(length as _) as _
}

pub unsafe fn CMSG_DATA(cmsg: *mut cmsghdr) -> *mut uchar {
    libc::CMSG_DATA(cmsg)
}

pub unsafe fn FD_CLR(fd: int, set: *mut fd_set) {
    libc::FD_CLR(fd, set)
}

pub unsafe fn FD_ISSET(fd: int, set: *const fd_set) -> bool {
    libc::FD_ISSET(fd, set)
}

pub unsafe fn FD_SET(fd: int, set: *mut fd_set) {
    libc::FD_SET(fd, set)
}

pub unsafe fn FD_ZERO(set: *mut fd_set) {
    libc::FD_ZERO(set)
}
