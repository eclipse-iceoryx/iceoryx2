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

use std::{sync::Mutex, thread, time::Instant};

pub struct Watchdog {
    termination_thread: Option<thread::JoinHandle<()>>,
    keep_running: Arc<Mutex<bool>>,
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        *self.keep_running.lock().unwrap() = false;
        let handle = self.termination_thread.take();
        handle.unwrap().join().unwrap();
    }
}

impl Default for Watchdog {
    fn default() -> Self {
        Self::new_with_timeout(Duration::from_secs(10))
    }
}

impl Watchdog {
    pub fn new_with_timeout(timeout: Duration) -> Self {
        let keep_running = Arc::new(Mutex::new(true));

        Self {
            keep_running: keep_running.clone(),
            termination_thread: Some(thread::spawn(move || {
                let now = Instant::now();
                while *keep_running.lock().unwrap() {
                    std::thread::yield_now();
                    std::thread::sleep(Duration::from_millis(10));
                    std::thread::yield_now();

                    if now.elapsed() > timeout {
                        eprintln!("Killing test since timeout of {timeout:?} was hit.");
                        std::process::exit(1);
                    }
                }
            })),
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}
