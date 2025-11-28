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

pub unsafe fn sem_create(name: *const c_char, oflag: int, mode: mode_t, value: uint) -> *mut sem_t {
    unimplemented!("sem_create")
}

pub unsafe fn sem_post(sem: *mut sem_t) -> int {
    unimplemented!("sem_post")
}

pub unsafe fn sem_wait(sem: *mut sem_t) -> int {
    unimplemented!("sem_wait")
}

pub unsafe fn sem_trywait(sem: *mut sem_t) -> int {
    unimplemented!("sem_trywait")
}

pub unsafe fn sem_timedwait(sem: *mut sem_t, abs_timeout: *const timespec) -> int {
    unimplemented!("sem_timedwait")
}

pub unsafe fn sem_unlink(name: *const c_char) -> int {
    unimplemented!("sem_unlink")
}

pub unsafe fn sem_open(name: *const c_char, oflag: int) -> *mut sem_t {
    unimplemented!("sem_open")
}

pub unsafe fn sem_destroy(sem: *mut sem_t) -> int {
    unimplemented!("sem_destroy")
}

pub unsafe fn sem_init(sem: *mut sem_t, pshared: int, value: uint) -> int {
    unimplemented!("sem_init")
}

pub unsafe fn sem_close(sem: *mut sem_t) -> int {
    unimplemented!("sem_close")
}
