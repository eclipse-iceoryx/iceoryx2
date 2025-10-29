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

pub unsafe fn memset(s: *mut void, c: int, n: size_t) -> *mut void {
    unimplemented!("memset")
}

pub unsafe fn memcpy(dest: *mut void, src: *const void, n: size_t) -> *mut void {
    unimplemented!("memcpy")
}

pub unsafe fn strncpy(dest: *mut c_char, src: *const c_char, n: size_t) -> *mut c_char {
    unimplemented!("strncpy")
}
