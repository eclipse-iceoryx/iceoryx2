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

pub unsafe fn open_with_mode(pathname: *const c_char, flags: int, mode: mode_t) -> int {
    unimplemented!("open_with_mode")
}

pub unsafe fn fstat(fd: int, buf: *mut stat_t) -> int {
    unimplemented!("fstat")
}

pub unsafe fn fcntl_int(fd: int, cmd: int, arg: int) -> int {
    unimplemented!("fcntl_int")
}

pub unsafe fn fcntl(fd: int, cmd: int, arg: *mut flock) -> int {
    unimplemented!("fcntl")
}

pub unsafe fn fcntl2(fd: int, cmd: int) -> int {
    unimplemented!("fcntl2")
}

pub unsafe fn fchmod(fd: int, mode: mode_t) -> int {
    unimplemented!("fchmod")
}

pub unsafe fn open(pathname: *const c_char, flags: int) -> int {
    unimplemented!("open")
}
