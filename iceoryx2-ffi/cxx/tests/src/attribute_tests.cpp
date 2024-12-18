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

#include "iox2/attribute.hpp"
#include "iox2/attribute_set.hpp"
#include "iox2/attribute_specifier.hpp"
#include "iox2/attribute_verifier.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

TEST(AttributeVerifier, require_is_listed_in_attributes) {
    auto key = Attribute::Key("some_key");
    auto value = Attribute::Value("oh my god, its a value");
    auto attribute_verifier = AttributeVerifier().require(key, value);

    auto attributes = attribute_verifier.attributes();

    ASSERT_THAT(attributes.len(), Eq(1));
    ASSERT_THAT(attributes.at(0).key(), Eq(key));
    ASSERT_THAT(attributes.at(0).value(), Eq(value));
}

TEST(AttributeVerifier, required_keys_are_listed_in_keys) {
    auto key_1 = Attribute::Key("where is my key");
    auto key_2 = Attribute::Key("Nala, find my keys!");
    auto attribute_verifier = AttributeVerifier().require_key(key_1).require_key(key_2);

    auto keys = attribute_verifier.keys();

    ASSERT_THAT(keys.size(), Eq(2));
    ASSERT_THAT(keys[0], Eq(key_1));
    ASSERT_THAT(keys[1], Eq(key_2));
}

TEST(AttributeVerifier, verify_requirements_successful_for_compatible_setups) {
    auto key = Attribute::Key("the secret to happiness");
    auto value = Attribute::Value("is on the nose of an iceoryx");
    auto attribute_verifier = AttributeVerifier().require(key, value);

    auto attributes = attribute_verifier.attributes();

    auto result = attribute_verifier.verify_requirements(attributes);

    ASSERT_THAT(result.has_value(), Eq(true));
}

TEST(AttributeVerifier, verify_requirements_returns_key_for_incompatible_setups) {
    auto key = Attribute::Key("is there a fireoryx");
    auto value = Attribute::Value("or a windoryx");
    auto missing_key = Attribute::Key("or a earthoryx");
    auto incompatible_attribute_verifier = AttributeVerifier().require(key, value);
    auto attribute_verifier = AttributeVerifier().require(key, value).require_key(missing_key);

    auto incompatible_attributes = incompatible_attribute_verifier.attributes();

    auto result = attribute_verifier.verify_requirements(incompatible_attributes);

    ASSERT_THAT(result.has_value(), Eq(false));
    ASSERT_THAT(result.error(), Eq(missing_key));
}

TEST(AttributeSpecifier, all_defined_attributes_are_set) {
    auto key_1 = Attribute::Key("our goal:");
    auto value_1 = Attribute::Value("iceoryx runs on the uss enterprise");
    auto key_2 = Attribute::Key("wouldn't it be cool if");
    auto value_2 = Attribute::Value("scotty must debug some ancient iceoryx2 technology");

    auto attribute_specifier = AttributeSpecifier().define(key_1, value_1).define(key_2, value_2);
    auto attributes = attribute_specifier.attributes();

    ASSERT_THAT(attributes.len(), Eq(2));
    ASSERT_THAT(attributes.at(0).key(), Eq(key_1));
    ASSERT_THAT(attributes.at(0).value(), Eq(value_1));
    ASSERT_THAT(attributes.at(1).key(), Eq(key_2));
    ASSERT_THAT(attributes.at(1).value(), Eq(value_2));
}

TEST(AttributeSet, all_key_values_can_be_listed) {
    auto key = Attribute::Key("shall zero-copy");
    auto value_1 = Attribute::Value("be with you");
    auto value_2 = Attribute::Value("or not be with you");

    auto attribute_specifer = AttributeSpecifier().define(key, value_1).define(key, value_2);
    auto attributes = attribute_specifer.attributes();

    ASSERT_THAT(attributes.len(), Eq(2));
    ASSERT_THAT(attributes.at(0).key(), Eq(key));
    ASSERT_THAT(attributes.at(1).key(), Eq(key));
    ASSERT_THAT(attributes.at(0).value(), Eq(value_1));
    ASSERT_THAT(attributes.at(1).value(), Eq(value_2));
}

TEST(AttributeSet, all_key_values_can_be_acquired) {
    auto key = Attribute::Key("santa clauses slide is actually run");
    std::vector<Attribute::Value> values = { Attribute::Value("by one iceoryx"),
                                             Attribute::Value("reindeers are retired") };

    auto attribute_specifer = AttributeSpecifier().define(key, values[0]).define(key, values[1]);
    auto attributes = attribute_specifer.attributes();

    auto counter = 0;

    attributes.get_key_values(key, [&](const auto& value) -> CallbackProgression {
        EXPECT_THAT(value, Eq(values[counter]));
        counter++;
        return CallbackProgression::Continue;
    });
}
} // namespace
