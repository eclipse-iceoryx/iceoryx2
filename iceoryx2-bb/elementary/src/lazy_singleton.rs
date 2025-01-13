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

//! Can be used to implement a singleton object which is not initialized when it is being created.
//!
//! Useful for global logger, error handling or config objects which are initialized sometime
//! during the startup phase. The object itself is not a singleton.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_elementary::lazy_singleton::*;
//!
//! static LAZY_GLOBAL: LazySingleton<u64> = LazySingleton::<u64>::new();
//!
//! // in startup phase
//! if LAZY_GLOBAL.set_value(1234) {
//!     println!("successfully initialized");
//! } else {
//!     println!("someone else already initialized the object");
//! }
//!
//! // during runtime from multiple threads
//! println!("{}", LAZY_GLOBAL.get());
//! ```

use core::{cell::UnsafeCell, sync::atomic::Ordering};
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;

/// The lazy initialized singleton building block of type T
#[derive(Debug)]
pub struct LazySingleton<T> {
    data: UnsafeCell<Option<T>>,
    is_initialized: IoxAtomicBool,
    is_finalized: IoxAtomicBool,
}

unsafe impl<T: Send> Send for LazySingleton<T> {}
unsafe impl<T: Send + Sync> Sync for LazySingleton<T> {}

impl<T> Default for LazySingleton<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LazySingleton<T> {
    /// Creates a new [`LazySingleton`] where the underlying value is not yet initialized.
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(None),
            is_initialized: IoxAtomicBool::new(false),
            is_finalized: IoxAtomicBool::new(false),
        }
    }

    /// Returns true if the underlying value was initialized, otherwise false.
    pub fn is_initialized(&self) -> bool {
        self.is_initialized.load(Ordering::Relaxed)
    }

    /// Sets the value of the uninitialized [`LazySingleton`]. If it was already initialized it
    /// returns false, otherwise true.
    pub fn set_value(&self, value: T) -> bool {
        let is_initialized =
            self.is_initialized
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed);

        if is_initialized.is_err() {
            return false;
        }

        unsafe { *self.data.get() = Some(value) };
        self.is_finalized.store(true, Ordering::Release);
        true
    }

    /// Returns a reference to the underlying object. If the [`LazySingleton`] does not contain
    /// any object it panics.
    pub fn get(&self) -> &T {
        if self.is_finalized.load(Ordering::Acquire) {
            return unsafe { self.data.get().as_ref().unwrap().as_ref().unwrap() };
        }

        if !self.is_initialized.load(Ordering::Relaxed) {
            panic!("You cannot acquire an unset value");
        }

        while !self.is_finalized.load(Ordering::Acquire) {
            core::hint::spin_loop()
        }
        unsafe { self.data.get().as_ref().unwrap().as_ref().unwrap() }
    }
}
