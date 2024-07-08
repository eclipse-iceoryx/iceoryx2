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

int main() {
    using namespace iox2;
    auto node =
        NodeBuilder()
            .name(NodeName::create("hello world").expect("valid node name"))
            .template create<NodeType::ZERO_COPY>()
            .expect("successful node creation");

    Node<NodeType::ZERO_COPY>::list(
        Config{}, [](auto) { return iox::ok(CallbackProgression::Continue); });

    return 0;
}
