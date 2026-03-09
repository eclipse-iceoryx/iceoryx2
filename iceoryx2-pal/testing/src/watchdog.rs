// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::time::Duration;

use iceoryx2_pal_posix::posix::{
    self, long, sigaction_t, sighandler_t,
    types::{int, itimerspec, sigevent, time_t, timer_t, timespec},
    Errno, MemZeroedStruct, CLOCK_REALTIME, SIGALRM, SIGEV_SIGNAL,
};

/// Fires `SIGALRM` after a configurable timeout, terminating the process via
/// [`posix::abort()`] in the signal handler. Intended for use in tests to
/// prevent hangs.
pub struct Watchdog {
    timer_id: timer_t,
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        // Disarm first (zero it_value), then release the kernel timer object.
        let disarm = itimerspec {
            it_interval: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            it_value: timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        };
        unsafe {
            posix::timer_settime(self.timer_id, 0, &disarm, core::ptr::null_mut());
            posix::timer_delete(self.timer_id);
        }
    }
}

impl Default for Watchdog {
    fn default() -> Self {
        Self::new_with_timeout(Duration::from_secs(10))
    }
}

/// Called by the kernel when `SIGALRM` fires.
///
/// `panic!` is not async-signal-safe (it may allocate and unwind), so
/// [`posix::abort()`] is used instead to terminate immediately.
unsafe extern "C" fn handler(_sig: int) {
    posix::abort();
}

impl Watchdog {
    /// Creates an `Alarm` that aborts the process after `timeout` elapses.
    pub fn new_with_timeout(timeout: Duration) -> Self {
        unsafe {
            // Register the handler for SIGALRM
            let mut sa = sigaction_t::new_zeroed();
            sa.set_handler(handler as *const () as sighandler_t);
            posix::sigaction(SIGALRM, &sa, &mut sigaction_t::new_zeroed());

            // SIGEV_SIGNAL delivers the SIGALRM signal to the process on timer expiry.
            let mut sev: sigevent = core::mem::zeroed();
            sev.sigev_notify = SIGEV_SIGNAL;
            sev.sigev_signo = SIGALRM;

            let mut timer_id: timer_t = core::mem::zeroed();
            if posix::timer_create(CLOCK_REALTIME, &mut sev, &mut timer_id) == -1 {
                panic!("failed to create POSIX timer, errno: {:?}", Errno::get());
            }

            // it_interval = 0 makes this a one-shot timer.
            let its = itimerspec {
                it_interval: timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                },
                it_value: timespec {
                    tv_sec: timeout.as_secs() as time_t,
                    tv_nsec: timeout.subsec_nanos() as long,
                },
            };
            if posix::timer_settime(timer_id, 0, &its, core::ptr::null_mut()) == -1 {
                panic!("failed to arm POSIX timer, errno: {:?}", Errno::get());
            }

            Self { timer_id }
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}
