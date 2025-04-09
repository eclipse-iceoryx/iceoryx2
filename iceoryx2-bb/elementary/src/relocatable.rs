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

/// Marks types that can be safely used from within different process address spaces.
///
/// # Safety
///
///  * The types must be self-contained, meaning they shall not contain pointer, references,
///    indices or handles that are identifying resources outside of the struct.
///    Examples:
///       * File descriptors point to resources that can be different in another process.
///       * A list with pointers into the heap.
///  * The type must have a uniform memory representation, meaning they are annotated with
///    `#[repr(C)]`
///  * The type does not have references or pointer members. It shall not use pointers to manage
///    its internal structure.
///    Examples:
///       * A list must be implemented using indices to structure itself.
pub unsafe trait Relocatable {
    #[doc(hidden)]
    // dummy function to check if a type is relocatable for ZeroCopySend derive macro
    unsafe fn _is_relocatable(&self) {}
}

unsafe impl Relocatable for usize {}
unsafe impl Relocatable for u8 {}
unsafe impl Relocatable for u16 {}
unsafe impl Relocatable for u32 {}
unsafe impl Relocatable for u64 {}
unsafe impl Relocatable for u128 {}

unsafe impl Relocatable for isize {}
unsafe impl Relocatable for i8 {}
unsafe impl Relocatable for i16 {}
unsafe impl Relocatable for i32 {}
unsafe impl Relocatable for i64 {}
unsafe impl Relocatable for i128 {}

unsafe impl Relocatable for f32 {}
unsafe impl Relocatable for f64 {}

unsafe impl Relocatable for char {}
unsafe impl Relocatable for bool {}

unsafe impl Relocatable for IoxAtomicUsize {}
unsafe impl Relocatable for IoxAtomicU8 {}
unsafe impl Relocatable for IoxAtomicU16 {}
unsafe impl Relocatable for IoxAtomicU32 {}
unsafe impl Relocatable for IoxAtomicU64 {}

unsafe impl Relocatable for IoxAtomicIsize {}
unsafe impl Relocatable for IoxAtomicI8 {}
unsafe impl Relocatable for IoxAtomicI16 {}
unsafe impl Relocatable for IoxAtomicI32 {}
unsafe impl Relocatable for IoxAtomicI64 {}

unsafe impl<T: Relocatable> Relocatable for [T] {}
unsafe impl<T: Relocatable, const N: usize> Relocatable for [T; N] {}
unsafe impl<T: Relocatable> Relocatable for Option<T> {}
unsafe impl<T: Relocatable, E: Relocatable> Relocatable for Result<T, E> {}
unsafe impl<T: Relocatable> Relocatable for core::mem::MaybeUninit<T> {}
unsafe impl<T: Relocatable> Relocatable for core::cell::UnsafeCell<T> {}

unsafe impl<T1: Relocatable, T2: Relocatable> Relocatable for (T1, T2) {}
unsafe impl<T1: Relocatable, T2: Relocatable, T3: Relocatable> Relocatable for (T1, T2, T3) {}
unsafe impl<T1: Relocatable, T2: Relocatable, T3: Relocatable, T4: Relocatable> Relocatable
    for (T1, T2, T3, T4)
{
}
unsafe impl<T1: Relocatable, T2: Relocatable, T3: Relocatable, T4: Relocatable, T5: Relocatable>
    Relocatable for (T1, T2, T3, T4, T5)
{
}
unsafe impl<
        T1: Relocatable,
        T2: Relocatable,
        T3: Relocatable,
        T4: Relocatable,
        T5: Relocatable,
        T6: Relocatable,
    > Relocatable for (T1, T2, T3, T4, T5, T6)
{
}
unsafe impl<
        T1: Relocatable,
        T2: Relocatable,
        T3: Relocatable,
        T4: Relocatable,
        T5: Relocatable,
        T6: Relocatable,
        T7: Relocatable,
    > Relocatable for (T1, T2, T3, T4, T5, T6, T7)
{
}
unsafe impl<
        T1: Relocatable,
        T2: Relocatable,
        T3: Relocatable,
        T4: Relocatable,
        T5: Relocatable,
        T6: Relocatable,
        T7: Relocatable,
        T8: Relocatable,
    > Relocatable for (T1, T2, T3, T4, T5, T6, T7, T8)
{
}
