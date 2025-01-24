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

use iceoryx2_pal_concurrency_sync::iox_atomic::*;

/// Marks types that can be safely stored in shared memory and consumed from multiple process
/// using different address spaces.
pub trait SharedMemorySafe {
    fn type_name() -> &'static str {
        core::any::type_name::<Self>()
    }
}

impl SharedMemorySafe for u8 {}
impl SharedMemorySafe for u16 {}
impl SharedMemorySafe for u32 {}
impl SharedMemorySafe for u64 {}
impl SharedMemorySafe for u128 {}

impl SharedMemorySafe for i8 {}
impl SharedMemorySafe for i16 {}
impl SharedMemorySafe for i32 {}
impl SharedMemorySafe for i64 {}
impl SharedMemorySafe for i128 {}

impl SharedMemorySafe for f32 {}
impl SharedMemorySafe for f64 {}

impl SharedMemorySafe for char {}
impl SharedMemorySafe for bool {}

impl SharedMemorySafe for IoxAtomicU8 {}
impl SharedMemorySafe for IoxAtomicU16 {}
impl SharedMemorySafe for IoxAtomicU32 {}
impl SharedMemorySafe for IoxAtomicU64 {}

impl SharedMemorySafe for IoxAtomicI8 {}
impl SharedMemorySafe for IoxAtomicI16 {}
impl SharedMemorySafe for IoxAtomicI32 {}
impl SharedMemorySafe for IoxAtomicI64 {}

impl<T: SharedMemorySafe> SharedMemorySafe for [T] {}
impl<T: SharedMemorySafe, const N: usize> SharedMemorySafe for [T; N] {}
impl<T: SharedMemorySafe> SharedMemorySafe for Option<T> {}
impl<T: SharedMemorySafe, E: SharedMemorySafe> SharedMemorySafe for Result<T, E> {}
impl<T: SharedMemorySafe> SharedMemorySafe for core::mem::MaybeUninit<T> {}
impl<T: SharedMemorySafe> SharedMemorySafe for core::cell::UnsafeCell<T> {}

impl<T1: SharedMemorySafe, T2: SharedMemorySafe> SharedMemorySafe for (T1, T2) {}
impl<T1: SharedMemorySafe, T2: SharedMemorySafe, T3: SharedMemorySafe> SharedMemorySafe
    for (T1, T2, T3)
{
}
impl<T1: SharedMemorySafe, T2: SharedMemorySafe, T3: SharedMemorySafe, T4: SharedMemorySafe>
    SharedMemorySafe for (T1, T2, T3, T4)
{
}
impl<
        T1: SharedMemorySafe,
        T2: SharedMemorySafe,
        T3: SharedMemorySafe,
        T4: SharedMemorySafe,
        T5: SharedMemorySafe,
    > SharedMemorySafe for (T1, T2, T3, T4, T5)
{
}
impl<
        T1: SharedMemorySafe,
        T2: SharedMemorySafe,
        T3: SharedMemorySafe,
        T4: SharedMemorySafe,
        T5: SharedMemorySafe,
        T6: SharedMemorySafe,
    > SharedMemorySafe for (T1, T2, T3, T4, T5, T6)
{
}
impl<
        T1: SharedMemorySafe,
        T2: SharedMemorySafe,
        T3: SharedMemorySafe,
        T4: SharedMemorySafe,
        T5: SharedMemorySafe,
        T6: SharedMemorySafe,
        T7: SharedMemorySafe,
    > SharedMemorySafe for (T1, T2, T3, T4, T5, T6, T7)
{
}
impl<
        T1: SharedMemorySafe,
        T2: SharedMemorySafe,
        T3: SharedMemorySafe,
        T4: SharedMemorySafe,
        T5: SharedMemorySafe,
        T6: SharedMemorySafe,
        T7: SharedMemorySafe,
        T8: SharedMemorySafe,
    > SharedMemorySafe for (T1, T2, T3, T4, T5, T6, T7, T8)
{
}
