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

TEST(Node, valid_node_name_can_be_created) {
    const auto* valid_name = "First time we met, I saw the ocean, it was wet!";
    auto sut = NodeName::create(valid_name);

    ASSERT_THAT(sut.has_value(), Eq(true));

    ASSERT_THAT(sut->to_string().c_str(), StrEq(valid_name));
}

TEST(Node, creating_node_name_with_too_long_name_fails) {
    auto invalid_name = std::string(NODE_NAME_LENGTH + 1, 's');
    auto sut = NodeName::create(invalid_name.c_str());

    ASSERT_THAT(sut.has_value(), Eq(false));
    ASSERT_THAT(sut.error(), Eq(SemanticStringError::ExceedsMaximumLength));
}

} // namespace
