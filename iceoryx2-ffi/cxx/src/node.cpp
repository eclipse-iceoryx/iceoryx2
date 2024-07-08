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

#include "iox/into.hpp"

namespace iox {
using namespace ::iox2;
template <>
constexpr NodeCreationFailure
from<iox2_node_creation_failure_e, NodeCreationFailure>(
    iox2_node_creation_failure_e e) noexcept {
    switch (e) {
        case iox2_node_creation_failure_e::
            iox2_node_creation_failure_e_INSUFFICIENT_PERMISSIONS:
            return NodeCreationFailure::InsufficientPermissions;
        case iox2_node_creation_failure_e::
            iox2_node_creation_failure_e_INTERNAL_ERROR:
            return NodeCreationFailure::InternalError;
    }
}
}  // namespace iox

namespace iox2 {
template <NodeType T>
Node<T>::Node(iox2_node_h handle) : m_handle{handle} {}

NodeBuilder::NodeBuilder() : m_handle{iox2_node_builder_new(nullptr)} {}

template <NodeType T>
iox::expected<Node<T>, NodeCreationFailure> NodeBuilder::create() const&& {
    iox2_node_h node_handle;
    int ret_val = iox2_node_builder_create(
        m_handle, nullptr, iox2_service_type_e_IPC, &node_handle);

    if (ret_val != IOX2_OK) {
        return iox::err(
            iox::from<iox2_node_creation_failure_e, NodeCreationFailure>(
                static_cast<iox2_node_creation_failure_e>(ret_val)));
    }

    return iox::ok(Node<T>(node_handle));
}

template iox::expected<Node<NodeType::ZERO_COPY>, NodeCreationFailure>
NodeBuilder::create() const&&;

template iox::expected<Node<NodeType::PROCESS_LOCAL>, NodeCreationFailure>
NodeBuilder::create() const&&;
}  // namespace iox2
