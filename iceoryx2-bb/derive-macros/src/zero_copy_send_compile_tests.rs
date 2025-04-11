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

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// struct Foo(u16);
///
/// #[derive(ZeroCopySend)]
/// struct NamedTestStruct {
///     val1: u64,
///     val2: Foo,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_named_struct_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// struct Foo(u16);
///
/// #[derive(ZeroCopySend)]
/// struct UnnamedTestStruct(i32, u32, Foo);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_unnamed_struct_when_not_all_members_implement_it() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[derive(ZeroCopySend)]
/// struct UnitLikeTestStruct;
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_unit_like_struct() {}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[derive(ZeroCopySend)]
/// struct GenericNamedTestStruct<T1, T2> {
///     val1: T1,
///     val2: T2,
/// }
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_named_struct_when_not_all_members_implement_it()
{
}

/// ``` compile_fail
/// use iceoryx2_bb_derive_macros::ZeroCopySend;
///
/// #[derive(ZeroCopySend)]
/// struct GenericUnnamedTestStruct<T1, T2>(T1, T2);
/// ```
#[cfg(doctest)]
fn zero_copy_send_derive_does_not_work_for_generic_unnamed_struct_when_not_all_members_implement_it(
) {
}
