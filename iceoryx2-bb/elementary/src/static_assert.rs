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

//! Static assertions in Rust.
//!
//! Useful for compile time assertions.
//!
//! # Example
//!
//! ```
//! use iceoryx2_bb_elementary::{
//!     static_assert_eq, static_assert_ge, static_assert_gt, static_assert_le,
//!     static_assert_lt, static_assert_size_of, static_assert_align_of,
//! };
//!
//! use core::mem::{align_of, size_of};
//!
//! static_assert_eq!(size_of::<u64>(), 8);
//! static_assert_ge!(size_of::<u64>(), size_of::<u32>());
//! static_assert_gt!(size_of::<u64>(), size_of::<u32>());
//! static_assert_le!(size_of::<u32>(), size_of::<u64>());
//! static_assert_lt!(size_of::<u32>(), size_of::<u64>());
//! static_assert_size_of!(u32, 4);
//! static_assert_align_of!(u32, 4);
//! ```

use core::marker::PhantomData;

/// A compile time assert to check for equal values
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert_eq;
///
/// static_assert_eq!(1, 1);
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_eq;
///
/// static_assert_eq!(1, 2);
/// ```
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_eq;
///
/// static_assert_eq!(2, 1);
/// ```
#[macro_export]
macro_rules! static_assert_eq {
    ($left:expr, $right:expr) => {
        let _: () = const {
            iceoryx2_bb_elementary::static_assert::static_assert_eq::<{ $left }, { $right }>()
        };
    };
}

/// Implementation detail! Use [`static_assert_eq!`] macro instead
pub const fn static_assert_eq<const L: usize, const R: usize>() {
    let () = AssertEq::<L, R>::OK;
}

struct AssertEq<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertEq<L, R> {
    const OK: () = assert!(L == R, "L must be equal to R");
}

/// A compile time assert to check for greater than or equal values
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert_ge;
///
/// static_assert_ge!(1, 1);
/// static_assert_ge!(2, 1);
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_ge;
///
/// static_assert_ge!(1, 2);
/// ```
#[macro_export]
macro_rules! static_assert_ge {
    ($left:expr, $right:expr) => {
        let _: () = const {
            iceoryx2_bb_elementary::static_assert::static_assert_ge::<{ $left }, { $right }>()
        };
    };
}

/// Implementation detail! Use [`static_assert_ge!`] macro instead
pub const fn static_assert_ge<const L: usize, const R: usize>() {
    let () = AssertGe::<L, R>::OK;
}

struct AssertGe<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertGe<L, R> {
    const OK: () = assert!(L >= R, "L must be greater than or equal to R");
}

/// A compile time assert to check for greater than values
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert_gt;
///
/// static_assert_gt!(2, 1);
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_gt;
///
/// static_assert_gt!(1, 1);
/// ```
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_gt;
///
/// static_assert_gt!(1, 2);
/// ```
#[macro_export]
macro_rules! static_assert_gt {
    ($left:expr, $right:expr) => {
        let _: () = const {
            iceoryx2_bb_elementary::static_assert::static_assert_gt::<{ $left }, { $right }>()
        };
    };
}

/// Implementation detail! Use [`static_assert_gt!`] macro instead
pub const fn static_assert_gt<const L: usize, const R: usize>() {
    let () = AssertGt::<L, R>::OK;
}

struct AssertGt<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertGt<L, R> {
    const OK: () = assert!(L > R, "L must be greater than R");
}

/// A compile time assert to check for less than or equal values
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert_le;
///
/// static_assert_le!(1, 1);
/// static_assert_le!(1, 2);
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_le;
///
/// static_assert_le!(2, 1);
/// ```
#[macro_export]
macro_rules! static_assert_le {
    ($left:expr, $right:expr) => {
        let _: () = const {
            iceoryx2_bb_elementary::static_assert::static_assert_le::<{ $left }, { $right }>()
        };
    };
}

/// Implementation detail! Use [`static_assert_le!`] macro instead
pub const fn static_assert_le<const L: usize, const R: usize>() {
    let () = AssertLe::<L, R>::OK;
}

struct AssertLe<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertLe<L, R> {
    const OK: () = assert!(L <= R, "L must be less than or equal to R");
}

/// A compile time assert to check for less than values
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert_lt;
///
/// static_assert_lt!(1, 2);
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_lt;
///
/// static_assert_lt!(1, 1);
/// ```
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_lt;
///
/// static_assert_lt!(2, 1);
/// ```
#[macro_export]
macro_rules! static_assert_lt {
    ($left:expr, $right:expr) => {
        let _: () = const {
            iceoryx2_bb_elementary::static_assert::static_assert_lt::<{ $left }, { $right }>()
        };
    };
}

/// Implementation detail! Use [`static_assert_lt!`] macro instead
pub const fn static_assert_lt<const L: usize, const R: usize>() {
    let () = AssertLt::<L, R>::OK;
}

struct AssertLt<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertLt<L, R> {
    const OK: () = assert!(L < R, "L must be less than R");
}

/// A compile time assert to check for the size of a type being equal to a value
///
/// Contrary to [`static_assert_eq!`], the [`static_assert_size_of!`](crate::static_assert_size_of) macro can easily
/// be used with const generics, avoiding the `cannot perform const operation using 'T'` error.
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert_size_of;
///
/// pub struct Hypnotoad<T> {
///     pub value: T,
/// }
///
/// impl<T: Default> Hypnotoad<T> {
///     fn new() -> Self {
///         static_assert_size_of!(T, 4);
///         Self { value: T::default() }
///     }
/// }
///
/// let hypnotoad = Hypnotoad::<u32>::new();
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_size_of;
///
/// pub struct Hypnotoad<T> {
///     pub value: T,
/// }
///
/// impl<T: Default> Hypnotoad<T> {
///     fn new() -> Self {
///         static_assert_size_of!(T, 1);
///         Self { value: T::default() }
///     }
/// }
///
/// let hypnotoad = Hypnotoad::<u32>::new();
/// ```
#[macro_export]
macro_rules! static_assert_size_of {
    ($ty:ty, $value:expr) => {
        let _: () = const {
            iceoryx2_bb_elementary::static_assert::static_assert_size_of::<$ty, { $value }>()
        };
    };
}

/// Implementation detail! Use [`static_assert_size_of!`](crate::static_assert_size_of) macro instead
pub const fn static_assert_size_of<T, const SIZE: usize>() {
    let () = AssertSizeOf::<T, SIZE>::OK;
}

struct AssertSizeOf<T, const SIZE: usize> {
    _phantom: PhantomData<T>,
}

impl<T, const SIZE: usize> AssertSizeOf<T, SIZE> {
    const OK: () = assert!(
        core::mem::size_of::<T>() == SIZE,
        "T must have size defined by SIZE"
    );
}

/// A compile time assert to check for the alignment of a type being equal to a value.
///
/// Contrary to [`static_assert_eq!`], the [`static_assert_align_of!`](crate::static_assert_align_of) macro can easily
/// be used with const generics, avoiding the `cannot perform const operation using 'T'` error.
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert_align_of;
///
/// pub struct Hypnotoad<T> {
///     pub value: T,
/// }
///
/// impl<T: Default> Hypnotoad<T> {
///     fn new() -> Self {
///         static_assert_align_of!(T, 4);
///         Self { value: T::default() }
///     }
/// }
///
/// let hypnotoad = Hypnotoad::<u32>::new();
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert_align_of;
///
/// pub struct Hypnotoad<T> {
///     pub value: T,
/// }
///
/// impl<T: Default> Hypnotoad<T> {
///     fn new() -> Self {
///         static_assert_align_of!(T, 1);
///         Self { value: T::default() }
///     }
/// }
///
/// let hypnotoad = Hypnotoad::<u32>::new();
/// ```
#[macro_export]
macro_rules! static_assert_align_of {
    ($ty:ty, $value:expr) => {
        let _: () = const {
            iceoryx2_bb_elementary::static_assert::static_assert_align_of::<$ty, { $value }>()
        };
    };
}

/// Implementation detail! Use [`static_assert_align_of!`](crate::static_assert_align_of) macro instead
pub const fn static_assert_align_of<T, const ALIGNMENT: usize>() {
    let () = AssertAlignOf::<T, ALIGNMENT>::OK;
}

struct AssertAlignOf<T, const ALIGNMENT: usize> {
    _phantom: PhantomData<T>,
}

impl<T, const ALIGNMENT: usize> AssertAlignOf<T, ALIGNMENT> {
    const OK: () = assert!(
        core::mem::align_of::<T>() == ALIGNMENT,
        "T must have alignment defined by ALIGNMENT"
    );
}
