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
#include "iox/into.hpp"

namespace iox2 {
template <ServiceType T>
Node<T>::Node(iox2_node_h handle)
    : m_handle { handle } {
}

NodeBuilder::NodeBuilder()
    : m_handle { iox2_node_builder_new(nullptr) } {
}

template <ServiceType T>
auto NodeBuilder::create() const&& -> iox::expected<Node<T>, NodeCreationFailure> {
    auto* handle_ref = iox2_cast_node_builder_ref_h(m_handle);

    if (m_name.has_value()) {
        iox2_node_builder_set_name(handle_ref, m_name->m_handle);
    }

    if (m_config.has_value()) {
        IOX_TODO();
    }

    iox2_node_h node_handle {};
    const auto ret_val = iox2_node_builder_create(m_handle, nullptr, iox::into<iox2_service_type_e>(T), &node_handle);

    if (ret_val == IOX2_OK) {
        return iox::ok(Node<T> { node_handle });
    }

    return iox::err(iox::from<iox2_node_creation_failure_e, NodeCreationFailure>(
        static_cast<iox2_node_creation_failure_e>(ret_val)));
}

template class Node<ServiceType::Ipc>;
template class Node<ServiceType::Local>;

template auto NodeBuilder::create() const&& -> iox::expected<Node<ServiceType::Ipc>, NodeCreationFailure>;
template auto NodeBuilder::create() const&& -> iox::expected<Node<ServiceType::Local>, NodeCreationFailure>;

} // namespace iox2
