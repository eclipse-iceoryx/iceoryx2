// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use std::ffi::CStr;

use r2r_rcl::rmw_get_implementation_identifier;

/// The rmw implementation the process is using.
pub fn implementation_identifier() -> &'static str {
    // SAFETY: rmw returns a static, NUL-terminated name.
    let name = unsafe { CStr::from_ptr(rmw_get_implementation_identifier()) };
    name.to_str().unwrap_or_default()
}
