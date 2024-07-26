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
auto Node<T>::name() const -> NodeNameView {
    const auto* node_name_ptr = iox2_node_name(iox2_cast_node_ref_h(m_handle));
    return NodeNameView { node_name_ptr };
}

template <ServiceType T>
auto Node<T>::id() const -> NodeId {
    IOX_TODO();
}

template <ServiceType T>
auto Node<T>::wait(iox::units::Duration cycle_time) const -> NodeEvent {
    auto time = cycle_time.timespec();
    return iox::into<NodeEvent>(iox2_node_wait(iox2_cast_node_ref_h(m_handle), time.tv_sec, time.tv_nsec));
}

template <ServiceType T>
auto Node<T>::service_builder(const ServiceName& name) const -> ServiceBuilder<T> {
    auto* ref_handle = iox2_cast_node_ref_h(m_handle);
    return ServiceBuilder<T> { ref_handle, name.as_view().m_ptr };
}

template <ServiceType T>
// NOLINTBEGIN(readability-function-size)
auto list_callback(iox2_node_state_e node_state,
                   iox2_node_id_ptr node_id,
                   iox2_node_name_ptr node_name,
                   iox2_config_ptr config,
                   iox2_callback_context context) -> iox2_callback_progression_e {
    auto node_details = [&] {
        if (node_id == nullptr || config == nullptr) {
            return iox::optional<NodeDetails>();
        }

        return iox::optional<NodeDetails>(NodeDetails { NodeNameView { node_name }.to_owned(), Config {} });
    }();

    auto node_state_object = [&] {
        switch (node_state) {
        case iox2_node_state_e_ALIVE:
            return NodeState<T> { AliveNodeView<T> { NodeId {}, node_details } };
        case iox2_node_state_e_DEAD:
            return NodeState<T> { DeadNodeView<T> { AliveNodeView<T> { NodeId {}, node_details } } };
        case iox2_node_state_e_UNDEFINED:
            return NodeState<T> { iox2_node_state_e_UNDEFINED, NodeId {} };
        case iox2_node_state_e_INACCESSIBLE:
            return NodeState<T> { iox2_node_state_e_INACCESSIBLE, NodeId {} };
        }

        IOX_UNREACHABLE();
    }();

    auto* callback = internal::ctx_cast<iox::function<CallbackProgression(NodeState<T>)>>(context);
    return iox::into<iox2_callback_progression_e>(callback->value()(node_state_object));
}
// NOLINTEND(readability-function-size)

template <ServiceType T>
auto Node<T>::list(ConfigView config, const iox::function<CallbackProgression(NodeState<T>)>& callback)
    -> iox::expected<void, NodeListFailure> {
    auto ctx = internal::ctx(callback);

    const auto ret_val =
        iox2_node_list(iox::into<iox2_service_type_e>(T), config.m_ptr, list_callback<T>, static_cast<void*>(&ctx));

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
        const auto* name_ptr = iox2_cast_node_name_ptr(m_name->m_handle);
        iox2_node_builder_set_name(handle_ref, name_ptr);
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
