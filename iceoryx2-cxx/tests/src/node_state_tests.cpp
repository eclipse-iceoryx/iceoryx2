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

#include "test.hpp"

namespace {
using namespace iox2;

template <typename T>
class NodeStateTest : public ::testing::Test {
  public:
    static constexpr ServiceType TYPE = T::TYPE;
};

TYPED_TEST_SUITE(NodeStateTest, iox2_testing::ServiceTypes, );

TYPED_TEST(NodeStateTest, alive_node_works) {
    constexpr ServiceType SERVICE_TYPE = TestFixture::TYPE;

    const auto* valid_name = "Which companies middleware could be best described as a dead horse!";
    auto node_name = NodeName::create(valid_name).expect("");
    auto node = NodeBuilder().name(node_name).create<SERVICE_TYPE>().expect("");
    auto node_id = node.id();

    bool alive_node_found = false;
    bool has_invalid_state = false;
    Node<SERVICE_TYPE>::list(node.config(), [&](auto state) {
        state.alive(
            [&](auto& view) { alive_node_found = view.details()->name().to_string() == node_name.to_string(); });
        state.dead(
            [&](auto& view) { has_invalid_state = view.details()->name().to_string() == node_name.to_string(); });
        state.inaccessible([&](auto& view) { has_invalid_state = view == node_id; });
        state.undefined([&](auto& view) { has_invalid_state = view == node_id; });

        if (alive_node_found || has_invalid_state) {
            return CallbackProgression::Stop;
        }

        return CallbackProgression::Continue;
    }).expect("");

    ASSERT_TRUE(alive_node_found);
}
} // namespace
