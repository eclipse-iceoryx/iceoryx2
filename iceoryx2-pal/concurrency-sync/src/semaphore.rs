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

use crate::iox_atomic::IoxAtomicU32;
use core::{hint::spin_loop, sync::atomic::Ordering};

use crate::{WaitAction, WaitResult, SPIN_REPETITIONS};

pub struct Semaphore {
    value: IoxAtomicU32,
}

impl Semaphore {
    pub fn new(initial_value: u32) -> Self {
        Self {
            value: IoxAtomicU32::new(initial_value),
        }
    }

    pub fn value(&self) -> u32 {
        self.value.load(Ordering::Relaxed)
    }

    pub fn post<WakeUp: Fn(&IoxAtomicU32)>(&self, wakeup: WakeUp, value: u32) {
        self.value.fetch_add(value, Ordering::Acquire);
        wakeup(&self.value);
    }

    pub fn wait<Wait: Fn(&IoxAtomicU32, &u32) -> WaitAction>(&self, wait: Wait) -> WaitResult {
        let mut retry_counter = 0;
        let mut current_value = self.value.load(Ordering::Relaxed);

        let mut keep_running = true;
        loop {
            loop {
                if current_value != 0 {
                    break;
                }

                if !keep_running {
                    return WaitResult::Interrupted;
                }

                if retry_counter < SPIN_REPETITIONS {
                    spin_loop();
                    retry_counter += 1;
                } else if wait(&self.value, &current_value) == WaitAction::Abort {
                    keep_running = false;
                }
                current_value = self.value.load(Ordering::Relaxed);
            }

            match self.value.compare_exchange_weak(
                current_value,
                current_value - 1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(v) => current_value = v,
            }
        }

        WaitResult::Success
    }

    pub fn try_wait(&self) -> WaitResult {
        let mut current_value = self.value.load(Ordering::Relaxed);

        loop {
            if current_value == 0 {
                return WaitResult::Interrupted;
            }

            match self.value.compare_exchange_weak(
                current_value,
                current_value - 1,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return WaitResult::Success,
                Err(v) => current_value = v,
            }
        }
    }
}
