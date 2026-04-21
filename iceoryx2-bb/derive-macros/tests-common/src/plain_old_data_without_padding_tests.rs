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

use iceoryx2_bb_derive_macros::{PlainOldDataWithoutPadding, ZeroCopySend, Zeroable};
use iceoryx2_bb_elementary_traits::plain_old_data_without_padding::PlainOldDataWithoutPadding;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zeroable::Zeroable;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

fn is_pod<T: PlainOldDataWithoutPadding>(_: &T) -> bool {
    true
}

fn is_zeroable<T: Zeroable>(_: &T) -> bool {
    true
}

fn is_copy<T: Copy>(_: &T) -> bool {
    true
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
struct NamedPodStruct {
    value1: u64,
    value2: u64,
    value3: u64,
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
struct UnnamedPodStruct(u64, u32, u32);

#[repr(C)]
#[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
struct ArrayFieldPodStruct {
    data: [u8; 16],
    count: u64,
}

// Nested: the inner struct must also be PlainOldDataWithoutPadding.
#[repr(C)]
#[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
struct InnerPodStruct {
    a: u32,
    b: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Zeroable, ZeroCopySend, PlainOldDataWithoutPadding)]
struct NestedPodStruct {
    inner: InnerPodStruct,
    trailing: u64,
}

#[test]
fn pod_derive_for_named_struct_satisfies_all_trait_bounds() {
    let v = NamedPodStruct {
        value1: 1,
        value2: 2,
        value3: 3,
    };

    assert_that!(is_pod(&v), eq true);
    assert_that!(is_zeroable(&v), eq true);
    assert_that!(is_copy(&v), eq true);
    assert_that!(core::mem::size_of::<NamedPodStruct>(), eq 24);
}

#[test]
fn pod_derive_for_tuple_struct_satisfies_trait_bound() {
    let v = UnnamedPodStruct(1, 2, 3);

    assert_that!(is_pod(&v), eq true);
    assert_that!(core::mem::size_of::<UnnamedPodStruct>(), eq 16);
}

#[test]
fn pod_derive_with_array_field_satisfies_trait_bound() {
    let v = ArrayFieldPodStruct {
        data: [0; 16],
        count: 0,
    };

    assert_that!(is_pod(&v), eq true);
    assert_that!(core::mem::size_of::<ArrayFieldPodStruct>(), eq 24);
}

#[test]
fn pod_derive_for_nested_struct_satisfies_trait_bound() {
    let v = NestedPodStruct {
        inner: InnerPodStruct { a: 1, b: 2 },
        trailing: 3,
    };

    assert_that!(is_pod(&v), eq true);
    assert_that!(
        core::mem::size_of::<NestedPodStruct>(),
        eq core::mem::size_of::<InnerPodStruct>() + core::mem::size_of::<u64>()
    );
}

#[test]
fn pod_derive_implies_zeroable_via_new_zeroed() {
    let v = NamedPodStruct::new_zeroed();

    assert_that!(v.value1, eq 0);
    assert_that!(v.value2, eq 0);
    assert_that!(v.value3, eq 0);
}
