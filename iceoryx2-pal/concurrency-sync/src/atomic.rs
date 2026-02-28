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

use core::{
    fmt::Debug,
    marker::Copy,
    ops::{AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, Not, SubAssign},
};

#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type Ordering = core::sync::atomic::Ordering;
#[cfg(all(test, loom, feature = "std"))]
pub use loom::sync::atomic::Ordering;

#[cfg(not(all(test, loom, feature = "std")))]
pub use core::sync::atomic::fence;
#[cfg(all(test, loom, feature = "std"))]
pub use loom::sync::atomic::fence;

use crate::cell::UnsafeCell;
use crate::rwlock::RwLockWriterPreference;
use crate::WaitAction;

/// Behaves like [`core::sync::atomic::AtomicBool`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicBool = core::sync::atomic::AtomicBool;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicBool = loom::sync::atomic::AtomicBool;

/// Behaves like [`core::sync::atomic::AtomicUsize`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicUsize = core::sync::atomic::AtomicUsize;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicUsize = loom::sync::atomic::AtomicUsize;

/// Behaves like [`core::sync::atomic::AtomicIsize`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicIsize = core::sync::atomic::AtomicIsize;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicIsize = loom::sync::atomic::AtomicIsize;

/// Behaves like [`core::sync::atomic::AtomicU8`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicU8 = core::sync::atomic::AtomicU8;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicU8 = loom::sync::atomic::AtomicU8;

/// Behaves like [`core::sync::atomic::AtomicU16`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicU16 = core::sync::atomic::AtomicU16;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicU16 = loom::sync::atomic::AtomicU16;

/// Behaves like [`core::sync::atomic::AtomicU32`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicU32 = core::sync::atomic::AtomicU32;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicU32 = loom::sync::atomic::AtomicU32;

/// Behaves like [`core::sync::atomic::AtomicI8`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicI8 = core::sync::atomic::AtomicI8;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicI8 = loom::sync::atomic::AtomicI8;

/// Behaves like [`core::sync::atomic::AtomicI16`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicI16 = core::sync::atomic::AtomicI16;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicI16 = loom::sync::atomic::AtomicI16;

/// Behaves like [`core::sync::atomic::AtomicI32`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicI32 = core::sync::atomic::AtomicI32;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicI32 = loom::sync::atomic::AtomicI32;

/// Behaves like [`core::sync::atomic::AtomicI64`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicI64 = core::sync::atomic::AtomicI64;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicI64 = loom::sync::atomic::AtomicI64;

/// Behaves like [`core::sync::atomic::AtomicU64`]
#[cfg(not(all(test, loom, feature = "std")))]
#[allow(clippy::disallowed_types)]
pub type AtomicU64 = core::sync::atomic::AtomicU64;

#[cfg(all(test, loom, feature = "std"))]
pub type AtomicU64 = loom::sync::atomic::AtomicU64;

type LockType = RwLockWriterPreference;

#[doc(hidden)]
pub mod internal {
    use core::ops::BitAnd;

    use super::*;

    pub trait AtomicInteger:
        Copy
        + Default
        + Send
        + Eq
        + AddAssign
        + SubAssign
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + BitAnd<Output = Self>
        + Ord
        + Not<Output = Self>
        + core::fmt::Debug
    {
        fn overflowing_add(self, rhs: Self) -> (Self, bool);
        fn overflowing_sub(self, rhs: Self) -> (Self, bool);
    }

    impl AtomicInteger for u64 {
        fn overflowing_add(self, rhs: Self) -> (Self, bool) {
            self.overflowing_add(rhs)
        }

        fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
            self.overflowing_sub(rhs)
        }
    }

    impl AtomicInteger for u128 {
        fn overflowing_add(self, rhs: Self) -> (Self, bool) {
            self.overflowing_add(rhs)
        }

        fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
            self.overflowing_sub(rhs)
        }
    }

    impl AtomicInteger for i64 {
        fn overflowing_add(self, rhs: Self) -> (Self, bool) {
            self.overflowing_add(rhs)
        }

        fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
            self.overflowing_sub(rhs)
        }
    }

    impl AtomicInteger for i128 {
        fn overflowing_add(self, rhs: Self) -> (Self, bool) {
            self.overflowing_add(rhs)
        }

        fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
            self.overflowing_sub(rhs)
        }
    }
}

/// iceoryx2 implementation of an atomic that has an internal [`RwLockWriterPreference`].
/// It enables atomic operations on platforms that do not support them with the restriction that
/// those operations are no longer lock-free.
#[derive(Default)]
#[repr(C)]
pub struct Atomic<T: internal::AtomicInteger> {
    data: UnsafeCell<T>,
    lock: LockType,
}

unsafe impl<T: internal::AtomicInteger> Send for Atomic<T> {}
unsafe impl<T: internal::AtomicInteger> Sync for Atomic<T> {}

impl<T: internal::AtomicInteger> Debug for Atomic<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Atomic<{}> {{ value: {:?} }}",
            core::any::type_name::<T>(),
            self.load(Ordering::Relaxed),
        )
    }
}

impl<T: internal::AtomicInteger> Atomic<T> {
    /// See [`core::sync::atomic::AtomicU64::new()`]
    #[cfg(not(all(test, loom, feature = "std")))]
    pub const fn new(v: T) -> Self {
        Self {
            data: UnsafeCell::new(v),
            lock: LockType::new(),
        }
    }

    /// See [`core::sync::atomic::AtomicU64::new()`]
    #[cfg(all(test, loom, feature = "std"))]
    pub fn new(v: T) -> Self {
        Self {
            data: UnsafeCell::new(v),
            lock: LockType::new(),
        }
    }

    #[cfg(not(all(test, loom, feature = "std")))]
    #[inline(always)]
    fn get_ptr(&self) -> *mut T {
        self.data.get()
    }

    #[cfg(all(test, loom, feature = "std"))]
    #[inline(always)]
    fn get_ptr(&self) -> *mut T {
        unsafe { self.data.get().deref() as *const T as *mut T }
    }

    fn read_lock(&self) {
        self.lock.read_lock(|_, _| WaitAction::Continue);
    }

    fn write_lock(&self) {
        self.lock
            .write_lock(|_, _| WaitAction::Continue, |_| {}, |_| {});
    }

    fn unlock(&self) {
        self.lock.unlock(|_| {}, |_| {});
    }

    /// See [`core::sync::atomic::AtomicU64::as_ptr()`]
    #[cfg(not(all(test, loom, feature = "std")))]
    pub const fn as_ptr(&self) -> *mut T {
        self.data.get()
    }

    /// See [`core::sync::atomic::AtomicU64::as_ptr()`]
    #[cfg(all(test, loom, feature = "std"))]
    pub fn as_ptr(&self) -> *mut T {
        self.get_ptr()
    }

    /// See [`core::sync::atomic::AtomicU64::compare_exchange()`]
    pub fn compare_exchange(
        &self,
        current: T,
        new: T,
        _success: Ordering,
        _failure: Ordering,
    ) -> Result<T, T> {
        self.write_lock();
        let data = unsafe { *self.get_ptr() };
        if data != current {
            fence(Ordering::SeqCst);
            self.unlock();
            return Err(data);
        }

        unsafe { *self.get_ptr() = new };
        fence(Ordering::SeqCst);
        self.unlock();
        Ok(data)
    }

    /// See [`core::sync::atomic::AtomicU64::compare_exchange_weak()`]
    pub fn compare_exchange_weak(
        &self,
        current: T,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<T, T> {
        self.compare_exchange(current, new, success, failure)
    }

    fn fetch_op<F: FnOnce() -> T>(&self, op: F, _order: Ordering) -> T {
        self.write_lock();
        let data = op();
        fence(Ordering::SeqCst);
        self.unlock();
        data
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_add()`]
    pub fn fetch_add(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() = old.overflowing_add(value).0 };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_and()`]
    pub fn fetch_and(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() &= value };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_max()`]
    pub fn fetch_max(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() = old.max(value) };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_min()`]
    pub fn fetch_min(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() = old.min(value) };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_nand()`]
    pub fn fetch_nand(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() = !(old & value) };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_or()`]
    pub fn fetch_or(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() |= value };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_sub()`]
    pub fn fetch_sub(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() = old.overflowing_sub(value).0 };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_update()`]
    pub fn fetch_update<F: FnMut(T) -> Option<T>>(
        &self,
        _set_order: Ordering,
        _fetch_order: Ordering,
        mut f: F,
    ) -> Result<T, T> {
        self.write_lock();
        let data = unsafe { *self.get_ptr() };

        match f(data) {
            Some(v) => {
                unsafe { *self.get_ptr() = v };
                fence(Ordering::SeqCst);
                self.unlock();
                Ok(data)
            }
            None => {
                fence(Ordering::SeqCst);
                self.unlock();
                Err(data)
            }
        }
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_xor()`]
    pub fn fetch_xor(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.get_ptr() };
                unsafe { *self.get_ptr() ^= value };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::into_inner()`]
    pub fn into_inner(self) -> T {
        unsafe { *self.get_ptr() }
    }

    /// See [`core::sync::atomic::AtomicU64::load()`]
    pub fn load(&self, _order: Ordering) -> T {
        self.read_lock();
        let data = unsafe { *self.get_ptr() };
        fence(Ordering::SeqCst);
        self.unlock();
        data
    }

    /// See [`core::sync::atomic::AtomicU64::store()`]
    pub fn store(&self, value: T, _order: Ordering) {
        self.write_lock();
        unsafe { *self.get_ptr() = value };
        fence(Ordering::SeqCst);
        self.unlock();
    }

    /// See [`core::sync::atomic::AtomicU64::swap()`]
    pub fn swap(&self, value: T, _order: Ordering) -> T {
        self.write_lock();
        let data = unsafe { *self.get_ptr() };
        unsafe { *self.get_ptr() = value };
        fence(Ordering::SeqCst);
        self.unlock();
        data
    }
}
