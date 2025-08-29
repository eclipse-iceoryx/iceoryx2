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
use crate::common::mem_zeroed_struct::MemZeroedStruct;
use crate::posix::sighandler_t;
use crate::posix::types::*;

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct sigaction_t(crate::internal::sigaction);

impl MemZeroedStruct for sigaction_t {}

impl sigaction_t {
    pub fn set_handler(&mut self, handler: sighandler_t) {
        unsafe {
            self.0.__sa_un._sa_handler = core::mem::transmute::<
                usize,
                core::option::Option<unsafe extern "C" fn(i32)>,
            >(handler);
        }
    }

    pub fn flags(&self) -> int {
        self.0.sa_flags
    }

    pub fn set_flags(&mut self, flags: int) {
        self.0.sa_flags = flags;
    }
}

impl core::fmt::Debug for sigaction_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let sa_handler = unsafe {
            #[allow(clippy::missing_transmute_annotations)]
            core::mem::transmute::<_, usize>(self.0.__sa_un._sa_handler)
        };
        f.debug_struct("sigaction_t")
            .field("__sigaction_handler", &sa_handler)
            .field("sa_mask", &self.0.sa_mask)
            .field("sa_flags", &self.0.sa_flags)
            .finish()
    }
}

pub unsafe fn sigaction(sig: int, act: &sigaction_t, oact: &mut sigaction_t) -> int {
    crate::internal::sigaction(sig, &act.0, &mut oact.0)
}

pub unsafe fn kill(pid: pid_t, sig: int) -> int {
    crate::internal::kill(pid, sig)
}
