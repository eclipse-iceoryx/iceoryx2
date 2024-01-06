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

use crate::posix::types::*;

pub unsafe fn getpwnam_r(
    name: *const c_char,
    pwd: *mut passwd,
    buf: *mut c_char,
    buflen: size_t,
    result: *mut *mut passwd,
) -> int {
    -1
}

pub unsafe fn getpwuid_r(
    uid: uid_t,
    pwd: *mut passwd,
    buf: *mut c_char,
    buflen: size_t,
    result: *mut *mut passwd,
) -> int {
    -1
}

pub unsafe fn getgrnam_r(
    name: *const c_char,
    grp: *mut group,
    buf: *mut c_char,
    buflen: size_t,
    result: *mut *mut group,
) -> int {
    -1
}

pub unsafe fn getgrgid_r(
    gid: gid_t,
    grp: *mut group,
    buf: *mut c_char,
    buflen: size_t,
    result: *mut *mut group,
) -> int {
    -1
}
