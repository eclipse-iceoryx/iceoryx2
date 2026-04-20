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

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::AtomicCopy;
///
/// #[derive(AtomicCopy)]
/// union Foo {
///     a: u32,
///     b: u8,
/// }
/// ```
#[cfg(doctest)]
fn atomic_copy_derive_does_not_work_for_unions() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::AtomicCopy;
/// use iceoryx2_bb_elementary_traits::atomic_copy::AtomicCopy;
///
/// #[derive(AtomicCopy)]
/// union Foo<T: AtomicCopy> {
///     a: u32,
///     b: T,
/// }
/// ```
#[cfg(doctest)]
fn atomic_copy_derive_does_not_work_for_generic_unions() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{AtomicCopy, ZeroCopySend};
/// use iceoryx2_bb_elementary_traits::atomic_copy::AtomicCopy;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
/// struct NamedStruct {
///     a: u8,
///     b: u64,
/// }
/// #[derive(AtomicCopy)]
/// union NestedUnion {
///     a: NamedStruct,
///     b: u8,
/// }
/// ```
#[cfg(doctest)]
fn atomic_copy_derive_does_not_work_for_unions_containing_struct_field() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::AtomicCopy;
///
/// #[derive(AtomicCopy)]
/// enum Foo {
///     A,
///     B,
/// }
/// ```
#[cfg(doctest)]
fn atomic_copy_derive_does_not_work_for_enums() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::AtomicCopy;
/// use iceoryx2_bb_elementary_traits::atomic_copy::AtomicCopy;
///
/// #[derive(AtomicCopy, Clone, Copy)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
#[cfg(doctest)]
fn atomic_copy_derive_does_not_work_when_struct_is_not_zero_copy_send() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::{AtomicCopy, ZeroCopySend};
/// use iceoryx2_bb_elementary_traits::atomic_copy::AtomicCopy;
/// use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
///
/// #[repr(C)]
/// #[derive(AtomicCopy, ZeroCopySend)]
/// struct Foo {
///     a: u32,
///     b: u64,
/// }
#[cfg(doctest)]
fn atomic_copy_derive_does_not_work_when_struct_is_not_copy() {}
