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
//! static_assert_equal::<{ size_of::<u64>() }, 8>();
//! static_assert_gt_or_equal::<{ size_of::<u64>() }, { size_of::<u32>() }>();
//! static_assert_lt_or_equal::<{ size_of::<u32>() }, { size_of::<u64>() }>();
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
/// static_assert_equal::<1, 1>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_equal::<1, 2>();
/// static_assert_equal::<2, 1>();
///
/// ```
pub const fn static_assert_equal<const L: usize, const R: usize>() {
    let () = AssertEqual::<L, R>::OK;
}

struct AssertEqual<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertEqual<L, R> {
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
/// static_assert_gt_or_equal::<1, 1>();
/// static_assert_gt_or_equal::<2, 1>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_gt_or_equal::<1, 2>();
///
/// ```
pub const fn static_assert_gt_or_equal<const L: usize, const R: usize>() {
    let () = AssertGtOrEqual::<L, R>::OK;
}

struct AssertGtOrEqual<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertGtOrEqual<L, R> {
    const OK: () = assert!(L >= R, "L must be greater than or equal to R");
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
/// static_assert_lt_or_equal::<1, 1>();
/// static_assert_lt_or_equal::<1, 2>();
///
/// ```
///
/// This does not compile!
///
/// ```compile_fail
/// use iceoryx2_bb_elementary::static_assert::*;
///
/// static_assert_lt_or_equal::<2, 1>();
///
/// ```
pub const fn static_assert_lt_or_equal<const L: usize, const R: usize>() {
    let () = AssertLtOrEqual::<L, R>::OK;
}

struct AssertLtOrEqual<const L: usize, const R: usize>;

impl<const L: usize, const R: usize> AssertLtOrEqual<L, R> {
    const OK: () = assert!(L <= R, "L must be less than or equal to R");
}
