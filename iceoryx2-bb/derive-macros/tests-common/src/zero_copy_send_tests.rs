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

extern crate alloc;

use alloc::vec::Vec;
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

fn is_zero_copy_send<T: ZeroCopySend>(_: &T) -> bool {
    true
}

#[allow(dead_code)]
struct Foo(u16);
unsafe impl ZeroCopySend for Foo {}

#[repr(C)]
#[derive(ZeroCopySend)]
struct NamedTestStruct {
    _val1: u64,
    _val2: Foo,
}

#[repr(C)]
#[derive(ZeroCopySend)]
#[type_name("Nala")]
struct NamedTestStructWithAttr {
    _val1: u64,
    _val2: Foo,
}

#[allow(dead_code)]
#[repr(C)]
#[derive(ZeroCopySend)]
struct UnnamedTestStruct(i32, u64, Foo);

#[repr(C)]
#[derive(ZeroCopySend)]
#[type_name("Hypnotoad")]
struct UnnamedTestStructWithAttr(i32, u32, Foo);

#[repr(C)]
#[derive(ZeroCopySend)]
struct GenericNamedTestStruct<T1, T2>
where
    T1: ZeroCopySend,
    T2: ZeroCopySend,
{
    _val1: T1,
    _val2: T2,
}

#[repr(C)]
#[derive(ZeroCopySend)]
#[type_name("Wolf")]
struct GenericNamedTestStructWithAttr<T1, T2>
where
    T1: ZeroCopySend,
    T2: ZeroCopySend,
{
    _val1: T1,
    _val2: T2,
}

#[repr(C)]
#[derive(ZeroCopySend)]
struct GenericUnnamedTestStruct<T1, T2>(T1, T2)
where
    T1: ZeroCopySend,
    T2: ZeroCopySend;

#[repr(C)]
#[derive(ZeroCopySend)]
#[type_name("Smeik")]
struct GenericUnnamedTestStructWithAttr<T1, T2>(T1, T2)
where
    T1: ZeroCopySend,
    T2: ZeroCopySend;

#[repr(C)]
#[derive(ZeroCopySend)]
#[type_name("TryMadHoney")]
union BasicUnionTest {
    _val1: u32,
    _val2: u8,
}

#[test]
pub fn works_for_named_struct() {
    let sut = NamedTestStruct {
        _val1: 1990,
        _val2: Foo(3),
    };
    assert_that!(is_zero_copy_send(&sut), eq true);
}

#[test]
pub fn works_for_unnamed_struct() {
    let sut = UnnamedTestStruct(4, 6, Foo(2));
    assert_that!(is_zero_copy_send(&sut), eq true);
}

#[test]
pub fn works_for_generic_named_struct() {
    let sut = GenericNamedTestStruct {
        _val1: 2.3,
        _val2: 1984,
    };
    assert_that!(is_zero_copy_send(&sut), eq true);
}

#[test]
pub fn works_for_generic_unnamed_struct() {
    let sut = GenericUnnamedTestStruct(23.4, 2023);
    assert_that!(is_zero_copy_send(&sut), eq true);
}

#[test]
pub fn sets_type_name_correctly_for_named_structs() {
    let sut = NamedTestStruct {
        _val1: 23,
        _val2: Foo(4),
    };
    assert_that!(is_zero_copy_send(&sut), eq true);
    assert_that!(unsafe { NamedTestStruct::type_name() }, eq core::any::type_name::<NamedTestStruct>());

    let sut_with_attr = NamedTestStructWithAttr {
        _val1: 20,
        _val2: Foo(23),
    };
    assert_that!(is_zero_copy_send(&sut_with_attr), eq true);
    assert_that!(unsafe { NamedTestStructWithAttr::type_name() }, eq "Nala");
}

#[test]
pub fn sets_type_name_correctly_for_unnamed_structs() {
    let sut = UnnamedTestStruct(1, 2, Foo(3));
    assert_that!(is_zero_copy_send(&sut), eq true);
    assert_that!(unsafe { UnnamedTestStruct::type_name() }, eq core::any::type_name::<UnnamedTestStruct>());

    let sut_with_attr = UnnamedTestStructWithAttr(84, 90, Foo(23));
    assert_that!(is_zero_copy_send(&sut_with_attr), eq true);
    assert_that!(unsafe { UnnamedTestStructWithAttr::type_name() }, eq "Hypnotoad");
}

#[test]
pub fn sets_type_name_correctly_for_generic_named_structs() {
    let sut = GenericNamedTestStruct {
        _val1: 11,
        _val2: Foo(11),
    };
    assert_that!(is_zero_copy_send(&sut), eq true);
    assert_that!(unsafe { GenericNamedTestStruct::<i32, Foo>::type_name() }, eq core::any::type_name::<GenericNamedTestStruct<i32, Foo>>());

    let sut_with_attr = GenericNamedTestStructWithAttr {
        _val1: 11.11,
        _val2: Foo(2008),
    };
    assert_that!(is_zero_copy_send(&sut_with_attr), eq true);
    assert_that!(unsafe { GenericNamedTestStructWithAttr::<f32, Foo>::type_name() }, eq "Wolf");
}

#[test]
pub fn sets_type_name_correctly_for_generic_unnamed_struct() {
    let sut = GenericUnnamedTestStruct(-13, 13);
    assert_that!(is_zero_copy_send(&sut), eq true);
    assert_that!(unsafe { GenericUnnamedTestStruct::<i32, i32>::type_name() }, eq core::any::type_name::<GenericUnnamedTestStruct<i32, i32>>());

    let sut_with_attr = GenericUnnamedTestStructWithAttr(-13, 13);
    assert_that!(is_zero_copy_send(&sut_with_attr), eq true);
    assert_that!(unsafe { GenericUnnamedTestStructWithAttr::<i32, i32>::type_name() }, eq "Smeik");
}

#[test]
pub fn for_unions() {
    let sut = BasicUnionTest { _val1: 12 };
    assert_that!(is_zero_copy_send(&sut), eq true);
    assert_that!(unsafe { BasicUnionTest::type_name() }, eq "TryMadHoney");
}

#[test]
#[should_panic]
pub fn field_offsets_and_sizes_cannot_be_calculated_for_unions() {
    let sut = BasicUnionTest { _val1: 12 };
    sut.__for_each_field(&mut |_, _| {});
}

#[test]
#[should_panic]
pub fn field_offsets_and_sizes_cannot_be_calculated_for_generic_unions() {
    #[repr(C)]
    #[derive(ZeroCopySend)]
    union GenericUnion<T: Copy + ZeroCopySend> {
        val1: u8,
        val2: T,
    }

    let sut = GenericUnion { val2: 0u64 };
    sut.__for_each_field(&mut |_, _| {});
}

#[test]
#[should_panic]
pub fn field_offsets_and_sizes_cannot_be_calculated_for_unions_with_struct_field() {
    #[repr(C)]
    #[derive(Clone, Copy, ZeroCopySend)]
    struct SomeNamedStruct {
        a: u8,
        b: u64,
    }

    #[repr(C)]
    #[derive(ZeroCopySend)]
    union NestedUnion {
        val1: u8,
        val2: SomeNamedStruct,
    }

    let sut = NestedUnion {
        val2: SomeNamedStruct { a: 0, b: 0 },
    };
    sut.__for_each_field(&mut |_, _| {});
}

#[test]
#[should_panic]
pub fn field_offsets_and_sizes_cannot_be_calculated_for_structs_with_union() {
    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct NestedStruct {
        a: BasicUnionTest,
        b: u8,
    }

    let sut = NestedStruct {
        a: BasicUnionTest { _val1: 12 },
        b: 0,
    };
    sut.__for_each_field(&mut |_, _| {});
}

#[test]
#[should_panic]
pub fn field_offsets_and_sizes_cannot_be_calculated_for_structs_with_enum() {
    #[repr(C)]
    #[derive(ZeroCopySend)]
    enum SomeEnum {
        A,
        B,
    }

    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct NestedStruct {
        a: SomeEnum,
        b: u8,
    }

    let sut = NestedStruct {
        a: SomeEnum::A,
        b: 0,
    };
    sut.__for_each_field(&mut |_, _| {});
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_named_struct() {
    let mut v = Vec::new();
    let sut = NamedTestStruct {
        _val1: 0,
        _val2: Foo(0),
    };
    sut.__for_each_field(&mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 2);
    assert_that!(v[0], eq(0, 8));
    assert_that!(v[1], eq(8, 2));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_generic_named_struct() {
    let mut v = Vec::new();
    let sut = GenericNamedTestStruct {
        _val1: 0u8,
        _val2: 0i32,
    };
    sut.__for_each_field(&mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 2);
    assert_that!(v[0], eq(0, 1));
    assert_that!(v[1], eq(4, 4));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_unnamed_struct() {
    let mut v = Vec::new();
    let sut = UnnamedTestStruct(0, 0, Foo(0));
    sut.__for_each_field(&mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 3);
    assert_that!(v[0], eq(0, 4));
    assert_that!(v[1], eq(8, 8));
    assert_that!(v[2], eq(16, 2));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_generic_unnamed_struct() {
    let mut v = Vec::new();
    let sut = GenericUnnamedTestStruct(0u64, UnnamedTestStruct(0, 0, Foo(0)));
    sut.__for_each_field(&mut |offset, size| {
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
    #[derive(ZeroCopySend)]
    struct AlignedU32(u32);

    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct SomeNamedStruct {
        a: u8,
        b: AlignedU32,
    }

    let mut v = Vec::new();
    let sut = SomeNamedStruct {
        a: 3,
        b: AlignedU32(9),
    };
    sut.__for_each_field(&mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 2);
    assert_that!(v[0], eq(0, 1));
    assert_that!(v[1], eq(16, 4));
}

#[test]
pub fn field_offsets_and_sizes_are_correct_for_nested_structs() {
    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct SomeUnnamedStruct(u64);

    #[repr(C)]
    #[derive(ZeroCopySend)]
    struct SomeNamedStruct {
        a: u8,
        b: i16,
        c: SomeUnnamedStruct,
    }

    #[repr(C)]
    #[derive(ZeroCopySend)]
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
    sut.__for_each_field(&mut |offset, size| {
        v.push((offset, size));
    });

    assert_that!(v, len 5);
    assert_that!(v[0], eq(0, 8));
    assert_that!(v[1], eq(8, 4));
    assert_that!(v[2], eq(16, 1));
    assert_that!(v[3], eq(18, 2));
    assert_that!(v[4], eq(24, 8));
}
