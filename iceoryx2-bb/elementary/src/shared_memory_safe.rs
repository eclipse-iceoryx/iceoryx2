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
/// * The types must be self-contained, meaning they shall not contain pointer, references,
///   indices or handles that are identifying resources outside of the struct.
///   Examples:
///      * File descriptors point to resources that can be different in another process.
///      * A list with pointers into the heap.
/// * The type must have a uniform memory representation, meaning they are annotated with
///   `#[repr(C)]`
/// * The type does not have references or pointer members. It shall not use pointers to manage
///   its internal structure.
///   Examples:
///      * A list must be implemented using indices to structure itself.
pub unsafe trait SharedMemorySafe {
    fn type_name() -> &'static str {
        core::any::type_name::<Self>()
    }
}

unsafe impl SharedMemorySafe for usize {}
unsafe impl SharedMemorySafe for u8 {}
unsafe impl SharedMemorySafe for u16 {}
unsafe impl SharedMemorySafe for u32 {}
unsafe impl SharedMemorySafe for u64 {}
unsafe impl SharedMemorySafe for u128 {}

unsafe impl SharedMemorySafe for isize {}
unsafe impl SharedMemorySafe for i8 {}
unsafe impl SharedMemorySafe for i16 {}
unsafe impl SharedMemorySafe for i32 {}
unsafe impl SharedMemorySafe for i64 {}
unsafe impl SharedMemorySafe for i128 {}

unsafe impl SharedMemorySafe for f32 {}
unsafe impl SharedMemorySafe for f64 {}

unsafe impl SharedMemorySafe for char {}
unsafe impl SharedMemorySafe for bool {}

unsafe impl SharedMemorySafe for IoxAtomicUsize {}
unsafe impl SharedMemorySafe for IoxAtomicU8 {}
unsafe impl SharedMemorySafe for IoxAtomicU16 {}
unsafe impl SharedMemorySafe for IoxAtomicU32 {}
unsafe impl SharedMemorySafe for IoxAtomicU64 {}

unsafe impl SharedMemorySafe for IoxAtomicIsize {}
unsafe impl SharedMemorySafe for IoxAtomicI8 {}
unsafe impl SharedMemorySafe for IoxAtomicI16 {}
unsafe impl SharedMemorySafe for IoxAtomicI32 {}
unsafe impl SharedMemorySafe for IoxAtomicI64 {}

unsafe impl<T: SharedMemorySafe> SharedMemorySafe for [T] {}
unsafe impl<T: SharedMemorySafe, const N: usize> SharedMemorySafe for [T; N] {}
unsafe impl<T: SharedMemorySafe> SharedMemorySafe for Option<T> {}
unsafe impl<T: SharedMemorySafe, E: SharedMemorySafe> SharedMemorySafe for Result<T, E> {}
unsafe impl<T: SharedMemorySafe> SharedMemorySafe for core::mem::MaybeUninit<T> {}
unsafe impl<T: SharedMemorySafe> SharedMemorySafe for core::cell::UnsafeCell<T> {}

unsafe impl<T1: SharedMemorySafe, T2: SharedMemorySafe> SharedMemorySafe for (T1, T2) {}
unsafe impl<T1: SharedMemorySafe, T2: SharedMemorySafe, T3: SharedMemorySafe> SharedMemorySafe
    for (T1, T2, T3)
{
}
unsafe impl<T1: SharedMemorySafe, T2: SharedMemorySafe, T3: SharedMemorySafe, T4: SharedMemorySafe>
    SharedMemorySafe for (T1, T2, T3, T4)
{
}
unsafe impl<
        T1: SharedMemorySafe,
        T2: SharedMemorySafe,
        T3: SharedMemorySafe,
        T4: SharedMemorySafe,
        T5: SharedMemorySafe,
    > SharedMemorySafe for (T1, T2, T3, T4, T5)
{
}
unsafe impl<
        T1: SharedMemorySafe,
        T2: SharedMemorySafe,
        T3: SharedMemorySafe,
        T4: SharedMemorySafe,
        T5: SharedMemorySafe,
        T6: SharedMemorySafe,
    > SharedMemorySafe for (T1, T2, T3, T4, T5, T6)
{
}
unsafe impl<
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
unsafe impl<
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
