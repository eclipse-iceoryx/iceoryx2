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

//! Trait to perform placement new construction on a given pointer via [`Default::default()`].
//! See [`PlacementDefault`] for example.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

use iceoryx2_pal_concurrency_sync::iox_atomic::*;

/// A trait that allows types to perform a placement new based on their
/// [`Default::default()`] implementation. This can be used to avoid additional
/// copies when a type must be written into a specific memory location.
///
/// ```
/// use core::alloc::Layout;
/// extern crate alloc;
/// use alloc::alloc::{alloc, dealloc};
/// use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
///
/// struct MyLargeType {
///     value_1: [u64; 10485760],
///     value_2: [u64; 10485760]
/// }
///
/// impl PlacementDefault for MyLargeType {
///     unsafe fn placement_default(ptr: *mut Self) {
///         let value_1_ptr = core::ptr::addr_of_mut!((*ptr).value_1);
///         let value_2_ptr = core::ptr::addr_of_mut!((*ptr).value_2);
///
///         PlacementDefault::placement_default(value_1_ptr);
///         PlacementDefault::placement_default(value_2_ptr);
///     }
/// }
///
/// let layout = Layout::new::<MyLargeType>();
/// let raw_memory = unsafe { alloc(layout) } as *mut MyLargeType;
/// unsafe { MyLargeType::placement_default(raw_memory) };
///
/// unsafe { &mut *raw_memory }.value_1[123] = 456;
///
/// unsafe { core::ptr::drop_in_place(raw_memory) };
/// unsafe { dealloc(raw_memory.cast(), layout) };
/// ```
pub trait PlacementDefault {
    /// Performs a initialization of Self at the provided memory position
    /// with [`Default::default()`].
    ///
    /// # Safety
    ///
    ///  * ptr must have at least the alignment of Self
    ///  * ptr must point to a memory location with at least the size of Size
    ///  * ptr must point to a valid memory location
    ///  * shall not be called on already initialized memory
    unsafe fn placement_default(ptr: *mut Self);
}

macro_rules! Impl {
    ($type:ty) => {
        impl PlacementDefault for $type {
            unsafe fn placement_default(ptr: *mut Self) {
                ptr.write(<$type>::default())
            }
        }
    };
}

Impl!(f32);
Impl!(f64);
Impl!(u8);
Impl!(u16);
Impl!(u32);
Impl!(u64);
Impl!(u128);
Impl!(i8);
Impl!(i16);
Impl!(i32);
Impl!(i64);
Impl!(i128);
Impl!(isize);
Impl!(usize);
Impl!(char);
Impl!(bool);
Impl!(IoxAtomicBool);
Impl!(IoxAtomicU8);
Impl!(IoxAtomicU16);
Impl!(IoxAtomicU32);
Impl!(IoxAtomicU64);
Impl!(IoxAtomicI8);
Impl!(IoxAtomicI16);
Impl!(IoxAtomicI32);
Impl!(IoxAtomicI64);
Impl!(IoxAtomicIsize);
Impl!(IoxAtomicUsize);

impl<T: PlacementDefault> PlacementDefault for [T] {
    unsafe fn placement_default(ptr: *mut Self) {
        let ptr = ptr as *mut [MaybeUninit<T>];
        for i in (*ptr).iter_mut() {
            PlacementDefault::placement_default(i.as_mut_ptr());
        }
    }
}

impl<T: PlacementDefault, const CAPACITY: usize> PlacementDefault for [T; CAPACITY] {
    unsafe fn placement_default(ptr: *mut Self) {
        for i in 0..CAPACITY {
            PlacementDefault::placement_default((ptr as *mut T).add(i))
        }
    }
}

impl<T1: PlacementDefault> PlacementDefault for (T1,) {
    unsafe fn placement_default(ptr: *mut Self) {
        let ptr = core::ptr::addr_of_mut!((*ptr).0);
        PlacementDefault::placement_default(ptr)
    }
}

impl<T1: PlacementDefault, T2: PlacementDefault> PlacementDefault for (T1, T2) {
    unsafe fn placement_default(ptr: *mut Self) {
        let elem = core::ptr::addr_of_mut!((*ptr).0);
        PlacementDefault::placement_default(elem);

        let elem = core::ptr::addr_of_mut!((*ptr).1);
        PlacementDefault::placement_default(elem)
    }
}

impl<T1: PlacementDefault, T2: PlacementDefault, T3: PlacementDefault> PlacementDefault
    for (T1, T2, T3)
{
    unsafe fn placement_default(ptr: *mut Self) {
        let elem = core::ptr::addr_of_mut!((*ptr).0);
        PlacementDefault::placement_default(elem);

        let elem = core::ptr::addr_of_mut!((*ptr).1);
        PlacementDefault::placement_default(elem);

        let elem = core::ptr::addr_of_mut!((*ptr).2);
        PlacementDefault::placement_default(elem)
    }
}

impl<T1: PlacementDefault, T2: PlacementDefault, T3: PlacementDefault, T4: PlacementDefault>
    PlacementDefault for (T1, T2, T3, T4)
{
    unsafe fn placement_default(ptr: *mut Self) {
        let elem = core::ptr::addr_of_mut!((*ptr).0);
        PlacementDefault::placement_default(elem);

        let elem = core::ptr::addr_of_mut!((*ptr).1);
        PlacementDefault::placement_default(elem);

        let elem = core::ptr::addr_of_mut!((*ptr).2);
        PlacementDefault::placement_default(elem);

        let elem = core::ptr::addr_of_mut!((*ptr).3);
        PlacementDefault::placement_default(elem)
    }
}

impl<T> PlacementDefault for Option<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(Option::default())
    }
}

impl<T: PlacementDefault> PlacementDefault for core::mem::MaybeUninit<T> {
    unsafe fn placement_default(_ptr: *mut Self) {}
}

impl<T: Default> PlacementDefault for UnsafeCell<T> {
    unsafe fn placement_default(ptr: *mut Self) {
        ptr.write(UnsafeCell::default());
    }
}
