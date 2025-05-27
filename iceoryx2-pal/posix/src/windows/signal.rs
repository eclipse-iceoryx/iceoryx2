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
#![allow(unused_variables)]

use iceoryx2_pal_concurrency_sync::mutex::Mutex;
use iceoryx2_pal_concurrency_sync::WaitAction;
use windows_sys::Win32::{
    Foundation::{FALSE, TRUE},
    System::{
        Console::{
            GenerateConsoleCtrlEvent, SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT,
            CTRL_C_EVENT,
        },
        Threading::{GetExitCodeProcess, OpenProcess, PROCESS_ALL_ACCESS},
    },
};

use core::cell::UnsafeCell;

use crate::{
    posix::getpid,
    posix::types::*,
    posix::{sighandler_t, MemZeroedStruct},
    posix::{Errno, SIGKILL, SIGSTOP, SIGTERM, SIGUSR1},
    win32call,
};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct sigaction_t {
    sa_handler: sighandler_t,
    sa_mask: sigset_t,
    sa_flags: int,
}

impl MemZeroedStruct for sigaction_t {
    fn new_zeroed() -> Self {
        Self {
            sa_handler: 0,
            sa_mask: sigset_t::new_zeroed(),
            sa_flags: 0,
        }
    }
}

impl sigaction_t {
    pub fn set_handler(&mut self, handler: sighandler_t) {
        self.sa_handler = handler;
    }

    pub fn flags(&self) -> int {
        self.sa_flags
    }

    pub fn set_flags(&mut self, flags: int) {
        self.sa_flags = flags;
    }
}

struct SigAction {
    action: UnsafeCell<sigaction_t>,
    mtx: Mutex,
}

impl SigAction {
    const fn new() -> Self {
        Self {
            action: UnsafeCell::new(sigaction_t {
                sa_handler: 0,
                sa_mask: sigset_t {},
                sa_flags: 0,
            }),
            mtx: Mutex::new(),
        }
    }

    fn get(&self) -> sigaction_t {
        self.mtx.lock(|_, _| WaitAction::Continue);
        let ret_val = unsafe { *self.action.get() };
        self.mtx.unlock(|_| {});
        ret_val
    }

    fn set(&self, value: sigaction_t) -> sigaction_t {
        self.mtx.lock(|_, _| WaitAction::Continue);
        let ret_val = unsafe { *self.action.get() };
        unsafe { *self.action.get() = value };
        self.mtx.unlock(|_| {});
        ret_val
    }
}

unsafe impl Send for SigAction {}
unsafe impl Sync for SigAction {}

static SIG_ACTION: SigAction = SigAction::new();

unsafe extern "system" fn ctrl_handler(value: u32) -> i32 {
    let action =
        core::mem::transmute::<sighandler_t, extern "C" fn(int)>(SIG_ACTION.get().sa_handler);

    let sigval = win32_event_to_signal(value);

    action(sigval);
    TRUE
}

fn signal_to_win32_event(sig: int) -> Option<u32> {
    match sig {
        SIGTERM => Some(CTRL_C_EVENT),
        SIGSTOP => Some(CTRL_BREAK_EVENT),
        SIGKILL => Some(CTRL_CLOSE_EVENT),
        _ => None,
    }
}

fn win32_event_to_signal(event: u32) -> int {
    match event {
        CTRL_C_EVENT => SIGTERM,
        CTRL_BREAK_EVENT => SIGSTOP,
        CTRL_CLOSE_EVENT => SIGKILL,
        _ => SIGUSR1,
    }
}

pub unsafe fn sigaction(sig: int, act: &sigaction_t, oact: &mut sigaction_t) -> int {
    *oact = SIG_ACTION.set(*act);

    if sig == SIGTERM {
        if act.sa_handler == 0 {
            SetConsoleCtrlHandler(None, FALSE);
        } else {
            SetConsoleCtrlHandler(Some(ctrl_handler), TRUE);
        }
    }
    0
}

pub unsafe fn kill(pid: pid_t, sig: int) -> int {
    if sig == 0 {
        let mut exit_code = 0;
        let (handle, _) = win32call! { OpenProcess(PROCESS_ALL_ACCESS, TRUE, pid) };
        let (has_exit_code, _) = win32call! { GetExitCodeProcess(handle, &mut exit_code) };
        return if has_exit_code == TRUE {
            0
        } else {
            Errno::set(Errno::ESRCH);
            -1
        };
    }

    if pid != getpid() {
        Errno::set(Errno::ENOTSUP);
        return -1;
    }

    match signal_to_win32_event(sig) {
        None => {
            Errno::set(Errno::ENOTSUP);
            -1
        }
        Some(e) => {
            win32call! {GenerateConsoleCtrlEvent(e, 0)};
            0
        }
    }
}
