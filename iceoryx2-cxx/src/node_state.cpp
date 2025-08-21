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

#include "iox2/node_state.hpp"

namespace iox2 {
constexpr uint64_t ALIVE_STATE = 0;
constexpr uint64_t DEAD_STATE = 1;
constexpr uint64_t INACCESSIBLE_STATE = 2;
constexpr uint64_t UNDEFINED_STATE = 3;

template <ServiceType T>
AliveNodeView<T>::AliveNodeView(NodeId node_id, const iox::optional<NodeDetails>& details)
    : m_id { std::move(node_id) }
    , m_details { details } {
}

template <ServiceType T>
auto AliveNodeView<T>::id() const -> const NodeId& {
    return m_id;
}

template <ServiceType T>
auto AliveNodeView<T>::details() const -> const iox::optional<NodeDetails>& {
    return m_details;
}

template <ServiceType T>
DeadNodeView<T>::DeadNodeView(const AliveNodeView<T>& view)
    : m_view { view } {
}

template <ServiceType T>
auto DeadNodeView<T>::id() const -> const NodeId& {
    return m_view.id();
}

template <ServiceType T>
auto DeadNodeView<T>::details() const -> iox::optional<NodeDetails> {
    return m_view.details();
}

template <ServiceType T>
auto DeadNodeView<T>::remove_stale_resources() -> iox::expected<bool, NodeCleanupFailure> {
    bool has_success = false;
    auto result = iox2_dead_node_remove_stale_resources(iox::into<iox2_service_type_e>(T),
                                                        &m_view.id().m_handle,
                                                        &m_view.details().value().config().m_handle,
                                                        &has_success);

    if (result == IOX2_OK) {
        return iox::ok(has_success);
    }

    return iox::err(iox::into<NodeCleanupFailure>(result));
}

template <ServiceType T>
NodeState<T>::NodeState(const AliveNodeView<T>& view)
    : m_state { view } {
}

template <ServiceType T>
NodeState<T>::NodeState(const DeadNodeView<T>& view)
    : m_state { view } {
}

template <ServiceType T>
NodeState<T>::NodeState(iox2_node_state_e node_state, const NodeId& node_id) {
    switch (node_state) {
    case iox2_node_state_e_INACCESSIBLE:
        m_state.template emplace_at_index<INACCESSIBLE_STATE>(node_id);
        break;
    case iox2_node_state_e_UNDEFINED:
        m_state.template emplace_at_index<UNDEFINED_STATE>(node_id);
        break;
    default:
        IOX_UNREACHABLE();
    }
}

template <ServiceType T>
auto NodeState<T>::alive(const iox::function<void(AliveNodeView<T>&)>& callback) -> NodeState& {
    if (m_state.index() == ALIVE_STATE) {
        callback(*m_state.template get_at_index<ALIVE_STATE>());
    }

    return *this;
}

template <ServiceType T>
auto NodeState<T>::dead(const iox::function<void(DeadNodeView<T>&)>& callback) -> NodeState& {
    if (m_state.index() == DEAD_STATE) {
        callback(*m_state.template get_at_index<DEAD_STATE>());
    }

    return *this;
}

template <ServiceType T>
auto NodeState<T>::inaccessible(const iox::function<void(NodeId&)>& callback) -> NodeState& {
    if (m_state.index() == INACCESSIBLE_STATE) {
        callback(*m_state.template get_at_index<INACCESSIBLE_STATE>());
    }

    return *this;
}

template <ServiceType T>
auto NodeState<T>::undefined(const iox::function<void(NodeId&)>& callback) -> NodeState& {
    if (m_state.index() == UNDEFINED_STATE) {
        callback(*m_state.template get_at_index<UNDEFINED_STATE>());
    }

    return *this;
}

template class NodeState<ServiceType::Ipc>;
template class NodeState<ServiceType::Local>;

template class DeadNodeView<ServiceType::Ipc>;
template class DeadNodeView<ServiceType::Local>;

template class AliveNodeView<ServiceType::Ipc>;
template class AliveNodeView<ServiceType::Local>;
} // namespace iox2
