// Copyright (c) 2024 Contributors to the Eclipse Foundation
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
mod attribute {
    use iceoryx2::service::attribute::AttributeVerifier;
    use iceoryx2_bb_elementary::CallbackProgression;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn attribute_returns_correct_key_value() {
        let sut = AttributeVerifier::new().require("key_1", "value_1");

        for entry in sut.attributes().iter() {
            assert_that!(entry.key(), eq "key_1");
            assert_that!(entry.value(), eq "value_1");
        }

        assert_that!(sut.attributes().iter(), len 1);
    }

    #[test]
    fn attribute_set_returns_correct_key_len() {
        let sut = AttributeVerifier::new()
            .require("key_1", "value_1")
            .require("key_1", "value_2");

        assert_that!(sut.attributes().get_key_value_len("key_1"), eq 2);
        assert_that!(sut.attributes().get_key_value_len("key_2"), eq 0);
    }

    #[test]
    fn attribute_set_returns_correct_value() {
        let sut = AttributeVerifier::new()
            .require("another_key", "another_value_1")
            .require("another_key", "another_value_2");

        assert_that!(sut.attributes().get_key_value_at("another_key", 0), eq Some("another_value_1"));
        assert_that!(sut.attributes().get_key_value_at("another_key", 1), eq Some("another_value_2"));
        assert_that!(sut.attributes().get_key_value_at("another_key", 2), eq None);
        assert_that!(sut.attributes().get_key_value_at("non_existing_key", 0), eq None);
    }

    #[test]
    fn attribute_set_get_key_value_works() {
        let sut = AttributeVerifier::new()
            .require("wild_ride", "XXL")
            .require("wild_ride", "S");

        let mut values = vec![];
        sut.attributes().get_key_values("wild_ride", |v| {
            values.push(v.to_string());
            CallbackProgression::Continue
        });

        assert_that!(values, contains String::from("XXL"));
        assert_that!(values, contains String::from("S"));
    }

    #[test]
    fn attribute_set_get_key_value_stops_on_request() {
        let sut = AttributeVerifier::new()
            .require("schwifty", "brothers")
            .require("schwifty", "sisters");

        let mut counter = 0;
        sut.attributes().get_key_values("schwifty", |_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn attribute_set_get_key_value_no_callback_call_when_key_does_not_exist() {
        let sut = AttributeVerifier::new()
            .require("schwifler", "brothers")
            .require("schwifler", "sisters");

        let mut counter = 0;
        sut.attributes().get_key_values("does not exist", |_| {
            counter += 1;
            CallbackProgression::Stop
        });

        assert_that!(counter, eq 0);
    }
}
