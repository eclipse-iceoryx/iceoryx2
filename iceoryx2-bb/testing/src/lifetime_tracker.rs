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

use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use std::sync::{Mutex, MutexGuard};

static CREATION_COUNTER: Mutex<usize> = Mutex::new(0);
static DROP_COUNTER: Mutex<usize> = Mutex::new(0);

static TRACKING_LOCK: Mutex<LifetimeTrackingState> = Mutex::new(LifetimeTrackingState {});

pub struct LifetimeTrackingState {}

impl LifetimeTrackingState {
    pub fn number_of_living_instances(&self) -> usize {
        *CREATION_COUNTER.lock().unwrap() - *DROP_COUNTER.lock().unwrap()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct LifetimeTracker {}

impl Default for LifetimeTracker {
    fn default() -> Self {
        *CREATION_COUNTER.lock().unwrap() += 1;

        Self {}
    }
}

#[doc(hidden)]
// ZeroCopySend can be derived because LifetimeTracker is only used for process local test purposes
unsafe impl ZeroCopySend for LifetimeTracker {}

impl LifetimeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_tracking() -> MutexGuard<'static, LifetimeTrackingState> {
        let guard = TRACKING_LOCK.lock().unwrap();

        *CREATION_COUNTER.lock().unwrap() = 0;
        *DROP_COUNTER.lock().unwrap() = 0;

        guard
    }
}

impl Clone for LifetimeTracker {
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl Drop for LifetimeTracker {
    fn drop(&mut self) {
        *DROP_COUNTER.lock().unwrap() += 1;
    }
}

impl PartialEq for LifetimeTracker {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

impl Eq for LifetimeTracker {}
