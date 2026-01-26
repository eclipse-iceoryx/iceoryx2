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

use core::mem::MaybeUninit;
use core::ops::Deref;

use crate::cell::UnsafeCell;
use crate::once::Once;

/// A synchronization primitive for lazy initialization.
///
/// This type provides thread-safe lazy initialization using spin-based
/// synchronization via the [`Once`] primitive.
///
/// # Examples
///
/// ```
/// use iceoryx2_pal_concurrency_sync::lazy_lock::LazyLock;
///
/// static VALUE: LazyLock<u32> = LazyLock::new(|| 42);
///
/// assert_eq!(*VALUE, 42);
/// ```
pub struct LazyLock<T, F = fn() -> T> {
    once: Once,
    data: UnsafeCell<MaybeUninit<T>>,
    init: UnsafeCell<Option<F>>,
}

unsafe impl<T: Send + Sync, F: Send> Sync for LazyLock<T, F> {}

impl<T, F> LazyLock<T, F> {
    /// Creates a new lazy-initialized value with the given initializing function.
    ///
    /// The function will not be called until the value is first accessed.
    pub const fn new(init: F) -> Self {
        Self {
            once: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
            init: UnsafeCell::new(Some(init)),
        }
    }

    /// Get a reference to the stored data.
    ///
    /// # Safety
    ///
    /// Must only be called after initialization is complete.
    unsafe fn get(&self) -> &T {
        let data_ptr = self.data.get();
        (*data_ptr).assume_init_ref()
    }

    /// Initialize the data by taking the init function, calling it, and storing the result.
    ///
    /// # Safety
    ///
    /// Must only be called once. The function is consumed after first execution leaving an
    /// empty optional.
    unsafe fn init(&self)
    where
        F: FnOnce() -> T,
    {
        let init_ptr = self.init.get();
        let init_fn = (*init_ptr)
            .take()
            .expect("initialization function already taken");
        let value = init_fn();

        let data_ptr = self.data.get();
        (*data_ptr).write(value);
    }
}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    /// Forces the evaluation of this lazy value and returns a reference to
    /// result. This is equivalent to the `Deref` impl, but is explicit.
    ///
    /// This method will block the calling thread if another initialization
    /// routine is currently running.
    pub fn force(&self) -> &T {
        // Safety: call_once guarantees that the init function is only ever called once,
        // ensuring usage here is safe
        self.once.call_once(|| unsafe {
            self.init();
        });

        // Safety: After call_once returns, initialization is guaranteed complete and
        // thus retrieving the value is safe.
        unsafe { self.get() }
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.force()
    }
}
