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
//! use iceoryx2_bb_elementary::static_assert::*;
//!
//! use core::mem::{align_of, size_of};
//!
//! static_assert_eq::<{ size_of::<u64>() }, 8>();
//! static_assert_ge::<{ size_of::<u64>() }, { size_of::<u32>() }>();
//! static_assert_gt::<{ size_of::<u64>() }, { size_of::<u32>() }>();
//! static_assert_le::<{ size_of::<u32>() }, { size_of::<u64>() }>();
//! static_assert_lt::<{ size_of::<u32>() }, { size_of::<u64>() }>();
//! ```

/// A compile time assert to check for equal values
///
/// # Examples
///
/// This does compile!
///
/// ```
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_eq::<1, 1>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_eq::<1, 2>();
/// static_assert_eq::<2, 1>();
///
/// ```
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
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_ge::<1, 1>();
/// static_assert_ge::<2, 1>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_ge::<1, 2>();
///
/// ```
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
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_gt::<2, 1>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_gt::<1, 1>();
/// static_assert_gt::<1, 2>();
///
/// ```
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
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_le::<1, 1>();
/// static_assert_le::<1, 2>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_le::<2, 1>();
///
/// ```
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
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_lt::<1, 2>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_lt::<1, 1>();
/// static_assert_lt::<2, 1>();
///
/// ```
pub const fn static_assert_lt<const L: usize, const R: usize>() {
    let () = AssertLt::<L, R>::OK;
}

struct AssertLt<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertLt<L, R> {
    const OK: () = assert!(L < R, "L must be less than R");
}
