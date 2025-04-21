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

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = node.service_builder(service_name).template publish_subscribe<uint64_t>().create().expect("");

        ASSERT_TRUE(
            Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
                .expect(""));
        ASSERT_FALSE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                         .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::PublishSubscribe)
            .expect(""));
    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}

TYPED_TEST(ServiceTest, list_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto service_name_1 = iox2_testing::generate_service_name();
    const auto service_name_2 = iox2_testing::generate_service_name();

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    auto sut_1 = node.service_builder(service_name_1).template publish_subscribe<uint64_t>().create().expect("");
    auto sut_2 = node.service_builder(service_name_2).event().create().expect("");

    //NOLINTBEGIN(readability-function-cognitive-complexity), false positive caused by EXPECT_THAT
    auto verify = [&](auto details) -> CallbackProgression {
        if (details.static_details.messaging_pattern() == MessagingPattern::PublishSubscribe) {
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_1.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_1.service_id().as_str()));
        } else {
            EXPECT_THAT(details.static_details.name(), StrEq(service_name_2.to_string().c_str()));
            EXPECT_THAT(details.static_details.id(), StrEq(sut_2.service_id().as_str()));
        }

        return CallbackProgression::Continue;
    };
    //NOLINTEND(readability-function-cognitive-complexity)

    auto result = Service<SERVICE_TYPE>::list(Config::global_config(), verify);

    ASSERT_THAT(result.has_value(), Eq(true));
}
} // namespace
