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
#ifndef IOX2_NODE_HPP_
#define IOX2_NODE_HPP_

#include <iox/builder.hpp>
#include <iox/expected.hpp>
#include <iox/function.hpp>
#include <iox/optional.hpp>

#include "internal/iceoryx2.hpp"
#include "node_name.hpp"

namespace iox2 {

enum class NodeListFailure {};

enum class CallbackProgression { Continue, Stop };

enum class NodeCreationFailure { InsufficientPermissions, InternalError };

enum class NodeCleanupFailure {};

enum class NodeType { PROCESS_LOCAL, ZERO_COPY };

class Config {};

class NodeId {};
class ServiceName {
   public:
    static iox::expected<ServiceName, SemanticStringError> create(
        const char* value);
    const std::string& as_string() const;
};

template <NodeType>
class Builder {};

class NodeDetails {
   public:
    const NodeName& name() const;
    const Config& config() const;
};

template <NodeType>
class AliveNodeView {
   public:
    const NodeId& id() const;
    const iox::optional<NodeDetails> details() const;
};

template <NodeType>
class DeadNodeView {
   public:
    const NodeId& id() const;
    const iox::optional<NodeDetails> details() const;
    iox::expected<bool, NodeCleanupFailure> remove_stale_resources();
};

template <NodeType T>
class NodeState {
   public:
    NodeState& if_alive(iox::function<void(AliveNodeView<T>&)> callback);
    NodeState& is_dead(iox::function<void(DeadNodeView<T>&)> callback);
};

template <NodeType T>
class Node {
   public:
    NodeName& name() const;
    NodeId& id() const;
    Builder<T> service_builder(const ServiceName& name) const;

    static iox::expected<void, NodeListFailure> list(
        const Config& config,
        const iox::function<iox::expected<CallbackProgression, NodeListFailure>(
            iox::expected<NodeState<T>, NodeListFailure>)>& callback);

   private:
    friend class NodeBuilder;

    Node(iox2_node_h handle);

    iox2_node_h m_handle;
};

class NodeBuilder {
    IOX_BUILDER_PARAMETER(NodeName, name, NodeName::create("").expect(""))
    IOX_BUILDER_PARAMETER(Config, config, Config{})

   public:
    NodeBuilder();

    template <NodeType T>
    iox::expected<Node<T>, NodeCreationFailure> create() const&&;

   private:
    iox2_node_builder_h m_handle;
};
}  // namespace iox2

#endif
