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
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicU64;

const UNDECIDED: u64 = u64::MAX;
const LOST: u64 = u64::MAX;

#[derive(Debug)]
pub(crate) struct DecisionCounter(IoxAtomicU64);

impl DecisionCounter {
    pub(crate) const fn new() -> Self {
        DecisionCounter(IoxAtomicU64::new(UNDECIDED))
    }

    pub(crate) fn set_to_undecided(&self) {
        self.0.store(UNDECIDED, Ordering::Relaxed);
    }

    pub(crate) fn set(&self, value: u64) -> bool {
        self.0
            .compare_exchange(UNDECIDED, value, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
    }

    pub(crate) fn does_value_win(&self, competing_value: u64) -> bool {
        let my_value = self.0.load(Ordering::Relaxed);

        if my_value == UNDECIDED {
            match self
                .0
                .compare_exchange(UNDECIDED, LOST, Ordering::Relaxed, Ordering::Relaxed)
            {
                Err(v) => competing_value < v,
                Ok(_) => true,
            }
        } else {
            competing_value < my_value
        }
    }
}
