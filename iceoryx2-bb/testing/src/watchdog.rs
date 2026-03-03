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

use alloc::sync::Arc;
use core::time::Duration;

use iceoryx2_bb_posix::clock::{nanosleep, Time};
use iceoryx2_bb_posix::ipc_capable::Handle;
use iceoryx2_bb_posix::mutex::{Mutex, MutexBuilder, MutexHandle};
use iceoryx2_bb_posix::thread::{Thread, ThreadBuilder};
use iceoryx2_bb_print::cerrln;

pub struct Watchdog {
    termination_thread: Option<Thread>,
    keep_running: Arc<MutexHandle<bool>>,
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        let mutex = unsafe { Mutex::from_handle(&self.keep_running) };
        *mutex.lock().expect("mutex corrupted") = false;

        // thread joins on drop
        // thread exits gracefully due to keep_running = false
        drop(self.termination_thread.take());
    }
}

impl Default for Watchdog {
    fn default() -> Self {
        Self::new_with_timeout(Duration::from_secs(10))
    }
}

impl Watchdog {
    pub fn new_with_timeout(timeout: Duration) -> Self {
        let keep_running = Arc::new(MutexHandle::<bool>::new());
        let keep_running_clone = keep_running.clone();

        // initialize keep_running to true
        MutexBuilder::new()
            .is_interprocess_capable(false)
            .create(true, &keep_running)
            .expect("failed to create mutex");

        let thread = ThreadBuilder::new()
            .spawn(move || {
                let start = Time::now().expect("failure retrieving current time");

                loop {
                    nanosleep(Duration::from_millis(10)).expect("failure in nanosleep");

                    let mutex = unsafe { Mutex::from_handle(&keep_running_clone) };

                    if !*mutex.lock().expect("mutex corrupted") {
                        break;
                    }

                    if start.elapsed().unwrap_or(Duration::ZERO) > timeout {
                        cerrln!("Killing test since timeout of {timeout:?} was hit.");
                        core::panic!("Watchdog timeout exceeded");
                    }
                }
            })
            .expect("failed to spawn watchdog thread");

        Self {
            keep_running,
            termination_thread: Some(thread),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}
