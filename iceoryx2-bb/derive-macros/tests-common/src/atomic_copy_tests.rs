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

extern crate alloc;

use alloc::vec::Vec;
use iceoryx2_bb_derive_macros::{AtomicCopy, ZeroCopySend};
use iceoryx2_bb_elementary::math::align_to;
use iceoryx2_bb_elementary_traits::atomic_copy::AtomicCopy;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[allow(dead_code)]
#[derive(Copy, Clone)]
struct Foo(u16);
unsafe impl ZeroCopySend for Foo {}
unsafe impl AtomicCopy for Foo {
    fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
        callback(align_to::<u16>(base_offset), 2);
    }
}

#[allow(dead_code)]
#[repr(C)]
#[derive(ZeroCopySend, Clone, Copy, AtomicCopy)]
struct NestedUnnamedTestStruct(i32, u64, Foo);

#[test]
pub fn field_offsets_and_sizes_are_correct_for_named_struct() {
    #[repr(C)]
    #[derive(ZeroCopySend, Copy, Clone, AtomicCopy)]
    struct NamedTestStruct {
        a: u64,
        b: Foo,
    }
    let sut = NamedTestStruct { a: 0, b: Foo(0) };

    let mut v = Vec::new();
    sut.__for_each_field(0, &mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 2);
    assert_that!(v[0], eq(0, 8));
    assert_that!(v[1], eq(8, 2));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_generic_named_struct() {
    #[repr(C)]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct GenericNamedTestStruct<T1, T2>
    where
        T1: AtomicCopy,
        T2: AtomicCopy,
    {
        a: T1,
        b: T2,
    }
    let sut = GenericNamedTestStruct { a: 0u8, b: 0i32 };

    let mut v = Vec::new();
    sut.__for_each_field(0, &mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 2);
    assert_that!(v[0], eq(0, 1));
    assert_that!(v[1], eq(4, 4));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_unnamed_struct() {
    let mut v = Vec::new();
    let sut = NestedUnnamedTestStruct(0, 0, Foo(0));
    sut.__for_each_field(0, &mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 3);
    assert_that!(v[0], eq(0, 4));
    assert_that!(v[1], eq(8, 8));
    assert_that!(v[2], eq(16, 2));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_generic_unnamed_struct() {
    #[repr(C)]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct GenericUnnamedTestStruct<T1, T2>(T1, T2)
    where
        T1: AtomicCopy,
        T2: AtomicCopy;
    let sut = GenericUnnamedTestStruct(0u64, NestedUnnamedTestStruct(0, 0, Foo(0)));

    let mut v = Vec::new();
    sut.__for_each_field(0, &mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 4);
    assert_that!(v[0], eq(0, 8));
    assert_that!(v[1], eq(8, 4));
    assert_that!(v[2], eq(16, 8));
    assert_that!(v[3], eq(24, 2));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_when_alignment_changes_inner_padding() {
    #[repr(C)]
    #[repr(align(16))]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct AlignedU32(u32);

    #[repr(C)]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct SomeNamedStruct {
        a: u8,
        b: AlignedU32,
    }

    let mut v = Vec::new();
    let sut = SomeNamedStruct {
        a: 3,
        b: AlignedU32(9),
    };
    sut.__for_each_field(0, &mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 2);
    assert_that!(v[0], eq(0, 1));
    assert_that!(v[1], eq(16, 4));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_nested_structs() {
    #[repr(C)]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct SomeUnnamedStruct(u64);

    #[repr(C)]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct SomeNamedStruct {
        a: u8,
        b: i16,
        c: SomeUnnamedStruct,
    }

    #[repr(C)]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct NestedStruct {
        a: SomeUnnamedStruct,
        b: u32,
        c: SomeNamedStruct,
    }

    let mut v = Vec::new();
    let sut = NestedStruct {
        a: SomeUnnamedStruct(32687),
        b: 7,
        c: SomeNamedStruct {
            a: 1,
            b: -4,
            c: SomeUnnamedStruct(5),
        },
    };
    sut.__for_each_field(0, &mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 5);
    assert_that!(v[0], eq(0, 8));
    assert_that!(v[1], eq(8, 4));
    assert_that!(v[2], eq(16, 1));
    assert_that!(v[3], eq(18, 2));
    assert_that!(v[4], eq(24, 8));
}

#[test]
pub fn field_offsets_are_correct_with_custom_implementation() {
    #[repr(C)]
    #[derive(Clone, Copy, ZeroCopySend)]
    struct Foo(u32, u8);
    unsafe impl AtomicCopy for Foo {
        fn __for_each_field<F: FnMut(usize, usize)>(&self, base_offset: usize, callback: &mut F) {
            let aligned_base_offset = align_to::<Self>(base_offset);
            callback(aligned_base_offset + core::mem::offset_of!(Foo, 0), 4);
            callback(aligned_base_offset + core::mem::offset_of!(Foo, 1), 1);
        }
    }

    #[repr(C)]
    #[derive(AtomicCopy, Clone, Copy, ZeroCopySend)]
    struct Bar(u8, Foo);

    let mut v = Vec::new();
    let sut = Bar(0, Foo(0, 0));
    sut.__for_each_field(0, &mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 3);
    assert_that!(v[0], eq(0, 1));
    assert_that!(v[1], eq(4, 4));
    assert_that!(v[2], eq(8, 1));
}
