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

use crate::common::mem_zeroed_struct::MemZeroedStruct;
use crate::posix::types::*;

pub unsafe fn stat(path: *const c_char, buf: *mut stat_t) -> int {
    let mut os_specific_buffer = native_stat_t::new_zeroed();
    match crate::internal::stat(path, &mut os_specific_buffer) {
        0 => {
            *buf = os_specific_buffer.into();
            0
        }
        v => v,
    }
}

pub unsafe fn umask(mask: mode_t) -> mode_t {
    crate::internal::umask(mask)
}
