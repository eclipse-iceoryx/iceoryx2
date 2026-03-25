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
    use iceoryx2::{
        prelude::{AttributeSet, AttributeSpecifier},
        service::attribute::{
            AttributeDefinitionError, AttributeVerificationError, AttributeVerifier,
        },
    };
    use iceoryx2_bb_elementary::CallbackProgression;
    use iceoryx2_bb_testing::assert_that;

    #[test]
    fn attribute_returns_correct_key_value() {
        let sut = AttributeVerifier::new()
            .require(&"key_1".try_into().unwrap(), &"value_1".try_into().unwrap())
            .unwrap();

        for entry in sut.required_attributes().iter() {
            assert_that!(&entry.key(), eq & "key_1");
            assert_that!(&entry.value(), eq & "value_1");
        }

        assert_that!(sut.required_attributes().iter(), len 1);
    }

    #[test]
    fn attribute_verifier_require_fails_when_it_exceeds_max_number_of_attributes() {
        let mut sut = AttributeVerifier::new();
        for n in 0..AttributeSet::capacity() {
            sut = sut
                .require(
                    &"key".try_into().unwrap(),
                    &n.to_string().as_str().try_into().unwrap(),
                )
                .unwrap();
        }

        assert_that!(sut.require(&"key".try_into().unwrap(), &"val".try_into().unwrap()).err(), eq Some(AttributeDefinitionError::ExceedsMaxSupportedAttributes));
    }

    #[test]
    fn attribute_verifier_require_key_fails_when_it_exceeds_max_number_of_attributes() {
        let mut sut = AttributeVerifier::new();
        for n in 0..AttributeSet::capacity() {
            sut = sut
                .require(
                    &"key".try_into().unwrap(),
                    &n.to_string().as_str().try_into().unwrap(),
                )
                .unwrap();
        }

        assert_that!(sut.require_key(&"key".try_into().unwrap()).err(), eq Some(AttributeDefinitionError::ExceedsMaxSupportedAttributes));
    }

    #[test]
    fn attribute_set_returns_correct_key_len() {
        let sut = AttributeVerifier::new()
            .require(&"key_1".try_into().unwrap(), &"value_1".try_into().unwrap())
            .unwrap()
            .require(&"key_1".try_into().unwrap(), &"value_2".try_into().unwrap())
            .unwrap();

        assert_that!(sut.required_attributes().number_of_key_values(&"key_1".try_into().unwrap()), eq 2);
        assert_that!(sut.required_attributes().number_of_key_values(&"key_2".try_into().unwrap()), eq 0);
    }

    #[test]
    fn attribute_set_returns_correct_value() {
        let sut = AttributeVerifier::new()
            .require(
                &"another_key".try_into().unwrap(),
                &"another_value_1".try_into().unwrap(),
            )
            .unwrap()
            .require(
                &"another_key".try_into().unwrap(),
                &"another_value_2".try_into().unwrap(),
            )
            .unwrap();

        assert_that!(
            sut.required_attributes()
                .key_value(&"another_key".try_into().unwrap(), 0),
            is_some
        );
        assert_that!(sut.required_attributes().key_value(&"another_key".try_into().unwrap(), 0).unwrap(), eq "another_value_1");
        assert_that!(
            sut.required_attributes()
                .key_value(&"another_key".try_into().unwrap(), 1),
            is_some
        );
        assert_that!(sut.required_attributes().key_value(&"another_key".try_into().unwrap(), 1).unwrap(), eq "another_value_2");
        assert_that!(sut.required_attributes().key_value(&"another_key".try_into().unwrap(), 2), eq None);
        assert_that!(sut.required_attributes().key_value(&"non_existing_key".try_into().unwrap(), 0), eq None);
    }

    #[test]
    fn attribute_set_get_key_value_works() {
        let sut = AttributeVerifier::new()
            .require(&"wild_ride".try_into().unwrap(), &"XXL".try_into().unwrap())
            .unwrap()
            .require(&"wild_ride".try_into().unwrap(), &"S".try_into().unwrap())
            .unwrap();

        let mut values = vec![];
        sut.required_attributes()
            .iter_key_values(&"wild_ride".try_into().unwrap(), |v| {
                values.push(v.to_string());
                CallbackProgression::Continue
            });

        assert_that!(values, contains String::from("XXL"));
        assert_that!(values, contains String::from("S"));
    }

    #[test]
    fn attribute_set_get_key_value_stops_on_request() {
        let sut = AttributeVerifier::new()
            .require(
                &"schwifty".try_into().unwrap(),
                &"brothers".try_into().unwrap(),
            )
            .unwrap()
            .require(
                &"schwifty".try_into().unwrap(),
                &"sisters".try_into().unwrap(),
            )
            .unwrap();

        let mut counter = 0;
        sut.required_attributes()
            .iter_key_values(&"schwifty".try_into().unwrap(), |_| {
                counter += 1;
                CallbackProgression::Stop
            });

        assert_that!(counter, eq 1);
    }

    #[test]
    fn attribute_set_get_key_value_no_callback_call_when_key_does_not_exist() {
        let sut = AttributeVerifier::new()
            .require(
                &"schwifler".try_into().unwrap(),
                &"brothers".try_into().unwrap(),
            )
            .unwrap()
            .require(
                &"schwifler".try_into().unwrap(),
                &"sisters".try_into().unwrap(),
            )
            .unwrap();

        let mut counter = 0;
        sut.required_attributes()
            .iter_key_values(&"does not exist".try_into().unwrap(), |_| {
                counter += 1;
                CallbackProgression::Stop
            });

        assert_that!(counter, eq 0);
    }

    #[test]
    fn attribute_set_verify_requirements_succeeds_when_compatible() {
        let sut_verifier = AttributeVerifier::new()
            .require(
                &"schwifler".try_into().unwrap(),
                &"brothers".try_into().unwrap(),
            )
            .unwrap();

        let sut_specifier = AttributeSpecifier::new()
            .define(
                &"schwifler".try_into().unwrap(),
                &"brothers".try_into().unwrap(),
            )
            .unwrap();

        assert_that!(
            sut_verifier.verify_requirements(sut_specifier.attributes()),
            is_ok
        );
    }

    #[test]
    fn attribute_set_verify_requirements_fails_when_value_is_wrong() {
        let sut_verifier = AttributeVerifier::new()
            .require(
                &"schwifler".try_into().unwrap(),
                &"brothers".try_into().unwrap(),
            )
            .unwrap();

        let sut_specifier = AttributeSpecifier::new()
            .define(
                &"schwifler".try_into().unwrap(),
                &"sisters".try_into().unwrap(),
            )
            .unwrap();

        assert_that!(
            sut_verifier.verify_requirements(sut_specifier.attributes()),
            eq Err(AttributeVerificationError::IncompatibleAttribute((
                "schwifler".try_into().unwrap(),
                "brothers".try_into().unwrap(),
            )))
        );
    }

    #[test]
    fn attribute_set_verify_requirements_fails_when_key_does_not_exist() {
        let sut_verifier = AttributeVerifier::new()
            .require_key(&"the toad toad toad".try_into().unwrap())
            .unwrap();

        let sut_specifier = AttributeSpecifier::new()
            .define(
                &"schwifler".try_into().unwrap(),
                &"sisters".try_into().unwrap(),
            )
            .unwrap();

        assert_that!(
            sut_verifier.verify_requirements(sut_specifier.attributes()),
            eq Err(AttributeVerificationError::NonExistingKey(
                "the toad toad toad".try_into().unwrap(),
            ))
        );
    }
}
