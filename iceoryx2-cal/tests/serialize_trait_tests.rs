// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

#[generic_tests::define]
mod serialize {
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::serialize::Serialize;

    #[derive(Debug, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
    struct TestStruct {
        value1: String,
        value2: u64,
        value3: bool,
    }

    #[test]
    fn serialize_deserialize_works<Sut: Serialize>() {
        let test_object = TestStruct {
            value1: "hello world".to_string(),
            value2: 192381,
            value3: false,
        };

        let serialized = Sut::serialize(&test_object);
        assert_that!(serialized, is_ok);

        let deserialized = Sut::deserialize::<TestStruct>(&serialized.unwrap());
        assert_that!(deserialized, is_ok);
        assert_that!(deserialized.unwrap(), eq test_object);
    }

    #[instantiate_tests(<iceoryx2_cal::serialize::toml::Toml>)]
    mod toml {}

    #[instantiate_tests(<iceoryx2_cal::serialize::cdr::Cdr>)]
    mod cdr {}

    #[instantiate_tests(<iceoryx2_cal::serialize::postcard::Postcard>)]
    mod postcard {}
}
