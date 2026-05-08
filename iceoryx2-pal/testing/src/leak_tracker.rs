// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

#[cfg(feature = "std")]
use std::sync::Mutex;

#[cfg(not(feature = "std"))]
use iceoryx2_pal_concurrency_sync::spin_lock::SpinLock as Mutex;

use crate::leakable::Abandonable;

static CREATION_COUNTER: Mutex<usize> = Mutex::new(0);
static DROP_COUNTER: Mutex<usize> = Mutex::new(0);
static LEAK_COUNTER: Mutex<usize> = Mutex::new(0);

#[derive(Debug)]
pub struct LeakTrackingState {}

impl LeakTrackingState {
    pub fn creation_count(&self) -> usize {
        *CREATION_COUNTER.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn drop_count(&self) -> usize {
        *DROP_COUNTER.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn leak_count(&self) -> usize {
        *LEAK_COUNTER.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeakTracker {
    pub value: usize,
}

impl Abandonable for LeakTracker {
    unsafe fn abandon_in_place(_this: *mut Self) {
        *LEAK_COUNTER.lock().unwrap_or_else(|e| e.into_inner()) += 1;
    }
}

impl Default for LeakTracker {
    fn default() -> Self {
        *CREATION_COUNTER.lock().unwrap_or_else(|e| e.into_inner()) += 1;

        Self { value: 0 }
    }
}

impl LeakTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_value(value: usize) -> Self {
        let mut new_self = Self::new();
        new_self.value = value;
        new_self
    }

    pub fn start_tracking() -> LeakTrackingState {
        *CREATION_COUNTER.lock().unwrap_or_else(|e| e.into_inner()) = 0;
        *LEAK_COUNTER.lock().unwrap_or_else(|e| e.into_inner()) = 0;
        *DROP_COUNTER.lock().unwrap_or_else(|e| e.into_inner()) = 0;

        LeakTrackingState {}
    }
}

impl Clone for LeakTracker {
    fn clone(&self) -> Self {
        let mut new_self = Self::new();
        new_self.value = self.value;
        new_self
    }
}

impl Drop for LeakTracker {
    fn drop(&mut self) {
        *DROP_COUNTER.lock().unwrap_or_else(|e| e.into_inner()) += 1;
    }
}
