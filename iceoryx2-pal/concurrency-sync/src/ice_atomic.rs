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
    ops::{AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, Not, SubAssign},
    sync::atomic::Ordering,
};

use crate::{rwlock::RwLockWriterPreference, WaitAction};

pub type IceAtomicBool = core::sync::atomic::AtomicBool;
pub type IceAtomicUsize = core::sync::atomic::AtomicUsize;

pub type IceAtomicU8 = core::sync::atomic::AtomicU8;
pub type IceAtomicU16 = core::sync::atomic::AtomicU16;
pub type IceAtomicU32 = core::sync::atomic::AtomicU32;
pub type IceAtomicI8 = core::sync::atomic::AtomicI8;
pub type IceAtomicI16 = core::sync::atomic::AtomicI16;
pub type IceAtomicI32 = core::sync::atomic::AtomicI32;

#[cfg(target_pointer_width = "64")]
pub type IceAtomicI64 = core::sync::atomic::AtomicI64;

#[cfg(target_pointer_width = "64")]
pub type IceAtomicU64 = core::sync::atomic::AtomicU64;

#[cfg(target_pointer_width = "32")]
pub type IceAtomicI64 = IceAtomic<i64>;

#[cfg(target_pointer_width = "32")]
pub type IceAtomicU64 = IceAtomic<u64>;

type LockType = RwLockWriterPreference;

pub mod internal {
    use super::*;

    pub trait AtomicInteger:
        Copy
        + Send
        + Eq
        + AddAssign
        + SubAssign
        + BitAndAssign
        + BitOrAssign
        + BitXorAssign
        + Ord
        + Not<Output = Self>
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

#[repr(C)]
pub struct IceAtomic<T: internal::AtomicInteger> {
    data: UnsafeCell<T>,
    lock: LockType,
}

impl<T: internal::AtomicInteger> IceAtomic<T> {
    pub fn new(v: T) -> Self {
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

    pub const fn as_ptr(&self) -> *mut T {
        self.data.get()
    }

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

    pub fn fetch_nand(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() &= !value };
                old
            },
            order,
        )
    }

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

    pub fn fetch_update(&self, value: T, order: Ordering) -> T {
        self.fetch_op(
            || {
                let old = unsafe { *self.data.get() };
                unsafe { *self.data.get() = value };
                old
            },
            order,
        )
    }

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

    pub fn into_inner(self) -> T {
        unsafe { *self.data.get() }
    }

    pub fn load(&self, _order: Ordering) -> T {
        self.read_lock();
        let data = unsafe { *self.data.get() };
        core::sync::atomic::fence(Ordering::SeqCst);
        self.unlock();
        data
    }

    pub fn store(&self, value: T, _order: Ordering) {
        self.write_lock();
        unsafe { *self.data.get() = value };
        core::sync::atomic::fence(Ordering::SeqCst);
        self.unlock();
    }

    pub fn swap(&self, value: T, _order: Ordering) -> T {
        self.write_lock();
        let data = unsafe { *self.data.get() };
        unsafe { *self.data.get() = value };
        core::sync::atomic::fence(Ordering::SeqCst);
        self.unlock();
        data
    }
}
