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

/// === Enum / union rejection ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::Zeroable;
///
/// #[derive(Zeroable)]
/// enum Color {
///     Red,
///     Blue,
/// }
/// ```
#[cfg(doctest)]
fn zeroable_derive_does_not_work_for_enum() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::Zeroable;
///
/// #[derive(Zeroable)]
/// union MyUnion {
///     a: u32,
///     b: u64,
/// }
/// ```
#[cfg(doctest)]
fn zeroable_derive_does_not_work_for_union() {}

/// === Structs containing a non-Zeroable field ===

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::Zeroable;
///
/// #[derive(Zeroable)]
/// struct NamedStructWithNonZeroField {
///     a: u32,
///     b: core::num::NonZeroU32,
/// }
/// ```
#[cfg(doctest)]
fn zeroable_derive_does_not_work_for_named_struct_with_non_zeroable_field() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::Zeroable;
///
/// #[derive(Zeroable)]
/// struct TupleStructWithNonZeroField(u32, core::num::NonZeroU32);
/// ```
#[cfg(doctest)]
fn zeroable_derive_does_not_work_for_tuple_struct_with_non_zeroable_field() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::Zeroable;
/// use iceoryx2_bb_elementary_traits::zeroable::Zeroable;
///
/// #[derive(Zeroable)]
/// struct StructWithReference {
///     a: u32,
///     b: &'static u32,
/// }
///
/// // Force monomorphization so the impl's where-bound is actually checked
/// fn assert_zeroable<T: Zeroable>() {}
/// assert_zeroable::<StructWithReference>();
/// ```
#[cfg(doctest)]
fn zeroable_derive_does_not_work_for_struct_with_reference_field() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::Zeroable;
///
/// struct NonZeroable;
///
/// #[derive(Zeroable)]
/// struct StructWithNonZeroableType {
///     a: u32,
///     b: NonZeroable,
/// }
/// ```
#[cfg(doctest)]
fn zeroable_derive_does_not_work_for_struct_with_non_zeroable_typed_field() {}
