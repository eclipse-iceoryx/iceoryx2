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

#![allow(non_camel_case_types)]
#![allow(clippy::missing_safety_doc)]
#![allow(unused_variables)]
#![allow(dead_code)]

use core::option::Option;
use core::unimplemented;

use crate::posix::types::*;

use crate::common::mem_zeroed_struct::MemZeroedStruct;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct sigaction_t {
    pub sa_sigaction: sighandler_t,
    pub sa_mask: sigset_t,
    pub sa_flags: int,
    pub sa_restorer: Option<extern "C" fn()>,
}
impl MemZeroedStruct for sigaction_t {}

impl sigaction_t {
    pub fn set_handler(&mut self, handler: sighandler_t) {
        self.sa_sigaction = handler;
    }

    pub fn flags(&self) -> int {
        self.sa_flags
    }

    pub fn set_flags(&mut self, flags: int) {
        self.sa_flags = flags;
    }
}

impl core::fmt::Debug for sigaction_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(target_pointer_width = "32")]
        type sa_mask_underlying = [u32; 32];
        #[cfg(target_pointer_width = "64")]
        type sa_mask_underlying = [u64; 16];

        f.debug_struct("sigaction_t")
            .field("sa_sigaction", &self.sa_sigaction)
            .field("sa_mask", &self.sa_mask)
            .field("sa_flags", &self.sa_flags)
            .finish()
    }
}

pub unsafe fn sigaction(sig: int, act: &sigaction_t, oact: &mut sigaction_t) -> int {
    unimplemented!("sigaction")
}

pub unsafe fn kill(pid: pid_t, sig: int) -> int {
    unimplemented!("kill")
}

pub unsafe fn sigaddset(set: *mut sigset_t, signo: int) -> int {
    unimplemented!("sigaddset")
}

pub unsafe fn sigdelset(set: *mut sigset_t, signo: int) -> int {
    unimplemented!("sigdelset")
}

pub unsafe fn sigismember(set: *const sigset_t, signo: int) -> int {
    unimplemented!("sigismember")
}

pub unsafe fn sigfillset(set: *mut sigset_t) -> int {
    unimplemented!("sigfillset")
}

pub unsafe fn sigemptyset(set: *mut sigset_t) -> int {
    unimplemented!("sigemptyset")
}

pub unsafe fn sigpending(set: *mut sigset_t) -> int {
    unimplemented!("sigpending")
}

pub unsafe fn abort() {
    unimplemented!("abort")
}
