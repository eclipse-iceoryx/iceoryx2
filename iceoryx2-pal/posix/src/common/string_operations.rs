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

#![allow(dead_code)]

pub(crate) unsafe fn c_string_length(value: *const crate::posix::c_char) -> usize {
    for i in 0..isize::MAX {
        if *value.offset(i) == crate::posix::NULL_TERMINATOR {
            return i as usize;
        }
    }

    unreachable!()
}

pub(crate) unsafe fn c_wide_string_length(value: *const u16) -> usize {
    for i in 0..usize::MAX {
        if *value.add(i) == 0u16 {
            return i;
        }
    }

    0
}
