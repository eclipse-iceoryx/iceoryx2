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

#include <iox2/iceoryx2.h>

namespace iox2 {
iox::expected<NodeName, SemanticStringError> NodeName::create(
    const char* value) {}

const std::string& NodeName::as_string() const {}

NodeBuilder::NodeBuilder() : m_handle{iox2_node_builder_new(nullptr)} {}

template <NodeType T>
iox::expected<Node<T>, NodeCreationFailure> NodeBuilder::create() const&& {}
}  // namespace iox2
