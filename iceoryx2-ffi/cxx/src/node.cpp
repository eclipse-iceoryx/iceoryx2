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

template <ServiceType T>
Node<T>::Node(Node&& rhs) noexcept
    : m_handle { std::move(rhs.m_handle) } {
    rhs.m_handle = nullptr;
}

template <ServiceType T>
auto Node<T>::operator=(Node&& rhs) noexcept -> Node& {
    if (this != &rhs) {
        drop();
        m_handle = std::move(rhs.m_handle);
        rhs.m_handle = nullptr;
    }

    return *this;
}

template <ServiceType T>
Node<T>::~Node() {
    drop();
}

template <ServiceType T>
auto Node<T>::name() const -> NodeName {
    const auto* const node_name_ptr = iox2_node_name(m_handle);
    size_t name_len = 0;
    const auto* const name_ptr = iox2_node_name_as_c_str(node_name_ptr, &name_len);

    return NodeName::create_impl(name_ptr, name_len).expect("Node shall always contain a valid NodeName");
}

template <ServiceType T>
auto Node<T>::id() const -> NodeId {
    IOX_TODO();
}

template <ServiceType T>
auto Node<T>::wait(const iox::units::Duration& cycle_time) const -> NodeEvent {
    IOX_TODO();
}

template <ServiceType T>
auto Node<T>::service_builder(const ServiceName& name) const -> ServiceBuilder<T> {
    IOX_TODO();
}

template <ServiceType T>
auto list_callback(iox2_node_state_e node_state,
                   iox2_node_id_ptr node_id,
                   iox2_node_name_ptr node_name,
                   iox2_config_ptr config,
                   iox2_node_list_callback_context context) -> iox2_callback_progression_e {
    auto* callback = static_cast<const iox::function<CallbackProgression(NodeState<T>)>*>(context);
    return iox::into<iox2_callback_progression_e>((*callback)(NodeState<T>()));
}

template <ServiceType T>
auto Node<T>::list(ConfigRef config, const iox::function<CallbackProgression(NodeState<T>)>& callback)
    -> iox::expected<void, NodeListFailure> {
    const auto ret_val = iox2_node_list(
        iox::into<iox2_service_type_e>(T), config.m_ptr, list_callback<T>, static_cast<const void*>(&callback));

    if (ret_val == IOX2_OK) {
        return iox::ok();
    }

    return iox::err(iox::into<NodeListFailure>(ret_val));
}

template <ServiceType T>
void Node<T>::drop() {
    if (m_handle != nullptr) {
        iox2_node_drop(m_handle);
        m_handle = nullptr;
    }
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

    return iox::err(iox::into<NodeCreationFailure>(ret_val));
}

template class Node<ServiceType::Ipc>;
template class Node<ServiceType::Local>;

template auto NodeBuilder::create() const&& -> iox::expected<Node<ServiceType::Ipc>, NodeCreationFailure>;
template auto NodeBuilder::create() const&& -> iox::expected<Node<ServiceType::Local>, NodeCreationFailure>;

} // namespace iox2
