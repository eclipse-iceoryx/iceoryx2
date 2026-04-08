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

use crate::{zero_copy_send::ZeroCopySend, zeroable::Zeroable};

/// Marks types as plain old data without any padding.
///
/// # Safety
///
/// 1. Any bit pattern must be valid for the type.
/// 2. The type must not have any padding between members or at the end.
/// 3. The type must have a C representation, `#[repr(C)]`.
/// 4. All members must implement [`PlainOldData`]
///
pub unsafe trait PlainOldData: ZeroCopySend + Zeroable + Copy + 'static {}

unsafe impl PlainOldData for usize {}
unsafe impl PlainOldData for u8 {}
unsafe impl PlainOldData for u16 {}
unsafe impl PlainOldData for u32 {}
unsafe impl PlainOldData for u64 {}
unsafe impl PlainOldData for u128 {}

unsafe impl PlainOldData for isize {}
unsafe impl PlainOldData for i8 {}
unsafe impl PlainOldData for i16 {}
unsafe impl PlainOldData for i32 {}
unsafe impl PlainOldData for i64 {}
unsafe impl PlainOldData for i128 {}

unsafe impl PlainOldData for f32 {}
unsafe impl PlainOldData for f64 {}

unsafe impl PlainOldData for () {}

unsafe impl<T: PlainOldData, const N: usize> PlainOldData for [T; N] {}
unsafe impl<T: PlainOldData> PlainOldData for core::mem::MaybeUninit<T> {}

unsafe impl<T1: PlainOldData, T2: PlainOldData> PlainOldData for (T1, T2) {}
unsafe impl<T1: PlainOldData, T2: PlainOldData, T3: PlainOldData> PlainOldData for (T1, T2, T3) {}
unsafe impl<T1: PlainOldData, T2: PlainOldData, T3: PlainOldData, T4: PlainOldData> PlainOldData
    for (T1, T2, T3, T4)
{
}
unsafe impl<T1: PlainOldData, T2: PlainOldData, T3: PlainOldData, T4: PlainOldData, T5: PlainOldData>
    PlainOldData for (T1, T2, T3, T4, T5)
{
}
unsafe impl<
        T1: PlainOldData,
        T2: PlainOldData,
        T3: PlainOldData,
        T4: PlainOldData,
        T5: PlainOldData,
        T6: PlainOldData,
    > PlainOldData for (T1, T2, T3, T4, T5, T6)
{
}
unsafe impl<
        T1: PlainOldData,
        T2: PlainOldData,
        T3: PlainOldData,
        T4: PlainOldData,
        T5: PlainOldData,
        T6: PlainOldData,
        T7: PlainOldData,
    > PlainOldData for (T1, T2, T3, T4, T5, T6, T7)
{
}
unsafe impl<
        T1: PlainOldData,
        T2: PlainOldData,
        T3: PlainOldData,
        T4: PlainOldData,
        T5: PlainOldData,
        T6: PlainOldData,
        T7: PlainOldData,
        T8: PlainOldData,
    > PlainOldData for (T1, T2, T3, T4, T5, T6, T7, T8)
{
}
