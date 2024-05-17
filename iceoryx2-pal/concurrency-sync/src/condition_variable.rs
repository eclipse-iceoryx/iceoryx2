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

use core::sync::atomic::Ordering;

use crate::iox_atomic::IoxAtomicU32;
pub use crate::mutex::Mutex;
use crate::{semaphore::Semaphore, WaitAction, WaitResult};

pub struct ConditionVariable {
    number_of_waiters: IoxAtomicU32,
    semaphore: Semaphore,
}

impl Default for ConditionVariable {
    fn default() -> Self {
        Self {
            semaphore: Semaphore::new(0),
            number_of_waiters: IoxAtomicU32::new(0),
        }
    }
}

impl ConditionVariable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn notify_one<WakeOne: Fn(&IoxAtomicU32)>(&self, wake_one: WakeOne) {
        self.semaphore.post(
            wake_one,
            1.min(self.number_of_waiters.load(Ordering::Relaxed)),
        );
    }

    pub fn notify_all<WakeAll: Fn(&IoxAtomicU32)>(&self, wake_all: WakeAll) {
        self.semaphore
            .post(wake_all, self.number_of_waiters.load(Ordering::Relaxed));
    }

    pub fn wait<
        WakeOne: Fn(&IoxAtomicU32),
        Wait: Fn(&IoxAtomicU32, &u32) -> WaitAction,
        MtxWait: Fn(&IoxAtomicU32, &u32) -> WaitAction,
    >(
        &self,
        mtx: &Mutex,
        mtx_wake_one: WakeOne,
        wait: Wait,
        mtx_wait: MtxWait,
    ) -> WaitResult {
        self.number_of_waiters.fetch_add(1, Ordering::Relaxed);
        mtx.unlock(mtx_wake_one);

        if self.semaphore.wait(wait) == WaitResult::Interrupted {
            self.number_of_waiters.fetch_sub(1, Ordering::Relaxed);
            return WaitResult::Interrupted;
        }
        self.number_of_waiters.fetch_sub(1, Ordering::Relaxed);

        // this maybe problematic when the wait has a timeout. it is possible that
        // the timeout is nearly doubled when wait is waken up at the end of the timeout
        // as well as the mtx lock
        mtx.lock(mtx_wait)
    }
}
