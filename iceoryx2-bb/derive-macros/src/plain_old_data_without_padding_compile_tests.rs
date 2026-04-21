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

/// === Requirement 3: must be `#[repr(C)]` ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
///
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct NoReprC {
///     a: u64,
///     b: u64,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_without_repr_c() {}

/// === Requirement 2: must not have padding between members ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
///
/// // `u8` at offset 0 followed by `u32` forces 3 bytes of padding
/// #[repr(C)]
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct InternalPadding {
///     a: u8,
///     b: u32,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_struct_with_internal_padding() {}

/// === Requirement 2: must not have trailing padding ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
///
/// // Fields sum to 5 bytes but alignment bumps struct size to 8 (3 bytes trailing pad)
/// #[repr(C)]
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct TrailingPadding {
///     a: u32,
///     b: u8,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_struct_with_trailing_padding() {}

/// === Supertrait bound: must implement `Copy` ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
///
/// #[repr(C)]
/// #[derive(Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct NoCopy {
///     a: u64,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_non_copy_struct() {}

/// === Requirement 4: every member must implement `PlainOldDataWithoutPadding` ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
///
/// #[repr(C)]
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend)]
/// struct MissingPodDerive { x: u32 }
///
/// #[repr(C)]
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct WithNonPodField {
///     a: u64,
///     b: MissingPodDerive,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_struct_with_non_pod_field() {}

/// === Enum rejection ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::PlainOldDataWithoutPadding;
///
/// #[repr(C)]
/// #[derive(PlainOldDataWithoutPadding)]
/// enum Kind {
///     A,
///     B,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_enum() {}

/// === Union rejection ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::PlainOldDataWithoutPadding;
///
/// #[repr(C)]
/// #[derive(Copy, Clone, PlainOldDataWithoutPadding)]
/// union MyUnion {
///     a: u64,
///     b: u64,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_union() {}

/// === References violate the `'static` supertrait bound ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
///
/// #[repr(C)]
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct WithReference<'a> {
///     a: u64,
///     b: &'a u64,
/// }
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_struct_with_reference() {}

/// === Tuple struct variants of the above ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
///
/// // tuple struct with internal padding: u8 followed by u32
/// #[repr(C)]
/// #[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
/// struct PaddedTuple(u8, u32);
/// ```
#[cfg(doctest)]
fn pod_derive_does_not_work_for_tuple_struct_with_padding() {}
