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

#include "callback_progression.hpp"
#include "config.hpp"
#include "internal/iceoryx2.hpp"
#include "iox/assertions_addendum.hpp"
#include "iox/builder_addendum.hpp"
#include "iox/duration.hpp"
#include "iox/expected.hpp"
#include "iox/function.hpp"
#include "node_id.hpp"
#include "node_name.hpp"
#include "node_state.hpp"
#include "service_builder.hpp"
#include "service_name.hpp"
#include "service_type.hpp"

namespace iox2 {
enum class NodeEvent : uint8_t {
    /// The timeout passed.
    Tick,
    /// SIGTERM signal was received
    TerminationRequest,
    /// SIGINT signal was received
    InterruptSignal,
};

template <ServiceType T>
class Node {
  public:
    Node() = default;
    Node(Node&&) = default;
    auto operator=(Node&&) -> Node& = default;
    ~Node() = default;

    Node(const Node&) = delete;
    auto operator=(const Node&) -> Node& = delete;

    auto name() const -> NodeName& {
        IOX_TODO();
    }
    auto id() const -> NodeId& {
        IOX_TODO();
    }
    auto service_builder(const ServiceName& name) const -> ServiceBuilder<T> {
        IOX_TODO();
    }
    auto wait(const iox::units::Duration& cycle_time) const -> NodeEvent {
        IOX_TODO();
    }

    static auto list(const Config& config, const iox::function<CallbackProgression(NodeState<T>)>& callback)
        -> iox::expected<void, NodeListFailure> {
        IOX_TODO();
    }

  private:
    friend class NodeBuilder;
};

class NodeBuilder {
    IOX_BUILDER_OPTIONAL(NodeName, name);
    IOX_BUILDER_OPTIONAL(Config, config);

  public:
    NodeBuilder() {
        IOX_TODO();
    }

    template <ServiceType T>
    auto create() const&& -> iox::expected<Node<T>, NodeCreationFailure> {
        IOX_TODO();
    }
};
} // namespace iox2

#endif
