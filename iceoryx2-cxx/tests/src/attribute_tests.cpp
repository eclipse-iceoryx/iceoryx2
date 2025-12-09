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
#include <cstddef>

namespace {
using namespace iox2;

TEST(AttributeVerifier, require_is_listed_in_attributes) {
    auto key = *Attribute::Key::from_utf8("some_key");
    auto value = *Attribute::Value::from_utf8("oh my god, its a value");
    auto attribute_verifier = AttributeVerifier();
    attribute_verifier.require(key, value).value();

    auto attributes = attribute_verifier.attributes();

    ASSERT_THAT(attributes.number_of_attributes(), Eq(1));
    ASSERT_THAT(attributes[0].key(), Eq(key));
    ASSERT_THAT(attributes[0].value(), Eq(value));
}

TEST(AttributeVerifier, required_keys_are_listed_in_keys) {
    auto key_1 = *Attribute::Key::from_utf8("where is my key");
    auto key_2 = *Attribute::Key::from_utf8("Nala, find my keys!");
    auto attribute_verifier = AttributeVerifier();
    attribute_verifier.require_key(key_1).value();
    attribute_verifier.require_key(key_2).value();

    auto keys = attribute_verifier.keys();

    ASSERT_THAT(keys.size(), Eq(2));
    ASSERT_THAT(keys.unchecked_access()[0], Eq(key_1));
    ASSERT_THAT(keys.unchecked_access()[1], Eq(key_2));
}

TEST(AttributeVerifier, verify_requirements_successful_for_compatible_setups) {
    auto key = *Attribute::Key::from_utf8("the secret to happiness");
    auto value = *Attribute::Value::from_utf8("is on the nose of an iceoryx");
    auto attribute_verifier = AttributeVerifier();
    attribute_verifier.require(key, value).value();

    auto attributes = attribute_verifier.attributes();

    auto result = attribute_verifier.verify_requirements(attributes);

    ASSERT_THAT(result.has_value(), Eq(true));
}

TEST(AttributeVerifier, verify_requirements_returns_key_for_incompatible_setups) {
    auto key = *Attribute::Key::from_utf8("is there a fireoryx");
    auto value = *Attribute::Value::from_utf8("or a windoryx");
    auto missing_key = *Attribute::Key::from_utf8("or a earthoryx");
    auto incompatible_attribute_verifier = AttributeVerifier();
    incompatible_attribute_verifier.require(key, value).value();
    auto attribute_verifier = AttributeVerifier();
    attribute_verifier.require(key, value).value();
    attribute_verifier.require_key(missing_key).value();

    auto incompatible_attributes = incompatible_attribute_verifier.attributes();

    auto result = attribute_verifier.verify_requirements(incompatible_attributes);

    ASSERT_THAT(result.has_value(), Eq(false));
    ASSERT_THAT(result.error(), Eq(missing_key));
}

TEST(AttributeSpecifier, all_defined_attributes_are_set) {
    auto key_1 = *Attribute::Key::from_utf8("our goal:");
    auto value_1 = *Attribute::Value::from_utf8("iceoryx runs on the uss enterprise");
    auto key_2 = *Attribute::Key::from_utf8("wouldn't it be cool if");
    auto value_2 = *Attribute::Value::from_utf8("scotty must debug some ancient iceoryx2 technology");

    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key_1, value_1).value();
    attribute_specifier.define(key_2, value_2).value();
    auto attributes = attribute_specifier.attributes();

    ASSERT_THAT(attributes.number_of_attributes(), Eq(2));
    ASSERT_THAT(attributes[0].key(), Eq(key_1));
    ASSERT_THAT(attributes[0].value(), Eq(value_1));
    ASSERT_THAT(attributes[1].key(), Eq(key_2));
    ASSERT_THAT(attributes[1].value(), Eq(value_2));
}

TEST(AttributeSet, all_key_values_can_be_listed) {
    auto key = *Attribute::Key::from_utf8("shall zero-copy");
    auto value_1 = *Attribute::Value::from_utf8("be with you");
    auto value_2 = *Attribute::Value::from_utf8("or not be with you");

    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value_1).value();
    attribute_specifier.define(key, value_2).value();
    auto attributes = attribute_specifier.attributes();

    ASSERT_THAT(attributes.number_of_attributes(), Eq(2));
    ASSERT_THAT(attributes[0].key(), Eq(key));
    ASSERT_THAT(attributes[1].key(), Eq(key));
    ASSERT_THAT(attributes[0].value(), Eq(value_1));
    ASSERT_THAT(attributes[1].value(), Eq(value_2));
}

TEST(AttributeSet, all_key_values_can_be_acquired) {
    auto key = *Attribute::Key::from_utf8("santa clauses slide is actually run");
    std::vector<Attribute::Value> values = { *Attribute::Value::from_utf8("by one iceoryx"),
                                             *Attribute::Value::from_utf8("reindeers are retired") };

    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, values[0]).value();
    attribute_specifier.define(key, values[1]).value();
    auto attributes = attribute_specifier.attributes();

    size_t counter = 0;

    attributes.iter_key_values(key, [&](const auto& value) -> CallbackProgression {
        EXPECT_THAT(value, Eq(values[counter]));
        counter++;
        return CallbackProgression::Continue;
    });
}

TEST(AttributeSet, get_key_value_len_works) {
    auto empty_key = *Attribute::Key::from_utf8("fuu");
    auto key = *Attribute::Key::from_utf8("whatever");
    auto value_1 = *Attribute::Value::from_utf8("you");
    auto value_2 = *Attribute::Value::from_utf8("want");

    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value_1).value();
    attribute_specifier.define(key, value_2).value();
    auto attributes = attribute_specifier.attributes();

    ASSERT_THAT(attributes.number_of_key_values(key), Eq(2));
    ASSERT_THAT(attributes.number_of_key_values(empty_key), Eq(0));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TEST(AttributeSet, get_key_value_at_works) {
    auto key = *Attribute::Key::from_utf8("schmu whatever");
    auto value_1 = *Attribute::Value::from_utf8("fuu you");
    auto value_2 = *Attribute::Value::from_utf8("blue want");

    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value_1).value();
    attribute_specifier.define(key, value_2).value();
    auto attributes = attribute_specifier.attributes();

    auto v_1 = attributes.key_value(key, 0);
    auto v_2 = attributes.key_value(key, 1);
    auto v_3 = attributes.key_value(key, 2);

    ASSERT_THAT(v_1.has_value(), Eq(true));
    ASSERT_THAT(v_2.has_value(), Eq(true));
    ASSERT_THAT(v_3.has_value(), Eq(false));

    if (v_1->size() == value_1.size()) {
        ASSERT_THAT(v_1.value().unchecked_access().c_str(), StrEq(value_1.unchecked_access().c_str()));
        ASSERT_THAT(v_2.value().unchecked_access().c_str(), StrEq(value_2.unchecked_access().c_str()));
    } else {
        ASSERT_THAT(v_2.value().unchecked_access().c_str(), StrEq(value_1.unchecked_access().c_str()));
        ASSERT_THAT(v_1.value().unchecked_access().c_str(), StrEq(value_2.unchecked_access().c_str()));
    }
}
//NOLINTEND(readability-function-cognitive-complexity)

TEST(AttributeSet, to_owned_works) {
    auto key = *Attribute::Key::from_utf8("your mind becomes a galaxy");
    auto value_1 = *Attribute::Value::from_utf8("shiny and bright");
    auto value_2 = *Attribute::Value::from_utf8("with spice aroma");

    auto attribute_specifier = AttributeSpecifier();
    attribute_specifier.define(key, value_1).value();
    attribute_specifier.define(key, value_2).value();
    auto attributes = attribute_specifier.attributes();
    auto attributes_owned = attributes.to_owned();

    ASSERT_THAT(attributes_owned.number_of_attributes(), Eq(2));
    ASSERT_THAT(attributes_owned[0].key(), Eq(key));
    ASSERT_THAT(attributes_owned[1].key(), Eq(key));
    ASSERT_THAT(attributes_owned[0].value(), Eq(value_1));
    ASSERT_THAT(attributes_owned[1].value(), Eq(value_2));
}
} // namespace
