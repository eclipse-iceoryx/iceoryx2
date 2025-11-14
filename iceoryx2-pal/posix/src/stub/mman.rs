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

use alloc::vec::Vec;
use core::unimplemented;

use crate::posix::types::*;

pub unsafe fn mlock(addr: *const void, len: size_t) -> int {
    unimplemented!("mlock")
}

pub unsafe fn munlock(addr: *const void, len: size_t) -> int {
    unimplemented!("munlock")
}

pub unsafe fn mlockall(flags: int) -> int {
    unimplemented!("mlockall")
}

pub unsafe fn munlockall() -> int {
    unimplemented!("munlockall")
}

pub unsafe fn shm_open(name: *const c_char, oflag: int, mode: mode_t) -> int {
    unimplemented!("shm_open")
}

pub unsafe fn shm_unlink(name: *const c_char) -> int {
    unimplemented!("shm_unlink")
}

pub unsafe fn mmap(
    addr: *mut void,
    len: size_t,
    prot: int,
    flags: int,
    fd: int,
    off: off_t,
) -> *mut void {
    unimplemented!("mmap")
}

pub unsafe fn munmap(addr: *mut void, len: size_t) -> int {
    unimplemented!("munmap")
}

pub unsafe fn mprotect(addr: *mut void, len: size_t, prot: int) -> int {
    unimplemented!("mprotect")
}

pub unsafe fn shm_list() -> Vec<[i8; 256]> {
    unimplemented!("shm_list")
}
