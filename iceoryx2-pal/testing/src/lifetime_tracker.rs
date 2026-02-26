// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use alloc::vec;
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::sync::Mutex;
#[cfg(feature = "std")]
use std::sync::MutexGuard;

#[cfg(not(feature = "std"))]
use iceoryx2_pal_concurrency_sync::spin_lock::SpinLock as Mutex;
#[cfg(not(feature = "std"))]
use iceoryx2_pal_concurrency_sync::spin_lock::SpinLockGuard as MutexGuard;

static CREATION_COUNTER: Mutex<usize> = Mutex::new(0);
static DROP_COUNTER: Mutex<usize> = Mutex::new(0);

static DROP_ORDER: Mutex<Vec<usize>> = Mutex::new(vec![]);
static TRACKING_LOCK: Mutex<LifetimeTrackingState> = Mutex::new(LifetimeTrackingState {});

#[derive(Debug)]
pub struct LifetimeTrackingState {}

impl LifetimeTrackingState {
    pub fn number_of_living_instances(&self) -> usize {
        *CREATION_COUNTER.lock().unwrap() - *DROP_COUNTER.lock().unwrap()
    }

    pub fn drop_order(&self) -> Vec<usize> {
        DROP_ORDER.lock().unwrap().clone()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LifetimeTracker {
    pub value: usize,
}

impl Default for LifetimeTracker {
    fn default() -> Self {
        *CREATION_COUNTER.lock().unwrap() += 1;

        Self { value: 0 }
    }
}

impl LifetimeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_value(value: usize) -> Self {
        let mut new_self = Self::new();
        new_self.value = value;
        new_self
    }

    pub fn start_tracking() -> MutexGuard<'static, LifetimeTrackingState> {
        let guard = TRACKING_LOCK.lock().unwrap();

        *CREATION_COUNTER.lock().unwrap() = 0;
        *DROP_COUNTER.lock().unwrap() = 0;
        DROP_ORDER.lock().unwrap().clear();

        guard
    }
}

impl Clone for LifetimeTracker {
    fn clone(&self) -> Self {
        let mut new_self = Self::new();
        new_self.value = self.value;
        new_self
    }
}

impl Drop for LifetimeTracker {
    fn drop(&mut self) {
        *DROP_COUNTER.lock().unwrap() += 1;
        DROP_ORDER.lock().unwrap().push(self.value);
    }
}
