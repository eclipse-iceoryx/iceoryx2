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

//! # Example
//!
//! ```
//! use std::thread;
//! use iceoryx2_bb_concurrency::spin_lock::SpinLock;
//!
//! let lk = SpinLock::new(Vec::new());
//! thread::scope(|s| {
//!     s.spawn(|| lk.blocking_lock().push(1));
//!     s.spawn(|| {
//!         let mut guard = lk.blocking_lock();
//!         guard.push(2);
//!         guard.push(3);
//!     });
//! });
//! let guard = lk.try_lock().unwrap();
//! assert!(guard.as_slice() == [1, 2, 3] || guard.as_slice() == [2, 3, 1]);
//!
//! ```

use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};

use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_pal_concurrency_sync::atomic::Ordering;

use crate::atomic::AtomicBool;
use crate::cell::UnsafeCell;

/// A spin lock with two locking functions: [`SpinLock::try_lock()`] only tries once to get a lock on `T`,
/// [`SpinLock::blocking_lock()`] goes into a spin loop until it gets a lock on `T`
#[derive(Debug)]
#[repr(C)]
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

// T must be Send as SpinLock::*_lock() can be used to send values of T to another thread
// T don't need to be Sync as the SpinLock guarantees that only one thread at a time can access
// T via *_lock()
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T: Default> PlacementDefault for SpinLock<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        let locked_ptr = core::ptr::addr_of_mut!((*ptr).locked);
        let value_ptr = core::ptr::addr_of_mut!((*ptr).value);

        PlacementDefault::placement_default(locked_ptr);
        PlacementDefault::placement_default(value_ptr);
    }
}

impl<T> SpinLock<T> {
    /// Creates a new SpinLock
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Blocks until a lock on `T` is obtained. Returns a [`SpinLockGuard`] that unlocks `T` when dropped.
    pub fn blocking_lock(&self) -> SpinLockGuard<'_, T> {
        while self.locked.swap(true, Ordering::Acquire) {
            spin_loop();
        }
        SpinLockGuard { lock: self }
    }

    /// Tries once to lock `T` and returns an [`Option`]. If successful, the [`Option`] contains
    /// a [`SpinLockGuard`] that unlocks `T` when dropped, otherwise it contains [`None`].
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        (!self.locked.swap(true, Ordering::Acquire)).then(|| SpinLockGuard { lock: self })
    }
}

/// A guard for a [`SpinLock<T>`] that takes care of unlocking `T` when dropped.
pub struct SpinLockGuard<'lk, T> {
    lock: &'lk SpinLock<T>,
}

// Sync implmentation needed because the unsafe Deref(Mut) implementation causes the SpinLockGuard
// to behave like &(mut) T
// Send implementation not needed as the SpinLock implements Send, i.e. lock (&SpinLock<T>) is Send
// only if T is Send
unsafe impl<T: Sync> Sync for SpinLockGuard<'_, T> {}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // The SpinLockGuard can only be obtained via SpinLock::*_lock() which guarantees an
        // exclusive lock on T, so only one thread at a time can access T.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // The SpinLockGuard can only be obtained via SpinLock::*_lock() which guarantees an
        // exclusive lock on T, so only one thread at a time can access T.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}
