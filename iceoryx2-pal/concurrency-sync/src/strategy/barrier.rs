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
use core::panic;

use crate::atomic::AtomicU64;
use crate::atomic::Ordering;
use crate::SPIN_REPETITIONS;

#[derive(Debug)]
pub struct Barrier {
    waiters: AtomicU64,
    number_of_waiters: u32,
}

fn pack(epoch: u32, count: u32) -> u64 {
    ((epoch as u64) << 32) | (count as u64)
}

fn unpack(value: u64) -> (u32, u32) {
    ((value >> 32) as u32, value as u32)
}

impl Barrier {
    pub fn new(number_of_waiters: u32) -> Self {
        Self {
            number_of_waiters,
            waiters: AtomicU64::new(0),
        }
    }

    pub fn wait<Wait: Fn(&AtomicU64, &u64), WakeAll: Fn(&AtomicU64)>(
        &self,
        wait: Wait,
        wake_all: WakeAll,
    ) {
        let (current_epoch, count) = unpack(self.waiters.fetch_add(1, Ordering::Release));

        let current_count = count + 1;
        if current_count == self.number_of_waiters {
            self.waiters
                .store(pack(current_epoch.wrapping_add(1), 0), Ordering::Release);
            wake_all(&self.waiters);
            return;
        } else if current_count > self.number_of_waiters {
            panic!("Barrier::wait() contract violation! More threads than configured call Barrier::wait() concurrently.");
        }

        let mut retry_counter = 0;
        loop {
            let current_value = self.waiters.load(Ordering::Acquire);
            let (epoch, _) = unpack(current_value);

            if epoch != current_epoch {
                return;
            }

            if retry_counter < SPIN_REPETITIONS {
                spin_loop();
                retry_counter += 1;
            } else {
                wait(&self.waiters, &current_value);
            }
        }
    }
}
