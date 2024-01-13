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
use crate::posix::{types::*, Struct};

impl Struct for crate::internal::sigaction {}

impl From<&sigaction_t> for crate::internal::sigaction {
    fn from(value: &sigaction_t) -> Self {
        let mut this = crate::internal::sigaction::new();
        this.sa_mask = value.sa_mask;
        this.sa_flags = value.sa_flags;
        this.__sigaction_handler.sa_handler =
            Some(unsafe { core::mem::transmute(value.sa_handler) });

        this
    }
}

impl From<&crate::internal::sigaction> for sigaction_t {
    fn from(value: &crate::internal::sigaction) -> Self {
        let mut this = sigaction_t::new();
        this.sa_mask = value.sa_mask;
        this.sa_flags = value.sa_flags;
        this.sa_handler = match unsafe { value.__sigaction_handler.sa_handler } {
            Some(v) => unsafe { core::mem::transmute(v) },
            None => 0,
        };

        this
    }
}

pub unsafe fn sigaction(sig: int, act: *const sigaction_t, oact: *mut sigaction_t) -> int {
    let c_act: crate::internal::sigaction = if act.is_null() {
        crate::internal::sigaction::new()
    } else {
        (&*act).into()
    };

    let mut c_oact: crate::internal::sigaction = if oact.is_null() {
        crate::internal::sigaction::new()
    } else {
        (&*oact).into()
    };

    let ret_val = crate::internal::sigaction(
        sig,
        if act.is_null() {
            core::ptr::null()
        } else {
            &c_act
        },
        if oact.is_null() {
            core::ptr::null_mut()
        } else {
            &mut c_oact
        },
    );

    if ret_val == 0 && !oact.is_null() {
        *oact = (&c_oact).into();
    }

    ret_val
}

pub unsafe fn kill(pid: pid_t, sig: int) -> int {
    crate::internal::kill(pid, sig)
}
