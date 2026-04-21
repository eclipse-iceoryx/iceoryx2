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

use iceoryx2_bb_derive_macros::Zeroable;
use iceoryx2_bb_elementary_traits::zeroable::Zeroable;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

fn is_zeroable<T: Zeroable>(_: &T) -> bool {
    true
}

#[derive(Zeroable)]
struct NamedTestStruct {
    value1: u64,
    value2: u32,
    value3: u8,
    value4: [u8; 8],
    value5: bool,
}

#[derive(Zeroable)]
struct UnnamedTestStruct(u64, u32, u8, [u8; 4]);

#[derive(Zeroable)]
struct NestedTestStruct {
    inner: NamedTestStruct,
    extra: u64,
}

#[derive(Zeroable)]
struct GenericNamedTestStruct<T1, T2>
where
    T1: Zeroable,
    T2: Zeroable,
{
    value1: T1,
    value2: T2,
}

#[derive(Zeroable)]
struct GenericUnnamedTestStruct<T1, T2>(T1, T2)
where
    T1: Zeroable,
    T2: Zeroable;

#[test]
fn zeroable_derive_for_named_struct_produces_all_zero_fields() {
    let v = NamedTestStruct::new_zeroed();

    assert_that!(v.value1, eq 0);
    assert_that!(v.value2, eq 0);
    assert_that!(v.value3, eq 0);
    assert_that!(v.value4, eq [0u8; 8]);
    assert_that!(v.value5, eq false);
    assert_that!(is_zeroable(&v), eq true);
}

#[test]
fn zeroable_derive_for_tuple_struct_produces_all_zero_fields() {
    let v = UnnamedTestStruct::new_zeroed();

    assert_that!(v.0, eq 0);
    assert_that!(v.1, eq 0);
    assert_that!(v.2, eq 0);
    assert_that!(v.3, eq [0u8; 4]);
    assert_that!(is_zeroable(&v), eq true);
}

#[test]
fn zeroable_derive_for_nested_struct_zeroes_all_inner_fields() {
    let v = NestedTestStruct::new_zeroed();

    assert_that!(v.inner.value1, eq 0);
    assert_that!(v.inner.value2, eq 0);
    assert_that!(v.inner.value3, eq 0);
    assert_that!(v.inner.value4, eq [0u8; 8]);
    assert_that!(v.inner.value5, eq false);
    assert_that!(v.extra, eq 0);
    assert_that!(is_zeroable(&v), eq true);
}

#[test]
fn zeroable_derive_for_generic_named_struct_works() {
    type SutType = GenericNamedTestStruct<u64, u32>;
    let v = SutType::new_zeroed();

    assert_that!(v.value1, eq 0);
    assert_that!(v.value2, eq 0);
    assert_that!(is_zeroable(&v), eq true);
}

#[test]
fn zeroable_derive_for_generic_tuple_struct_works() {
    type SutType = GenericUnnamedTestStruct<u64, [u8; 8]>;
    let v = SutType::new_zeroed();

    assert_that!(v.0, eq 0);
    assert_that!(v.1, eq [0u8; 8]);
    assert_that!(is_zeroable(&v), eq true);
}
