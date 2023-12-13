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
    crate::internal::select(nfds, readfds, writefds, errorfds, timeout)
}

pub unsafe fn CMSG_SPACE(length: size_t) -> size_t {
    internal::iceoryx2_cmsg_space(length)
}

pub unsafe fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
    internal::iceoryx2_cmsg_firsthdr(mhdr)
}

pub unsafe fn CMSG_NXTHDR(header: *const msghdr, sub_header: *const cmsghdr) -> *mut cmsghdr {
    internal::iceoryx2_cmsg_nxthdr(header as *mut msghdr, sub_header as *mut cmsghdr)
}

pub unsafe fn CMSG_LEN(length: size_t) -> size_t {
    internal::iceoryx2_cmsg_len(length)
}

pub unsafe fn CMSG_DATA(cmsg: *mut cmsghdr) -> *mut uchar {
    internal::iceoryx2_cmsg_data(cmsg)
}

pub unsafe fn FD_CLR(fd: int, set: *mut fd_set) {
    internal::iceoryx2_fd_clr(fd, set)
}

pub unsafe fn FD_ISSET(fd: int, set: *const fd_set) -> bool {
    internal::iceoryx2_fd_isset(fd, set) != 0
}

pub unsafe fn FD_SET(fd: int, set: *mut fd_set) {
    internal::iceoryx2_fd_set(fd, set)
}

pub unsafe fn FD_ZERO(set: *mut fd_set) {
    internal::iceoryx2_fd_zero(set)
}

mod internal {
    use super::*;

    #[cfg_attr(target_os = "freebsd", link(name = "c"))]
    extern "C" {
        pub(super) fn iceoryx2_cmsg_space(len: size_t) -> size_t;
        pub(super) fn iceoryx2_cmsg_firsthdr(hdr: *const msghdr) -> *mut cmsghdr;
        pub(super) fn iceoryx2_cmsg_nxthdr(hdr: *mut msghdr, sub: *mut cmsghdr) -> *mut cmsghdr;
        pub(super) fn iceoryx2_cmsg_len(len: size_t) -> size_t;
        pub(super) fn iceoryx2_cmsg_data(cmsg: *mut cmsghdr) -> *mut uchar;
        pub(super) fn iceoryx2_fd_clr(fd: int, set: *mut fd_set);
        pub(super) fn iceoryx2_fd_isset(fd: int, set: *const fd_set) -> int;
        pub(super) fn iceoryx2_fd_set(fd: int, set: *mut fd_set);
        pub(super) fn iceoryx2_fd_zero(set: *mut fd_set);
    }
}
