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

use windows_sys::Win32::System::Threading::GetCurrentThreadStackLimits;

use crate::{
    posix::{RLIMIT_STACK, types::*},
    win32call,
};

pub unsafe fn getrlimit(resource: int, rlim: *mut rlimit) -> int {
    match resource as u64 {
        RLIMIT_STACK => {
            let mut low = 0;
            let mut high = 0;
            unsafe {
                win32call! {GetCurrentThreadStackLimits(&mut low, &mut high)}
            };
            let stack_size = high - low;
            unsafe { (*rlim).rlim_cur = stack_size as _ };
            unsafe { (*rlim).rlim_max = stack_size as _ };
            0
        }
        _ => 0,
    }
}

pub unsafe fn setrlimit(resource: int, rlim: *const rlimit) -> int {
    0
}
