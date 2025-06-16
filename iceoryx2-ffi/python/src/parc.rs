// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use core::fmt::Debug;
use std::sync::{Arc, Mutex, MutexGuard};

/// Internal helper struct to handle the Python memory model where everything
/// which is shared - even accross thread boundaries - is just a pointer to
/// some heap memory location. The `Arc<Mutex<T>>` ensures basic safety and this
/// construct takes also care of the annoying `unwrap` inside the mutex. A mutex
/// can fail when the thread died holding the mutex but in those cases a panic
/// is absolutely justified in a python context.
pub struct Parc<T: Send> {
    value: Arc<Mutex<T>>,
}

impl<T: Send + Debug> Debug for Parc<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", *self.lock())
    }
}

impl<T: Send> Clone for Parc<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T: Send> Parc<T> {
    /// Creates a new [`Parc`]
    pub fn new(value: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(value)),
        }
    }

    /// Acquires a reference inside a [`MutexGuard`] to the underlying object.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.value.lock().unwrap()
    }
}
