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

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU32;
use iceoryx2_pal_concurrency_sync::semaphore::Semaphore;
use iceoryx2_pal_concurrency_sync::{WaitAction, WaitResult};
use windows_sys::Win32::System::Threading::WaitOnAddress;
use windows_sys::Win32::System::Threading::WakeByAddressSingle;
use windows_sys::Win32::System::Threading::INFINITE;

use crate::posix::constants::*;
use crate::posix::types::*;
use crate::posix::Errno;

pub unsafe fn sem_create(name: *const c_char, oflag: int, mode: mode_t, value: uint) -> *mut sem_t {
    SEM_FAILED
}

pub unsafe fn sem_post(sem: *mut sem_t) -> int {
    if (*sem).semaphore.value() == u32::MAX {
        Errno::set(Errno::EOVERFLOW);
        return -1;
    }

    (*sem).semaphore.post(
        |atomic| {
            WakeByAddressSingle((atomic as *const IoxAtomicU32).cast());
        },
        1,
    );

    Errno::set(Errno::ESUCCES);
    0
}

pub unsafe fn sem_wait(sem: *mut sem_t) -> int {
    (*sem).semaphore.wait(|atomic, value| -> WaitAction {
        WaitOnAddress(
            (atomic as *const IoxAtomicU32).cast(),
            (value as *const u32).cast(),
            4,
            INFINITE,
        );

        WaitAction::Continue
    });

    Errno::set(Errno::ESUCCES);
    0
}

pub unsafe fn sem_trywait(sem: *mut sem_t) -> int {
    match (*sem).semaphore.try_wait() {
        WaitResult::Success => {
            Errno::set(Errno::ESUCCES);
            0
        }
        WaitResult::Interrupted => {
            Errno::set(Errno::EAGAIN);
            -1
        }
    }
}

pub unsafe fn sem_timedwait(sem: *mut sem_t, abs_timeout: *const timespec) -> int {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let milli_seconds = (*abs_timeout).tv_sec * 1000 + (*abs_timeout).tv_nsec as i64 / 1000000
        - now.as_millis() as i64;

    #[allow(clippy::blocks_in_conditions)]
    match (*sem).semaphore.wait(|atomic, value| -> WaitAction {
        WaitOnAddress(
            (atomic as *const IoxAtomicU32).cast(),
            (value as *const u32).cast(),
            4,
            milli_seconds as _,
        );

        WaitAction::Abort
    }) {
        WaitResult::Success => {
            Errno::set(Errno::ESUCCES);
            0
        }
        WaitResult::Interrupted => {
            Errno::set(Errno::ETIMEDOUT);
            -1
        }
    }
}

pub unsafe fn sem_unlink(name: *const c_char) -> int {
    -1
}

pub unsafe fn sem_open(name: *const c_char, oflag: int) -> *mut sem_t {
    SEM_FAILED
}

pub unsafe fn sem_close(sem: *mut sem_t) -> int {
    -1
}

pub unsafe fn sem_destroy(sem: *mut sem_t) -> int {
    0
}

pub unsafe fn sem_init(sem: *mut sem_t, pshared: int, value: uint) -> int {
    (*sem).semaphore = Semaphore::new(value as _);
    Errno::set(Errno::ESUCCES);
    0
}
