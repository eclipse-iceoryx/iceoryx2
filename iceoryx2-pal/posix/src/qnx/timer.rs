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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;

pub unsafe fn timer_create(
    clock_id: clockid_t,
    sevp: *mut sigevent,
    timer_id: *mut timer_t,
) -> int {
    let mut os_specific_buffer: native_sigevent = sevp.into();

    match crate::internal::timer_create(clock_id, &mut os_specific_buffer, timer_id) {
        0 => {
            *sevp = os_specific_buffer.into();
            0
        }
        v => v,
    }
}

pub unsafe fn timer_delete(timer_id: timer_t) -> int {
    crate::internal::timer_delete(timer_id)
}

pub unsafe fn timer_settime(
    timer_id: timer_t,
    flags: int,
    new_value: *const itimerspec,
    old_value: *mut itimerspec,
) -> int {
    crate::internal::timer_settime(timer_id, flags, new_value, old_value)
}

pub unsafe fn timer_gettime(timer_id: timer_t, current_value: *mut itimerspec) -> int {
    crate::internal::timer_gettime(timer_id, current_value)
}
