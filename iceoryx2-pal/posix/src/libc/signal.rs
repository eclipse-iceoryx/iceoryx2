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

#[derive(Copy, Clone)]
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

impl core::fmt::Debug for sigaction_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[cfg(target_pointer_width = "32")]
        type sa_mask_underlying = [u32; 32];
        #[cfg(target_pointer_width = "64")]
        type sa_mask_underlying = [u64; 16];

        let sa_mask = unsafe {
            #[allow(clippy::missing_transmute_annotations)]
            core::mem::transmute::<_, sa_mask_underlying>(self.0.sa_mask)
        };

        f.debug_struct("sigaction_t")
            .field("sa_sigaction", &self.0.sa_sigaction)
            .field("sa_mask", &sa_mask)
            .field("sa_flags", &self.0.sa_flags)
            .finish()
    }
}

pub unsafe fn sigaction(sig: int, act: &sigaction_t, oact: &mut sigaction_t) -> int {
    libc::sigaction(sig, &act.0, &mut oact.0)
}

pub unsafe fn kill(pid: pid_t, sig: int) -> int {
    libc::kill(pid, sig)
}
