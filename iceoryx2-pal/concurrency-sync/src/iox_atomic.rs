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
    cell::UnsafeCell,
    fmt::Debug,
    ops::{AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, Not, SubAssign},
    sync::atomic::Ordering,
};

use crate::{rwlock::RwLockWriterPreference, WaitAction};

/// Behaves like [`core::sync::atomic::AtomicBool`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicBool = core::sync::atomic::AtomicBool;

/// Behaves like [`core::sync::atomic::AtomicUsize`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicUsize = core::sync::atomic::AtomicUsize;

/// Behaves like [`core::sync::atomic::AtomicIsize`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicIsize = core::sync::atomic::AtomicIsize;

/// Behaves like [`core::sync::atomic::AtomicU8`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicU8 = core::sync::atomic::AtomicU8;

/// Behaves like [`core::sync::atomic::AtomicU16`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicU16 = core::sync::atomic::AtomicU16;

/// Behaves like [`core::sync::atomic::AtomicU32`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicU32 = core::sync::atomic::AtomicU32;

/// Behaves like [`core::sync::atomic::AtomicI8`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicI8 = core::sync::atomic::AtomicI8;

/// Behaves like [`core::sync::atomic::AtomicI16`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicI16 = core::sync::atomic::AtomicI16;

/// Behaves like [`core::sync::atomic::AtomicI32`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicI32 = core::sync::atomic::AtomicI32;

/// Behaves like [`core::sync::atomic::AtomicI64`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicI64 = core::sync::atomic::AtomicI64;

/// Behaves like [`core::sync::atomic::AtomicU64`]
#[allow(clippy::disallowed_types)]
pub type IoxAtomicU64 = core::sync::atomic::AtomicU64;

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
pub struct IoxAtomic<T: internal::AtomicInteger> {
    data: UnsafeCell<T>,
    lock: LockType,
}

unsafe impl<T: internal::AtomicInteger> Send for IoxAtomic<T> {}
unsafe impl<T: internal::AtomicInteger> Sync for IoxAtomic<T> {}

impl<T: internal::AtomicInteger> Debug for IoxAtomic<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "IoxAtomic<{}> {{ value: {:?} }}",
            core::any::type_name::<T>(),
            self.load(Ordering::Relaxed),
        )
    }
}

impl<T: internal::AtomicInteger> IoxAtomic<T> {
    /// See [`core::sync::atomic::AtomicU64::new()`]
    pub const fn new(v: T) -> Self {
        Self {
            data: UnsafeCell::new(v),
            lock: LockType::new(),
        }
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
    pub const fn as_ptr(&self) -> *mut T {
        self.data.get()
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
        let data = unsafe { *self.data.get() };
        if data != current {
            core::sync::atomic::fence(Ordering::SeqCst);
            self.unlock();
            return Err(data);
        }

        unsafe { *self.data.get() = new };
        core::sync::atomic::fence(Ordering::SeqCst);
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
        core::sync::atomic::fence(Ordering::SeqCst);
        self.unlock();
        data
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_add()`]
    pub fn fetch_add(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() = old.overflowing_add(value).0 };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_and()`]
    pub fn fetch_and(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() &= value };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_max()`]
    pub fn fetch_max(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() = old.max(value) };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_min()`]
    pub fn fetch_min(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() = old.min(value) };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_nand()`]
    pub fn fetch_nand(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() = !(old & value) };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_or()`]
    pub fn fetch_or(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() |= value };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_sub()`]
    pub fn fetch_sub(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() = old.overflowing_sub(value).0 };
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
        let data = unsafe { *self.data.get() };

        match f(data) {
            Some(v) => {
                unsafe { *self.data.get() = v };
                core::sync::atomic::fence(Ordering::SeqCst);
                self.unlock();
                Ok(data)
            }
            None => {
                core::sync::atomic::fence(Ordering::SeqCst);
                self.unlock();
                Err(data)
            }
        }
    }

    /// See [`core::sync::atomic::AtomicU64::fetch_xor()`]
    pub fn fetch_xor(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() ^= value };
                old
            },
            order,
        )
    }

    /// See [`core::sync::atomic::AtomicU64::into_inner()`]
    pub fn into_inner(self) -> T {
        unsafe { *self.data.get() }
    }

    /// See [`core::sync::atomic::AtomicU64::load()`]
    pub fn load(&self, _order: Ordering) -> T {
        self.read_lock();
        let data = unsafe { *self.data.get() };
        core::sync::atomic::fence(Ordering::SeqCst);
        self.unlock();
        data
    }

    /// See [`core::sync::atomic::AtomicU64::store()`]
    pub fn store(&self, value: T, _order: Ordering) {
        self.write_lock();
        unsafe { *self.data.get() = value };
        core::sync::atomic::fence(Ordering::SeqCst);
        self.unlock();
    }

    /// See [`core::sync::atomic::AtomicU64::swap()`]
    pub fn swap(&self, value: T, _order: Ordering) -> T {
        self.write_lock();
        let data = unsafe { *self.data.get() };
        unsafe { *self.data.get() = value };
        core::sync::atomic::fence(Ordering::SeqCst);
        self.unlock();
        data
    }
}
