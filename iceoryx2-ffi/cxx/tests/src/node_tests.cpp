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

#include <vector>

#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
class NodeTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(NodeTest, iox2_testing::ServiceTypes);

TYPED_TEST(NodeTest, node_name_is_applied) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* name_value = "First time we met, I saw the ocean, it was wet!";
    auto node_name = NodeName::create(name_value).expect("");

    auto sut = NodeBuilder().name(node_name).create<SERVICE_TYPE>().expect("");
    ASSERT_THAT(sut.name().to_string(), Eq(node_name.to_string()));
}

TYPED_TEST(NodeTest, created_nodes_can_be_listed) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto node_name_1 = NodeName::create("Nala does not like water.").expect("");
    auto node_name_2 = NodeName::create("Nala does not like paprika.").expect("");

    {
        auto sut_1 = NodeBuilder().name(node_name_1).create<SERVICE_TYPE>().expect("");
        auto sut_2 = NodeBuilder().name(node_name_2).create<SERVICE_TYPE>().expect("");

        std::vector<NodeName> nodes;
        auto result = Node<SERVICE_TYPE>::list(Config::global_config(), [&](auto node_state) {
            node_state.alive([&](auto& view) { nodes.push_back(view.details()->name()); });
            return CallbackProgression::Continue;
        });
        ASSERT_TRUE(result.has_value());

        auto contains = [&](const NodeName& name) {
            // NOLINTNEXTLINE(readability-use-anyofallof), not yet supported in all compilers
            for (const auto& node : nodes) {
                if (node.to_string() == name.to_string()) {
                    return true;
                }
            }
            return false;
        };

        ASSERT_TRUE(contains(node_name_1));
        ASSERT_TRUE(contains(node_name_2));
    }

    uint64_t counter = 0;
    auto result = Node<SERVICE_TYPE>::list(Config::global_config(), [&](auto node_state) {
        counter++;
        return CallbackProgression::Continue;
    });
    ASSERT_TRUE(result.has_value());
    ASSERT_THAT(counter, Eq(0));
}

TYPED_TEST(NodeTest, node_wait_returns_tick_on_timeout) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    constexpr uint64_t TIMEOUT = 10;
    auto sut = NodeBuilder().create<SERVICE_TYPE>().expect("");
    auto event = sut.wait(iox::units::Duration::fromMilliseconds(TIMEOUT));

    ASSERT_THAT(event, Eq(WaitEvent::Tick));
}

} // namespace
