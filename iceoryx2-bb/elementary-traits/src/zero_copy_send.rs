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

/// Marks types that can be sent to another process in a zero-copy manner, i.e. the types can be
/// safely used from within different process address spaces and can be uniquely identified by their
/// [`ZeroCopySend::type_name()`] in an inter-process communication context.
///
/// # Safety
///
/// The user must ensure that
///  * the types are self-contained, meaning they shall not contain pointers, references, indices
///    or handles to resources that are not part of the type.
///    Examples:
///       * file descriptors point to resources that can be different in another process
///       * a list with pointers into the heap
///  * the types do not have references or pointer members; they shall not use pointers to manage
///    their internal structure.
///    Example:
///       * a list must be implemented using indices to structure itself
///  * the types must have a uniform memory representation, meaning they are annotated with
///    `#[repr(C)]`.
///
pub unsafe trait ZeroCopySend {
    /// The unique identifier of the type. It shall be used to identify a specific type across
    /// processes and languages.
    ///
    /// # Safety
    ///
    ///  * The user must guarantee that all types, also definitions in different languages, have
    ///    the same memory layout.
    unsafe fn type_name() -> &'static str {
        core::any::type_name::<Self>()
    }

    #[doc(hidden)]
    /// used as dummy call in the derive macro to ensure at compile-time that all fields of
    /// a struct implement ZeroCopySend
    fn __is_zero_copy_send(&self) {}
}

unsafe impl ZeroCopySend for usize {}
unsafe impl ZeroCopySend for u8 {}
unsafe impl ZeroCopySend for u16 {}
unsafe impl ZeroCopySend for u32 {}
unsafe impl ZeroCopySend for u64 {}
unsafe impl ZeroCopySend for u128 {}

unsafe impl ZeroCopySend for isize {}
unsafe impl ZeroCopySend for i8 {}
unsafe impl ZeroCopySend for i16 {}
unsafe impl ZeroCopySend for i32 {}
unsafe impl ZeroCopySend for i64 {}
unsafe impl ZeroCopySend for i128 {}

unsafe impl ZeroCopySend for f32 {}
unsafe impl ZeroCopySend for f64 {}

unsafe impl ZeroCopySend for char {}
unsafe impl ZeroCopySend for bool {}

unsafe impl ZeroCopySend for IoxAtomicUsize {}
unsafe impl ZeroCopySend for IoxAtomicU8 {}
unsafe impl ZeroCopySend for IoxAtomicU16 {}
unsafe impl ZeroCopySend for IoxAtomicU32 {}
unsafe impl ZeroCopySend for IoxAtomicU64 {}

unsafe impl ZeroCopySend for IoxAtomicIsize {}
unsafe impl ZeroCopySend for IoxAtomicI8 {}
unsafe impl ZeroCopySend for IoxAtomicI16 {}
unsafe impl ZeroCopySend for IoxAtomicI32 {}
unsafe impl ZeroCopySend for IoxAtomicI64 {}

unsafe impl ZeroCopySend for () {}

unsafe impl ZeroCopySend for core::time::Duration {}

unsafe impl<T: ZeroCopySend> ZeroCopySend for [T] {}
unsafe impl<T: ZeroCopySend, const N: usize> ZeroCopySend for [T; N] {}
unsafe impl<T: ZeroCopySend> ZeroCopySend for Option<T> {}
unsafe impl<T: ZeroCopySend, E: ZeroCopySend> ZeroCopySend for Result<T, E> {}
unsafe impl<T: ZeroCopySend> ZeroCopySend for core::mem::MaybeUninit<T> {}
unsafe impl<T: ZeroCopySend> ZeroCopySend for core::cell::UnsafeCell<T> {}

unsafe impl<T1: ZeroCopySend, T2: ZeroCopySend> ZeroCopySend for (T1, T2) {}
unsafe impl<T1: ZeroCopySend, T2: ZeroCopySend, T3: ZeroCopySend> ZeroCopySend for (T1, T2, T3) {}
unsafe impl<T1: ZeroCopySend, T2: ZeroCopySend, T3: ZeroCopySend, T4: ZeroCopySend> ZeroCopySend
    for (T1, T2, T3, T4)
{
}
unsafe impl<T1: ZeroCopySend, T2: ZeroCopySend, T3: ZeroCopySend, T4: ZeroCopySend, T5: ZeroCopySend>
    ZeroCopySend for (T1, T2, T3, T4, T5)
{
}
unsafe impl<
        T1: ZeroCopySend,
        T2: ZeroCopySend,
        T3: ZeroCopySend,
        T4: ZeroCopySend,
        T5: ZeroCopySend,
        T6: ZeroCopySend,
    > ZeroCopySend for (T1, T2, T3, T4, T5, T6)
{
}
unsafe impl<
        T1: ZeroCopySend,
        T2: ZeroCopySend,
        T3: ZeroCopySend,
        T4: ZeroCopySend,
        T5: ZeroCopySend,
        T6: ZeroCopySend,
        T7: ZeroCopySend,
    > ZeroCopySend for (T1, T2, T3, T4, T5, T6, T7)
{
}
unsafe impl<
        T1: ZeroCopySend,
        T2: ZeroCopySend,
        T3: ZeroCopySend,
        T4: ZeroCopySend,
        T5: ZeroCopySend,
        T6: ZeroCopySend,
        T7: ZeroCopySend,
        T8: ZeroCopySend,
    > ZeroCopySend for (T1, T2, T3, T4, T5, T6, T7, T8)
{
}
