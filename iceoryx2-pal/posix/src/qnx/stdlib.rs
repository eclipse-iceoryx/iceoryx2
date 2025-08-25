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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;

pub unsafe fn malloc(size: size_t) -> *mut void {
    crate::internal::malloc(size as _)
}

pub unsafe fn calloc(nmemb: size_t, size: size_t) -> *mut void {
    crate::internal::calloc(nmemb as _, size as _)
}

pub unsafe fn realloc(ptr: *mut void, size: size_t) -> *mut void {
    crate::internal::realloc(ptr, size as _)
}

pub unsafe fn free(ptr: *mut void) {
    crate::internal::free(ptr)
}
