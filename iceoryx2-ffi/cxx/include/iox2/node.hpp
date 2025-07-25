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

#ifndef IOX2_NODE_HPP
#define IOX2_NODE_HPP

#include "iox/builder_addendum.hpp"
#include "iox/duration.hpp"
#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "iox2/callback_progression.hpp"
#include "iox2/config.hpp"
#include "iox2/internal/iceoryx2.hpp"
#include "iox2/node_id.hpp"
#include "iox2/node_name.hpp"
#include "iox2/node_state.hpp"
#include "iox2/node_wait_failure.hpp"
#include "iox2/service_builder.hpp"
#include "iox2/service_name.hpp"
#include "iox2/service_type.hpp"
#include "iox2/signal_handling_mode.hpp"

namespace iox2 {
/// The central entry point of iceoryx2. Represents a node of the iceoryx2
/// system. One process can have arbitrary many nodes but usually it should be
/// only one node per process.
/// Can be created via the [`NodeBuilder`].
template <ServiceType T>
class Node {
  public:
    Node(Node&&) noexcept;
    auto operator=(Node&&) noexcept -> Node&;
    Node(const Node&) = delete;
    auto operator=(const Node&) -> Node& = delete;
    ~Node();

    /// Returns the [`Config`] that the [`Node`] will use to create any iceoryx2 entity.
    auto config() const -> ConfigView;

    /// Returns the name of the node inside a [`NodeNameView`].
    auto name() const -> NodeNameView;

    /// Returns the unique id of the [`Node`].
    auto id() const -> NodeId;

    /// Returns the [`ServiceBuilder`] to create a new service. The
    /// [`ServiceName`] of the [`Service`] is provided as argument.
    auto service_builder(const ServiceName& name) const -> ServiceBuilder<T>;

    /// Waits for a given `cycle_time`.
    auto wait(iox::units::Duration cycle_time) const -> iox::expected<void, NodeWaitFailure>;

    /// Lists all [`Node`]s under a provided config. The provided callback is
    /// called for every [`Node`] and gets the [`NodeState`] as input argument.
    /// The callback can return [`CallbackProgression::Stop`] if the iteration
    /// shall stop or [`CallbackProgression::Continue`];
    static auto list(ConfigView config, const iox::function<CallbackProgression(NodeState<T>)>& callback)
        -> iox::expected<void, NodeListFailure>;

    /// Returns the [`SignalHandlingMode`] with which the [`Node`] was created.
    auto signal_handling_mode() const -> SignalHandlingMode;

  private:
    explicit Node(iox2_node_h handle);
    void drop();

    friend class NodeBuilder;

    iox2_node_h m_handle = nullptr;
};

/// Creates a new [`Node`].
class NodeBuilder {
  public:
    /// The [`NodeName`] that shall be assigned to the [`Node`]. It does not
    /// have to be unique. If no [`NodeName`] is defined then the [`Node`]
    /// does not have a name.
#ifdef DOXYGEN_MACRO_FIX
    auto name(const NodeName value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(NodeName, name);
#endif

    /// The [`Config`] that shall be used for the [`Node`]. If no [`Config`]
    /// is specified the [`Config::global_config()`] is used.
#ifdef DOXYGEN_MACRO_FIX
    auto config(const Config value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(Config, config);
#endif

    /// Defines the [`SignalHandlingMode`] for the [`Node`]. It affects the [`Node::wait()`] call
    /// that returns any received signal via its [`NodeWaitFailure`]
#ifdef DOXYGEN_MACRO_FIX
    auto signal_handling_mode(const SignalHandlingMode value) -> decltype(auto);
#else
    IOX_BUILDER_OPTIONAL(SignalHandlingMode, signal_handling_mode);
#endif

  public:
    NodeBuilder();
    NodeBuilder(NodeBuilder&&) = default;
    auto operator=(NodeBuilder&&) -> NodeBuilder& = default;
    ~NodeBuilder() = default;

    NodeBuilder(const NodeBuilder&) = delete;
    auto operator=(const NodeBuilder&) -> NodeBuilder& = delete;

    /// Creates a new [`Node`] for a specified [`ServiceType`].
    template <ServiceType T>
    auto create() const&& -> iox::expected<Node<T>, NodeCreationFailure>;

  private:
    iox2_node_builder_h m_handle = nullptr;
};
} // namespace iox2

#endif
