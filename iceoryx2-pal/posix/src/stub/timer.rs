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
    _clock_id: clockid_t,
    _sevp: *mut sigevent,
    _timer_id: *mut timer_t,
) -> int {
    0
}

pub unsafe fn timer_delete(_timer_id: timer_t) -> int {
    0
}

pub unsafe fn timer_settime(
    _timer_id: timer_t,
    _flags: int,
    _new_value: *const itimerspec,
    _old_value: *mut itimerspec,
) -> int {
    0
}

pub unsafe fn timer_gettime(_timer_id: timer_t, _current_value: *mut itimerspec) -> int {
    0
}
