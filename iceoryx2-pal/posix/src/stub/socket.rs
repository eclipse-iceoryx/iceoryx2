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

#![allow(non_camel_case_types, non_snake_case)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]

use core::unimplemented;

use crate::posix::types::*;

pub unsafe fn socketpair(
    domain: int,
    socket_type: int,
    protocol: int,
    socket_vector: *mut int, // actually it shall be [int; 2]
) -> int {
    unimplemented!("socketpair")
}

pub unsafe fn setsockopt(
    socket: int,
    level: int,
    option_name: int,
    option_value: *const void,
    option_len: socklen_t,
) -> int {
    unimplemented!("setsockopt")
}

pub unsafe fn getsockname(socket: int, address: *mut sockaddr, address_len: *mut socklen_t) -> int {
    unimplemented!("getsockname")
}

pub unsafe fn getsockopt(
    socket: int,
    level: int,
    option_name: int,
    option_value: *mut void,
    option_len: *mut socklen_t,
) -> int {
    unimplemented!("getsockopt")
}

pub unsafe fn bind(socket: int, address: *const sockaddr, address_len: socklen_t) -> int {
    unimplemented!("bind")
}

pub unsafe fn connect(socket: int, address: *const sockaddr, address_len: socklen_t) -> int {
    unimplemented!("connect")
}

pub unsafe fn socket(domain: int, socket_type: int, protocol: int) -> int {
    unimplemented!("socket")
}

pub unsafe fn sendmsg(socket: int, message: *const msghdr, flags: int) -> ssize_t {
    unimplemented!("sendmsg")
}

pub unsafe fn sendto(
    socket: int,
    message: *const void,
    length: size_t,
    flags: int,
    dest_addr: *const sockaddr,
    dest_len: socklen_t,
) -> ssize_t {
    unimplemented!("sendto")
}

pub unsafe fn send(socket: int, message: *const void, length: size_t, flags: int) -> ssize_t {
    unimplemented!("send")
}

pub unsafe fn recvmsg(socket: int, message: *mut msghdr, flags: int) -> ssize_t {
    unimplemented!("recvmsg")
}

pub unsafe fn recvfrom(
    socket: int,
    buffer: *mut void,
    length: size_t,
    flags: int,
    address: *mut sockaddr,
    address_len: *mut socklen_t,
) -> ssize_t {
    unimplemented!("recvfrom")
}

pub unsafe fn recv(socket: int, buffer: *mut void, length: size_t, flags: int) -> ssize_t {
    unimplemented!("recv")
}
