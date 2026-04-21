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

/// Marks types that can be created by zeroing all bytes. Usually used for
/// low-level POSIX structs in C where the struct is initialized by calling
/// `memset(&some_struct, 0, sizeof(some_struct))`.
///
/// # Safety
///
/// 1. The user must ensure that an all-zero bit pattern is valid.
///
pub unsafe trait Zeroable: core::marker::Sized {
    /// Creates a new zerod value.
    fn new_zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }

    #[doc(hidden)]
    /// used as dummy call in the derive macro to ensure at compile-time that all fields of
    /// a struct implement Zeroable
    fn __is_zeroable(&self) {}
}

unsafe impl Zeroable for usize {}
unsafe impl Zeroable for u8 {}
unsafe impl Zeroable for u16 {}
unsafe impl Zeroable for u32 {}
unsafe impl Zeroable for u64 {}
unsafe impl Zeroable for u128 {}

unsafe impl Zeroable for isize {}
unsafe impl Zeroable for i8 {}
unsafe impl Zeroable for i16 {}
unsafe impl Zeroable for i32 {}
unsafe impl Zeroable for i64 {}
unsafe impl Zeroable for i128 {}

unsafe impl Zeroable for f32 {}
unsafe impl Zeroable for f64 {}

unsafe impl Zeroable for char {}
unsafe impl Zeroable for bool {}

unsafe impl Zeroable for () {}

unsafe impl<T: Zeroable, const N: usize> Zeroable for [T; N] {}
unsafe impl<T: Zeroable> Zeroable for core::mem::MaybeUninit<T> {}

unsafe impl<T1: Zeroable, T2: Zeroable> Zeroable for (T1, T2) {}
unsafe impl<T1: Zeroable, T2: Zeroable, T3: Zeroable> Zeroable for (T1, T2, T3) {}
unsafe impl<T1: Zeroable, T2: Zeroable, T3: Zeroable, T4: Zeroable> Zeroable for (T1, T2, T3, T4) {}
unsafe impl<T1: Zeroable, T2: Zeroable, T3: Zeroable, T4: Zeroable, T5: Zeroable> Zeroable
    for (T1, T2, T3, T4, T5)
{
}
unsafe impl<T1: Zeroable, T2: Zeroable, T3: Zeroable, T4: Zeroable, T5: Zeroable, T6: Zeroable>
    Zeroable for (T1, T2, T3, T4, T5, T6)
{
}
unsafe impl<
    T1: Zeroable,
    T2: Zeroable,
    T3: Zeroable,
    T4: Zeroable,
    T5: Zeroable,
    T6: Zeroable,
    T7: Zeroable,
> Zeroable for (T1, T2, T3, T4, T5, T6, T7)
{
}
unsafe impl<
    T1: Zeroable,
    T2: Zeroable,
    T3: Zeroable,
    T4: Zeroable,
    T5: Zeroable,
    T6: Zeroable,
    T7: Zeroable,
    T8: Zeroable,
> Zeroable for (T1, T2, T3, T4, T5, T6, T7, T8)
{
}
