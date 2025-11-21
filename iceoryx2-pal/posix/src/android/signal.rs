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
use crate::common::mem_zeroed_struct::MemZeroedStruct;
use crate::posix::sighandler_t;
use crate::posix::types::*;

#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct sigaction_t(libc::sigaction);

impl MemZeroedStruct for sigaction_t {}

impl sigaction_t {
    pub fn set_handler(&mut self, handler: sighandler_t) {
        self.0.sa_sigaction = handler;
    }

    pub fn flags(&self) -> int {
        self.0.sa_flags
    }

    pub fn set_flags(&mut self, flags: int) {
        self.0.sa_flags = flags;
    }
}

pub unsafe fn sigaction(sig: int, act: &sigaction_t, oact: &mut sigaction_t) -> int {
    libc::sigaction(sig, &act.0, &mut oact.0)
}

pub unsafe fn kill(pid: pid_t, sig: int) -> int {
    libc::kill(pid, sig)
}

pub unsafe fn sigaddset(set: *mut sigset_t, signo: int) -> int {
    libc::sigaddset(set, signo)
}

pub unsafe fn sigdelset(set: *mut sigset_t, signo: int) -> int {
    libc::sigdelset(set, signo)
}

pub unsafe fn sigismember(set: *const sigset_t, signo: int) -> int {
    libc::sigismember(set, signo)
}

pub unsafe fn sigfillset(set: *mut sigset_t) -> int {
    libc::sigfillset(set)
}

pub unsafe fn sigemptyset(set: *mut sigset_t) -> int {
    libc::sigemptyset(set)
}

pub unsafe fn sigpending(set: *mut sigset_t) -> int {
    libc::sigpending(set)
}
