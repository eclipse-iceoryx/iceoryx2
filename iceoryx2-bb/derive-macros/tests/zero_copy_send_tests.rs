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

#[cfg(test)]
mod zero_copy_send {
    use iceoryx2_bb_derive_macros::ZeroCopySend;
    use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
    use iceoryx2_bb_testing::assert_that;

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
    struct UnnamedTestStruct(i32, u32, Foo);

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

    #[test]
    fn zero_copy_send_derive_works_for_named_struct() {
        let sut = NamedTestStruct {
            _val1: 1990,
            _val2: Foo(3),
        };
        assert_that!(is_zero_copy_send(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_works_for_unnamed_struct() {
        let sut = UnnamedTestStruct(4, 6, Foo(2));
        assert_that!(is_zero_copy_send(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_works_for_generic_named_struct() {
        let sut = GenericNamedTestStruct {
            _val1: 2.3,
            _val2: 1984,
        };
        assert_that!(is_zero_copy_send(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_works_for_generic_unnamed_struct() {
        let sut = GenericUnnamedTestStruct(23.4, 2023);
        assert_that!(is_zero_copy_send(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_sets_type_name_correctly_for_named_structs() {
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
    fn zero_copy_send_derive_sets_type_name_correctly_for_unnamed_structs() {
        let sut = UnnamedTestStruct(1, 2, Foo(3));
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(unsafe { UnnamedTestStruct::type_name() }, eq core::any::type_name::<UnnamedTestStruct>());

        let sut_with_attr = UnnamedTestStructWithAttr(84, 90, Foo(23));
        assert_that!(is_zero_copy_send(&sut_with_attr), eq true);
        assert_that!(unsafe { UnnamedTestStructWithAttr::type_name() }, eq "Hypnotoad");
    }

    #[test]
    fn zero_copy_send_derive_sets_type_name_correctly_for_generic_named_structs() {
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
    fn zero_copy_send_derive_sets_type_name_correctly_for_generic_unnamed_struct() {
        let sut = GenericUnnamedTestStruct(-13, 13);
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(unsafe { GenericUnnamedTestStruct::<i32, i32>::type_name() }, eq core::any::type_name::<GenericUnnamedTestStruct<i32, i32>>());

        let sut_with_attr = GenericUnnamedTestStructWithAttr(-13, 13);
        assert_that!(is_zero_copy_send(&sut_with_attr), eq true);
        assert_that!(unsafe { GenericUnnamedTestStructWithAttr::<i32, i32>::type_name() }, eq "Smeik");
    }
}
