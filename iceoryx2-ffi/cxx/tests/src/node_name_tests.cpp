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

#include "iox2/node_name.hpp"

#include "test.hpp"

namespace {
using namespace iox2;

TEST(NodeName, valid_node_name_can_be_created) {
    const auto* valid_name = "First time we met, I saw the ocean, it was wet!";
    auto sut = NodeName::create(valid_name);

    ASSERT_THAT(sut.has_value(), Eq(true));

    ASSERT_THAT(sut->to_string().c_str(), StrEq(valid_name));
}

TEST(NodeName, creating_node_name_with_too_long_name_fails) {
    constexpr uint64_t MAX_OVERLENGTH = 10;
    auto test = [](auto overlength) {
        auto invalid_name = std::string(IOX2_NODE_NAME_LENGTH + overlength, 's');
        auto sut = NodeName::create(invalid_name.c_str());

        ASSERT_THAT(sut.has_value(), Eq(false));
        ASSERT_THAT(sut.error(), Eq(SemanticStringError::ExceedsMaximumLength));
    };

    for (uint64_t i = 1; i < MAX_OVERLENGTH; ++i) {
        test(i);
    }
}

TEST(NodeName, as_view_works) {
    const auto* valid_name = "You're my heart, you my toad.";
    auto sut = NodeName::create(valid_name).expect("");
    auto sut_view = sut.as_view();

    ASSERT_THAT(sut.to_string().c_str(), StrEq(sut_view.to_string().c_str()));
}

TEST(NodeName, to_owned_works) {
    const auto* valid_name = "Brother toady toad.";
    auto sut = NodeName::create(valid_name).expect("");
    auto sut_view = sut.as_view();
    auto sut_owned = sut_view.to_owned();

    ASSERT_THAT(sut_view.to_string().c_str(), StrEq(sut_owned.to_string().c_str()));
}

TEST(NodeName, copy_works) {
    const auto* valid_name = "A glass full of apple beans - what?";
    const NodeName sut = NodeName::create(valid_name).expect("");
    NodeName sut_assign = NodeName::create("blarb").expect("");
    NodeName sut_copy { sut }; //NOLINT
    sut_assign = sut;

    ASSERT_THAT(sut.to_string().c_str(), StrEq(valid_name));
    ASSERT_THAT(sut.to_string(), Eq(sut_copy.to_string()));
    ASSERT_THAT(sut.to_string(), Eq(sut_assign.to_string()));
}

TEST(NodeName, move_works) {
    const auto* valid_name = "Who is the Cyborg-Vampire?";
    NodeName sut = NodeName::create(valid_name).expect("");
    NodeName sut_move { std::move(sut) };

    ASSERT_THAT(sut_move.to_string().c_str(), StrEq(valid_name));
    sut = std::move(sut_move);
    ASSERT_THAT(sut.to_string().c_str(), StrEq(valid_name));
}
} // namespace
