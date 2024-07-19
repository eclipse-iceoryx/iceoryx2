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

#ifndef IOX2_NODE_STATE_HPP
#define IOX2_NODE_STATE_HPP

#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox/optional.hpp"
#include "iox/variant.hpp"
#include "node_details.hpp"
#include "node_failure_enums.hpp"
#include "node_id.hpp"
#include "service_type.hpp"

namespace iox2 {

template <ServiceType>
class AliveNodeView {
  public:
    auto id() const -> const NodeId&;
    auto details() const -> const iox::optional<NodeDetails>&;

  private:
    AliveNodeView(const NodeId& node_id, const iox::optional<NodeDetails>& details);

    NodeId m_id;
    iox::optional<NodeDetails> m_details;
};

template <ServiceType T>
class DeadNodeView {
  public:
    auto id() const -> const NodeId&;
    auto details() const -> iox::optional<NodeDetails>;
    auto remove_stale_resources() -> iox::expected<bool, NodeCleanupFailure>;

  private:
    explicit DeadNodeView(const AliveNodeView<T>& view);

    AliveNodeView<T> m_view;
};

template <ServiceType T>
class NodeState {
  public:
    auto alive(const iox::function<void(AliveNodeView<T>&)>& callback) -> NodeState&;
    auto dead(const iox::function<void(DeadNodeView<T>&)>& callback) -> NodeState&;
    auto inaccessible(const iox::function<void(NodeId&)>& callback) -> NodeState&;
    auto undefined(const iox::function<void(NodeId&)>& callback) -> NodeState&;

  private:
    template <ServiceType>
    friend auto list_callback(iox2_node_state_e,
                              iox2_node_id_ptr,
                              iox2_node_name_ptr,
                              iox2_config_ptr,
                              iox2_node_list_callback_context) -> iox2_callback_progression_e;
    explicit NodeState(const AliveNodeView<T>& view);
    explicit NodeState(const DeadNodeView<T>& view);
    NodeState(iox2_node_state_e node_state, const NodeId& node_id);

    iox::variant<AliveNodeView<T>, DeadNodeView<T>, NodeId, NodeId> m_state;
};
} // namespace iox2

#endif
