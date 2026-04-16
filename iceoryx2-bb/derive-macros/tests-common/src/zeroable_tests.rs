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
    _val1: u64,
    _val2: u32,
    _val3: [u8; 16],
}

#[derive(Zeroable)]
struct UnnamedTestStruct(u64, u32);

#[derive(Zeroable)]
struct UnitTestStruct;

#[derive(Zeroable)]
struct GenericTestStruct<T: Zeroable> {
    _val1: T,
    _val2: u64,
}

#[test]
fn zeroable_derive_works_for_named_struct() {
    let sut = NamedTestStruct::new_zeroed();
    assert_that!(is_zeroable(&sut), eq true);
}

#[test]
fn zeroable_derive_works_for_unnamed_struct() {
    let sut = UnnamedTestStruct::new_zeroed();
    assert_that!(is_zeroable(&sut), eq true);
}

#[test]
fn zeroable_derive_works_for_unit_struct() {
    let sut = UnitTestStruct::new_zeroed();
    assert_that!(is_zeroable(&sut), eq true);
}

#[test]
fn zeroable_derive_works_for_generic_struct() {
    let sut = GenericTestStruct::<u32>::new_zeroed();
    assert_that!(is_zeroable(&sut), eq true);
}
