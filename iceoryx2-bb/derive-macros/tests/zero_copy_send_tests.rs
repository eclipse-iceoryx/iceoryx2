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
    struct GenericNamedTestStruct<T1, T2>
    where
        T1: Relocatable,
        T2: Relocatable,
    {
        _val1: T1,
        _val2: T2,
    }

    #[derive(ZeroCopySend)]
    struct GenericUnnamedTestStruct<T1, T2>(T1, T2)
    where
        T1: Relocatable,
        T2: Relocatable;

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
    fn zero_copy_send_derive_works_for_generic_named_struct() {
        let sut = GenericNamedTestStruct {
            _val1: 2.3,
            _val2: 1984,
        };
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(is_identifiable(&sut), eq true);
    }

    #[test]
    fn zero_copy_send_derive_works_for_generic_unnamed_struct() {
        let sut = GenericUnnamedTestStruct(23.4, 2023);
        assert_that!(is_zero_copy_send(&sut), eq true);
        assert_that!(is_identifiable(&sut), eq true);
    }
}
