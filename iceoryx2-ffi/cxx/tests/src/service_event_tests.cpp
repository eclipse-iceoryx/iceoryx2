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

#include "iox2/node.hpp"
#include "iox2/node_name.hpp"
#include "iox2/service.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
class ServiceEventTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(ServiceEventTest, iox2_testing::ServiceTypes);

TYPED_TEST(ServiceEventTest, created_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "First time we met, I saw the ocean, it was wet!";
    const auto service_name = ServiceName::create(name_value).expect("");

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = node.service_builder(service_name).event().create().expect("");

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}

TYPED_TEST(ServiceEventTest, creating_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "First time we met, I saw the ocean, it was wet!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).event().create().expect("");

    auto sut_2 = node.service_builder(service_name).event().create();
    ASSERT_TRUE(sut_2.has_error());
    ASSERT_THAT(sut_2.error(), Eq(EventCreateError::AlreadyExists));
}

TYPED_TEST(ServiceEventTest, open_or_create_service_does_exist) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "First time we met, I saw the ocean, it was wet!";
    const auto service_name = ServiceName::create(name_value).expect("");

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");

    {
        auto sut = iox::optional<PortFactoryEvent<SERVICE_TYPE>>(
            node.service_builder(service_name).event().open_or_create().expect(""));

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));

        auto sut_2 = iox::optional<PortFactoryEvent<SERVICE_TYPE>>(
            node.service_builder(service_name).event().open_or_create().expect(""));

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));

        sut.reset();

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));

        sut_2.reset();
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}

TYPED_TEST(ServiceEventTest, opening_non_existing_service_fails) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "First time we met, I saw the ocean, it was wet!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut = node.service_builder(service_name).event().open();
    ASSERT_TRUE(sut.has_error());
    ASSERT_THAT(sut.error(), Eq(EventOpenError::DoesNotExist));
}

TYPED_TEST(ServiceEventTest, opening_existing_service_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "First time we met, I saw the ocean, it was wet!";
    const auto service_name = ServiceName::create(name_value).expect("");

    auto node = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto sut_create = node.service_builder(service_name).event().create();
    auto sut = node.service_builder(service_name).event().open();
    ASSERT_TRUE(sut.has_value());
}
} // namespace
