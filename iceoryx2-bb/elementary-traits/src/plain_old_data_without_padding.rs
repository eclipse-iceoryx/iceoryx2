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
/// 4. All members must implement [`PlainOldDataWithoutPadding`]
///
pub unsafe trait PlainOldDataWithoutPadding:
    ZeroCopySend + Zeroable + Copy + 'static
{
}

unsafe impl PlainOldDataWithoutPadding for usize {}
unsafe impl PlainOldDataWithoutPadding for u8 {}
unsafe impl PlainOldDataWithoutPadding for u16 {}
unsafe impl PlainOldDataWithoutPadding for u32 {}
unsafe impl PlainOldDataWithoutPadding for u64 {}
unsafe impl PlainOldDataWithoutPadding for u128 {}

unsafe impl PlainOldDataWithoutPadding for isize {}
unsafe impl PlainOldDataWithoutPadding for i8 {}
unsafe impl PlainOldDataWithoutPadding for i16 {}
unsafe impl PlainOldDataWithoutPadding for i32 {}
unsafe impl PlainOldDataWithoutPadding for i64 {}
unsafe impl PlainOldDataWithoutPadding for i128 {}

unsafe impl PlainOldDataWithoutPadding for f32 {}
unsafe impl PlainOldDataWithoutPadding for f64 {}

unsafe impl<T: PlainOldDataWithoutPadding, const N: usize> PlainOldDataWithoutPadding for [T; N] {}
