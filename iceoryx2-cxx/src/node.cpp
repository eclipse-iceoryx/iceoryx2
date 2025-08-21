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
#include "iox2/internal/callback_context.hpp"

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
auto Node<T>::signal_handling_mode() const -> SignalHandlingMode {
    return iox::into<SignalHandlingMode>(static_cast<int>(iox2_node_signal_handling_mode(&m_handle)));
}

template <ServiceType T>
auto Node<T>::name() const -> NodeNameView {
    const auto* node_name_ptr = iox2_node_name(&m_handle);
    return NodeNameView { node_name_ptr };
}

template <ServiceType T>
auto Node<T>::config() const -> ConfigView {
    const auto* config_ptr = iox2_node_config(&m_handle);
    return ConfigView { config_ptr };
}

template <ServiceType T>
auto Node<T>::id() const -> NodeId {
    const auto* node_id_ptr = iox2_node_id(&m_handle, iox::into<iox2_service_type_e>(T));
    iox2_node_id_h node_id_handle = nullptr;
    iox2_node_id_clone_from_ptr(nullptr, node_id_ptr, &node_id_handle);
    return NodeId(node_id_handle);
}

template <ServiceType T>
auto Node<T>::wait(iox::units::Duration cycle_time) const -> iox::expected<void, NodeWaitFailure> {
    auto time = cycle_time.timespec();

    auto result = iox2_node_wait(&m_handle, time.tv_sec, time.tv_nsec);
    if (result == IOX2_OK) {
        return iox::ok();
    }
    return iox::err(iox::into<NodeWaitFailure>(result));
}

template <ServiceType T>
auto Node<T>::service_builder(const ServiceName& name) const -> ServiceBuilder<T> {
    return ServiceBuilder<T> { &m_handle, name.as_view().m_ptr };
}

template <ServiceType T>
auto Node<T>::list(ConfigView config, const iox::function<CallbackProgression(NodeState<T>)>& callback)
    -> iox::expected<void, NodeListFailure> {
    auto ctx = internal::ctx(callback);

    const auto ret_val = iox2_node_list(
        iox::into<iox2_service_type_e>(T), config.m_ptr, internal::list_callback<T>, static_cast<void*>(&ctx));

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
    if (m_name.has_value()) {
        const auto* name_ptr = iox2_cast_node_name_ptr(m_name->m_handle);
        iox2_node_builder_set_name(&m_handle, name_ptr);
    }

    if (m_config.has_value()) {
        iox2_node_builder_set_config(&m_handle, &m_config.value().m_handle);
    }

    if (m_signal_handling_mode.has_value()) {
        iox2_node_builder_set_signal_handling_mode(
            &m_handle, iox::into<iox2_signal_handling_mode_e>(m_signal_handling_mode.value()));
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
