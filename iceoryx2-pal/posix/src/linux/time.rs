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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;

pub unsafe fn clock_gettime(clock_id: clockid_t, tp: *mut timespec) -> int {
    crate::internal::clock_gettime(clock_id, tp)
}

pub unsafe fn clock_settime(clock_id: clockid_t, tp: *const timespec) -> int {
    crate::internal::clock_settime(clock_id, tp)
}

pub unsafe fn clock_nanosleep(
    clock_id: clockid_t,
    flags: int,
    rqtp: *const timespec,
    rmtp: *mut timespec,
) -> int {
    crate::internal::clock_nanosleep(clock_id, flags, rqtp, rmtp)
}
