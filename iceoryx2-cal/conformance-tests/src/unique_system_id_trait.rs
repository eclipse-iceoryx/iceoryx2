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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod unique_system_id_trait {
    use iceoryx2_bb_container::string::StaticString;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_bb_testing_macros::test;
    use iceoryx2_cal::unique_system_id_generator::{Entity, UniqueId, UniqueSystemIdGenerator};

    #[test]
    fn unique_id_can_be_created_from_value() {
        let value: u128 = 43209;
        let id = unsafe { UniqueId::from_value(value) };
        assert_that!(id.value(), eq value);
    }

    #[conformance_test]
    pub fn generate_works_with_valid_arguments<Sut: UniqueSystemIdGenerator>() {
        let sut = Sut::generate(&Entity {
            name: StaticString::try_from("id").unwrap(),
            id: 0,
        });
        assert_that!(sut, is_ok);
    }

    #[conformance_test]
    pub fn generate_returns_unique_ids<Sut: UniqueSystemIdGenerator>() {
        let sut1 = Sut::generate(&Entity {
            name: StaticString::try_from("id").unwrap(),
            id: 0,
        });
        let sut2 = Sut::generate(&Entity {
            name: StaticString::try_from("id").unwrap(),
            id: 1,
        });
        let sut3 = Sut::generate(&Entity {
            name: StaticString::try_from("ID").unwrap(),
            id: 0,
        });
        assert_that!(sut1, ne sut2);
        assert_that!(sut1, ne sut3);
        assert_that!(sut2, ne sut3);
    }
}
