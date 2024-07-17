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

#include "test.hpp"

namespace {
using namespace iox2;

TEST(Node, node_name_is_applied) {
    const auto* name_value = "First time we met, I saw the ocean, it was wet!";
    auto node_name = NodeName::create(name_value).expect("");

    auto sut = NodeBuilder().name(node_name).create<ServiceType::Local>().expect("");
    // ASSERT_THAT(sut.name().to_string(), Eq(node_name.to_string()));
}
} // namespace
