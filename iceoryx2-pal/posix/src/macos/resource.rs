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

pub unsafe fn getrlimit(resource: int, rlim: *mut rlimit) -> int {
    crate::internal::getrlimit(resource, rlim)
}

pub unsafe fn setrlimit(resource: int, rlim: *const rlimit) -> int {
    crate::internal::setrlimit(resource, rlim)
}
