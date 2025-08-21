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
/// Contains all details of a [`Node`] that is alive.
template <ServiceType>
class AliveNodeView {
  public:
    AliveNodeView(const AliveNodeView&) = default;
    AliveNodeView(AliveNodeView&&) = default;
    auto operator=(const AliveNodeView&) -> AliveNodeView& = default;
    auto operator=(AliveNodeView&&) -> AliveNodeView& = default;
    ~AliveNodeView() = default;

    AliveNodeView(NodeId node_id, const iox::optional<NodeDetails>& details);

    /// Returns the [`NodeId`].
    auto id() const -> const NodeId&;

    /// Returns optional [`NodeDetails`] that contains further information about the [`Node`].
    /// Can only be acquired when the process has the access right to read it.
    auto details() const -> const iox::optional<NodeDetails>&;

  private:
    NodeId m_id;
    iox::optional<NodeDetails> m_details;
};

/// Contains all details of a [`Node`] that is dead.
template <ServiceType T>
class DeadNodeView {
  public:
    DeadNodeView(const DeadNodeView&) = default;
    DeadNodeView(DeadNodeView&&) = default;
    auto operator=(const DeadNodeView&) -> DeadNodeView& = default;
    auto operator=(DeadNodeView&&) -> DeadNodeView& = default;
    ~DeadNodeView() = default;

    explicit DeadNodeView(const AliveNodeView<T>& view);

    /// Returns the [`NodeId`].
    auto id() const -> const NodeId&;

    /// Returns a optional [`NodeDetails`] that contains further information about the [`Node`].
    /// Can only be acquired when the process has the access right to read it.
    auto details() const -> iox::optional<NodeDetails>;

    /// Removes all stale resources of the dead [`Node`]. On error it returns a [`NodeCleanupFailure`].
    /// It returns true if the stale resources could be removed, otherwise false.
    auto remove_stale_resources() -> iox::expected<bool, NodeCleanupFailure>;

  private:
    AliveNodeView<T> m_view;
};

/// Describes the state of a [`Node`].
template <ServiceType T>
class NodeState {
  public:
    NodeState(const NodeState&) = default;
    NodeState(NodeState&&) = default;
    auto operator=(const NodeState&) -> NodeState& = default;
    auto operator=(NodeState&&) -> NodeState& = default;
    ~NodeState() = default;

    /// If the [`Node`] is alive the provided callback is called with an [`AliveNodeView`]
    /// as argument.
    auto alive(const iox::function<void(AliveNodeView<T>&)>& callback) -> NodeState&;

    /// If the [`Node`] is dead the provided callback is called with a [`DeadNodeView`] as
    /// argument.
    auto dead(const iox::function<void(DeadNodeView<T>&)>& callback) -> NodeState&;

    /// If the [`Node`] is inaccessible due to a lack of permissions the provided callback is
    /// called with a [`NodeId`] as argument.
    auto inaccessible(const iox::function<void(NodeId&)>& callback) -> NodeState&;

    /// If the [`Node`] is files are corrupted or some essential constructs are missing the
    /// provided callback is called with a [`NodeId`] as argument.
    auto undefined(const iox::function<void(NodeId&)>& callback) -> NodeState&;

  private:
    template <ServiceType>
    friend auto internal::list_callback(
        iox2_node_state_e, iox2_node_id_ptr, const char*, iox2_node_name_ptr, iox2_config_ptr, iox2_callback_context)
        -> iox2_callback_progression_e;

    explicit NodeState(const AliveNodeView<T>& view);
    explicit NodeState(const DeadNodeView<T>& view);
    NodeState(iox2_node_state_e node_state, const NodeId& node_id);

    iox::variant<AliveNodeView<T>, DeadNodeView<T>, NodeId, NodeId> m_state;
};
} // namespace iox2

#endif
