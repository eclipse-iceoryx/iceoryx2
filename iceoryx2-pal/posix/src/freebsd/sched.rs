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

#![allow(non_camel_case_types, dead_code)]
#![allow(clippy::missing_safety_doc)]

use crate::posix::types::*;

pub unsafe fn sched_get_priority_max(policy: int) -> int {
    crate::internal::sched_get_priority_max(policy)
}

pub unsafe fn sched_get_priority_min(policy: int) -> int {
    crate::internal::sched_get_priority_min(policy)
}

pub unsafe fn sched_yield() -> int {
    crate::internal::sched_yield()
}

pub unsafe fn sched_getparam(pid: pid_t, param: *mut sched_param) -> int {
    crate::internal::sched_getparam(pid, param)
}

pub unsafe fn sched_getscheduler(pid: pid_t) -> int {
    crate::internal::sched_getscheduler(pid)
}

pub unsafe fn sched_setparam(pid: pid_t, param: *const sched_param) -> int {
    crate::internal::sched_setparam(pid, param)
}

pub unsafe fn sched_setscheduler(pid: pid_t, policy: int, param: *const sched_param) -> int {
    crate::internal::sched_setscheduler(pid, policy, param)
}
