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

use core::{hint::spin_loop, sync::atomic::Ordering};

use crate::iox_atomic::IoxAtomicU32;
use crate::SPIN_REPETITIONS;

#[derive(Debug)]
pub struct Barrier {
    waiters: IoxAtomicU32,
}

impl Barrier {
    pub fn new(number_of_waiters: u32) -> Self {
        Self {
            waiters: IoxAtomicU32::new(number_of_waiters),
        }
    }

    pub fn wait<Wait: Fn(&IoxAtomicU32, &u32), WakeAll: Fn(&IoxAtomicU32)>(
        &self,
        wait: Wait,
        wake_all: WakeAll,
    ) {
        if self.waiters.fetch_sub(1, Ordering::AcqRel) == 1 {
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
                return;
            }

            wait(&self.waiters, &current_value);
        }
    }
}
