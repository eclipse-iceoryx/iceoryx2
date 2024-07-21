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

#include <vector>

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
        auto sut = node.service_builder(service_name).event().open_or_create().expect("");

        ASSERT_TRUE(Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event)
                        .expect(""));
    }

    ASSERT_FALSE(
        Service<SERVICE_TYPE>::does_exist(service_name, Config::global_config(), MessagingPattern::Event).expect(""));
}
} // namespace
