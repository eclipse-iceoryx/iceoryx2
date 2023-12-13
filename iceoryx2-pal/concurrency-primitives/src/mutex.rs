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

use core::{
    hint::spin_loop,
    sync::atomic::{AtomicU32, Ordering},
};

pub struct Mutex {
    // we use an AtomicU32 since it should be supported on nearly every platform
    state: AtomicU32,
}

impl Default for Mutex {
    fn default() -> Self {
        Self::new()
    }
}

impl Mutex {
    pub const fn new() -> Self {
        Self {
            state: AtomicU32::new(0),
        }
    }

    pub fn lock<Wait: Fn(&AtomicU32, &u32) -> bool>(&self, wait: Wait) -> bool {
        if self.uncontested_lock(crate::SPIN_REPETITIONS) {
            return true;
        }

        loop {
            let keep_running = wait(&self.state, &1);

            if self
                .state
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return true;
            }

            if !keep_running {
                return false;
            }
        }
    }

    pub fn unlock<WakeOne: Fn(&AtomicU32)>(&self, wake_one: WakeOne) {
        self.state.store(0, Ordering::Release);
        wake_one(&self.state);
    }

    pub fn try_lock(&self) -> bool {
        self.state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    fn uncontested_lock(&self, retry_limit: u64) -> bool {
        if self.try_lock() {
            return true;
        }

        let mut retry_counter = 0;
        while self
            .state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            spin_loop();
            retry_counter += 1;

            if retry_limit == retry_counter {
                return false;
            }
        }

        true
    }
}
