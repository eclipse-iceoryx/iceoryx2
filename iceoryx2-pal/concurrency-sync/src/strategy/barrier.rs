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

use core::hint::spin_loop;

use crate::atomic::AtomicU32;
use crate::atomic::Ordering;
use crate::SPIN_REPETITIONS;

#[derive(Debug)]
pub struct Barrier {
    waiters: AtomicU32,
    number_of_waiters: u32,
}

impl Barrier {
    pub fn new(number_of_waiters: u32) -> Self {
        Self {
            number_of_waiters,
            waiters: AtomicU32::new(number_of_waiters),
        }
    }

    fn reset_barrier(&self) {
        let _ = self.waiters.compare_exchange(
            0,
            self.number_of_waiters,
            Ordering::Relaxed,
            Ordering::Relaxed,
        );
    }

    pub fn wait<Wait: Fn(&AtomicU32, &u32), WakeAll: Fn(&AtomicU32)>(
        &self,
        wait: Wait,
        wake_all: WakeAll,
    ) {
        if self.waiters.fetch_sub(1, Ordering::AcqRel) == 1 {
            if self.number_of_waiters == 1 {
                self.reset_barrier();
            }
            wake_all(&self.waiters);
            return;
        }

        let mut retry_counter = 0;
        while self.waiters.load(Ordering::Acquire) > 0 {
            spin_loop();
            retry_counter += 1;

            if SPIN_REPETITIONS == retry_counter {
                break;
            }
        }

        loop {
            let current_value = self.waiters.load(Ordering::Acquire);
            if current_value == 0 {
                self.reset_barrier();
                return;
            }

            wait(&self.waiters, &current_value);
        }
    }
}
