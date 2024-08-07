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

#include "iox2/node_state.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
class NodeStateTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(NodeStateTest, iox2_testing::ServiceTypes);

TYPED_TEST(NodeStateTest, alive_node_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* valid_name = "Which companies middleware could be best described as a dead horse!";
    auto node_name = NodeName::create(valid_name).expect("");
    auto sut = NodeState<SERVICE_TYPE>(
        AliveNodeView<SERVICE_TYPE>(NodeId {}, NodeDetails { node_name, Config::global_config().to_owned() }));

    iox::optional<NodeName> test_name;
    bool entered_wrong_callback = false;
    sut.alive([&](auto& view) { test_name = view.details()->name(); });
    sut.dead([&](auto& view) { entered_wrong_callback = true; });
    sut.undefined([&](auto& view) { entered_wrong_callback = true; });
    sut.inaccessible([&](auto& view) { entered_wrong_callback = true; });

    ASSERT_FALSE(entered_wrong_callback);
    ASSERT_TRUE(test_name.has_value());
    ASSERT_THAT(test_name->to_string().c_str(), StrEq(valid_name));
}

TYPED_TEST(NodeStateTest, dead_node_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* valid_name = "Oh look there is Super-Hypnotoad flying to the moon!";
    auto node_name = NodeName::create(valid_name).expect("");
    auto sut = NodeState<SERVICE_TYPE>(DeadNodeView<SERVICE_TYPE>(
        AliveNodeView<SERVICE_TYPE>(NodeId {}, NodeDetails { node_name, Config::global_config().to_owned() })));

    iox::optional<NodeName> test_name;
    bool entered_wrong_callback = false;
    sut.alive([&](auto& view) { entered_wrong_callback = true; });
    sut.dead([&](auto& view) { test_name = view.details()->name(); });
    sut.undefined([&](auto& view) { entered_wrong_callback = true; });
    sut.inaccessible([&](auto& view) { entered_wrong_callback = true; });

    ASSERT_FALSE(entered_wrong_callback);
    ASSERT_TRUE(test_name.has_value());
    ASSERT_THAT(test_name->to_string().c_str(), StrEq(valid_name));
}

TYPED_TEST(NodeStateTest, inaccessible_node_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto sut = NodeState<SERVICE_TYPE>(iox2_node_state_e_INACCESSIBLE, NodeId {});

    bool entered_right_callback = false;
    bool entered_wrong_callback = false;
    sut.alive([&](auto& view) { entered_wrong_callback = true; });
    sut.dead([&](auto& view) { entered_wrong_callback = true; });
    sut.undefined([&](auto& view) { entered_wrong_callback = true; });
    sut.inaccessible([&](auto& view) { entered_right_callback = true; });

    ASSERT_FALSE(entered_wrong_callback);
    ASSERT_TRUE(entered_right_callback);
}

TYPED_TEST(NodeStateTest, undefined_node_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    auto sut = NodeState<SERVICE_TYPE>(iox2_node_state_e_UNDEFINED, NodeId {});

    bool entered_right_callback = false;
    bool entered_wrong_callback = false;
    sut.alive([&](auto& view) { entered_wrong_callback = true; });
    sut.dead([&](auto& view) { entered_wrong_callback = true; });
    sut.undefined([&](auto& view) { entered_right_callback = true; });
    sut.inaccessible([&](auto& view) { entered_wrong_callback = true; });

    ASSERT_FALSE(entered_wrong_callback);
    ASSERT_TRUE(entered_right_callback);
}
} // namespace
