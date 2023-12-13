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

pub unsafe fn sigaction(sig: int, act: *const sigaction_t, oact: *mut sigaction_t) -> int {
    crate::internal::sigaction(
        sig,
        act as *const crate::internal::sigaction,
        oact as *mut crate::internal::sigaction,
    )
}

pub unsafe fn kill(pid: pid_t, sig: int) -> int {
    crate::internal::kill(pid, sig)
}
