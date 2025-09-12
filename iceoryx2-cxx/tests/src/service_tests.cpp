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

#include "iox2/attribute_specifier.hpp"
#include "iox2/node.hpp"
#include "iox2/service.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
class ServiceTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServiceTest, iox2_testing::ServiceTypes, );

TYPED_TEST(ServiceTest, does_exist_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name = iox2_testing::generate_service_name();

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));
    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
                .expect(""));
        ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                         .expect(""));
        ASSERT_FALSE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
                .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));
    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::RequestResponse)
            .expect(""));
}

TYPED_TEST(ServiceTest, list_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    const auto service_name_3 = iox2_testing::generate_service_name();
    const auto service_name_4 = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    auto sut_1 = node.service_builder(service_name_1).template publish_subscribe<uint64_t>().create().expect("");
    auto sut_2 = node.service_builder(service_name_2).event().create().expect("");
    auto sut_3 =
        node.service_builder(service_name_3).template request_response<uint64_t, uint64_t>().create().expect("");

    //NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by EXPECT_THAT
    auto verify = [&](auto details) -> CallbackProgression {
        switch (details.static_details.messaging_pattern()) {
        case MessagingPattern::PublishSubscribe:
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_1.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_1.service_id().c_str()));
            break;
        case MessagingPattern::Event:
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_2.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_2.service_id().c_str()));
            break;
        case MessagingPattern::RequestResponse:
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_3.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_3.service_id().c_str()));
            break;
        }

        return CallbackProgression::Continue;
    };
    //NOLINTEND(readability-function-cognitive-complexity)

    auto result = Service<SERVICE_TYPE>::list(Config::global_config(), verify);

    ASSERT_THAT(result.has_value(), Eq(true));
}

TYPED_TEST(ServiceTest, list_works_with_attributes) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key_1 = Attribute::Key("do elephants like strawberries?");
    auto value_1 = Attribute::Value("do strawberries like elephants?");
    auto key_2 = Attribute::Key("the berry of the straw");
    auto value_2 = Attribute::Value("has left the field!");


    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();
    const auto service_name_3 = iox2_testing::generate_service_name();
    const auto service_name_4 = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    auto sut_1 = node.service_builder(service_name_1)
                     .template publish_subscribe<uint64_t>()
                     .create_with_attributes(AttributeSpecifier().define(key_1, value_1).define(key_2, value_2))
                     .expect("");
    auto sut_2 = node.service_builder(service_name_2).event().create().expect("");
    auto sut_3 = node.service_builder(service_name_3)
                     .template request_response<uint64_t, uint64_t>()
                     .create_with_attributes(AttributeSpecifier().define(key_1, value_1).define(key_2, value_2))
                     .expect("");

    auto counter = 0;
    //NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by EXPECT_THAT
    auto verify = [&](auto details) -> CallbackProgression {
        switch (details.static_details.messaging_pattern()) {
        case MessagingPattern::PublishSubscribe:
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_1.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_1.service_id().c_str()));

            counter = 0;
            details.static_details.attributes().iter_key_values(key_1, [&](auto& value) -> CallbackProgression {
                EXPECT_THAT(value.c_str(), StrEq(value_1.c_str()));
                counter++;
                return CallbackProgression::Continue;
            });
            EXPECT_THAT(counter, Eq(1));

            counter = 0;
            details.static_details.attributes().iter_key_values(key_2, [&](auto& value) -> CallbackProgression {
                EXPECT_THAT(value.c_str(), StrEq(value_2.c_str()));
                counter++;
                return CallbackProgression::Continue;
            });
            EXPECT_THAT(counter, Eq(1));
            break;
        case MessagingPattern::Event:
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_2.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_2.service_id().c_str()));
            break;
        case MessagingPattern::RequestResponse:
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_3.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_3.service_id().c_str()));

            counter = 0;
            details.static_details.attributes().iter_key_values(key_1, [&](auto& value) -> CallbackProgression {
                EXPECT_THAT(value.c_str(), StrEq(value_1.c_str()));
                counter++;
                return CallbackProgression::Continue;
            });
            EXPECT_THAT(counter, Eq(1));

            counter = 0;
            details.static_details.attributes().iter_key_values(key_2, [&](auto& value) -> CallbackProgression {
                EXPECT_THAT(value.c_str(), StrEq(value_2.c_str()));
                counter++;
                return CallbackProgression::Continue;
            });
            EXPECT_THAT(counter, Eq(1));
            break;
        }

        return CallbackProgression::Continue;
    };
    //NOLINTEND(readability-function-cognitive-complexity)

    auto result = Service<SERVICE_TYPE>::list(Config::global_config(), verify);

    ASSERT_THAT(result.has_value(), Eq(true));
}

//NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by ASSERT_THAT
TYPED_TEST(ServiceTest, details_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto key_1 = Attribute::Key("gimme a strawberries?");
    auto value_1 = Attribute::Value("i want a strawberry!");
    auto key_2 = Attribute::Key("it makes me immortal");
    auto value_2 = Attribute::Value("or at least sticky");


    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    auto sut = node.service_builder(service_name_1)
                   .template publish_subscribe<uint64_t>()
                   .create_with_attributes(AttributeSpecifier().define(key_1, value_1).define(key_2, value_2))
                   .expect("");

    auto result =
        Service<SERVICE_TYPE>::details(service_name_1, Config::global_config(), MessagingPattern::PublishSubscribe);

    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(result->has_value(), Eq(true));

    ASSERT_THAT(result.value()->static_details.name(), StrEq(service_name_1.to_string().c_str()));
    ASSERT_THAT(result.value()->static_details.name(), StrEq(service_name_1.to_string().c_str()));

    auto counter = 0;
    result.value()->static_details.attributes().iter_key_values(key_1, [&](auto& value) -> CallbackProgression {
        EXPECT_THAT(value.c_str(), StrEq(value_1.c_str()));
        counter++;
        return CallbackProgression::Continue;
    });
    EXPECT_THAT(counter, Eq(1));

    counter = 0;
    result.value()->static_details.attributes().iter_key_values(key_2, [&](auto& value) -> CallbackProgression {
        EXPECT_THAT(value.c_str(), StrEq(value_2.c_str()));
        counter++;
        return CallbackProgression::Continue;
    });
    EXPECT_THAT(counter, Eq(1));

    result = Service<SERVICE_TYPE>::details(service_name_1, Config::global_config(), MessagingPattern::Event);
    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(result->has_value(), Eq(false));

    result =
        Service<SERVICE_TYPE>::details(service_name_2, Config::global_config(), MessagingPattern::PublishSubscribe);
    ASSERT_THAT(result.has_value(), Eq(true));
    ASSERT_THAT(result->has_value(), Eq(false));
}
//NOLINTEND(readability-function-cognitive-complexity)
} // namespace
