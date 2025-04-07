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

// Test if something doesn't compile? compiletests.rs

#[cfg(test)]
mod zero_copy_send {
    use iceoryx2_bb_derive_macros::ZeroCopySend;
    use iceoryx2_bb_elementary::{
        identifiable::Identifiable, relocatable::Relocatable, zero_copy_send::ZeroCopySend,
    };
    use iceoryx2_bb_testing::assert_that;

    fn is_zero_copy_send<T: ZeroCopySend>(_: &T) -> bool {
        true
    }

    fn is_identifiable<T: Identifiable>(_: &T) -> bool {
        true
    }

    #[allow(dead_code)]
    struct Foo(u16);
    unsafe impl Relocatable for Foo {}

    #[derive(ZeroCopySend)]
    struct NamedTestStruct {
        _val1: u64,
        _val2: Foo,
    }

    #[allow(dead_code)]
    #[derive(ZeroCopySend)]
    struct UnnamedTestStruct(i32, u32, Foo);

    #[derive(ZeroCopySend)]
    struct GenericNamedTestStruct<T1, T2> {
        _val1: T1,
        _val2: T2,
    }

    #[derive(ZeroCopySend)]
    struct GenericUnnamedTestStruct<T1, T2, T3>(T1, T2, T3);

    #[test]
    fn zero_copy_send_derive_works_for_named_struct() {
        let sut = NamedTestStruct {
            _val1: 1990,
            _val2: Foo(3),
        };
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(is_identifiable(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_works_for_unnamed_struct() {
        let sut = UnnamedTestStruct(4, 6, Foo(2));
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(is_identifiable(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_works_for_named_generic_struct() {
        let sut = GenericNamedTestStruct {
            _val1: false,
            _val2: -1984,
        };
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(is_identifiable(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_works_for_unnamed_generic_struct() {
        let sut = GenericUnnamedTestStruct(2023.4, 'N', 23);
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(is_identifiable(&sut), eq true);
    }

    #[derive(ZeroCopySend)]
    struct MyType {
        val: u64,
        //val: String,
    }

    #[derive(ZeroCopySend)]
    struct MyTupleStruct(i32, u64);
    //struct MyTupleStruct(i32, String);

    //#[derive(ZeroCopySend)]
    //struct MyUnitStruct;

    #[test]
    fn blub_works() {
        let x = MyType { val: 9 };
        //let x = MyType {
        //val: String::from("test"),
        //};
        println!("val = {}", x.val);
        println!("type name = {}", unsafe { MyType::type_name() });

        let y = MyTupleStruct(0, 0);
        //let y = MyTupleStruct(0, String::from("hui"));
        println!("tuple = {}, {}", y.0, y.1);
        println!("type name = {}", unsafe { MyTupleStruct::type_name() });
    }
}
