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

#include "callback_progression.hpp"
#include "config.hpp"
#include "internal/iceoryx2.hpp"
#include "node_id.hpp"
#include "node_name.hpp"
#include "node_state.hpp"
#include "service_builder.hpp"
#include "service_name.hpp"
#include "service_type.hpp"

namespace iox2 {
template <ServiceType T>
class Node {
   public:
    NodeName& name() const {}
    NodeId& id() const {}
    ServiceBuilder<T> service_builder(const ServiceName& name) const {}

    static iox::expected<void, NodeListFailure> list(
        const Config& config,
        const iox::function<CallbackProgression(NodeState<T>)>& callback);

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

    template <ServiceType T>
    iox::expected<Node<T>, NodeCreationFailure> create() const&&;

   private:
    iox2_node_builder_h m_handle;
};
}  // namespace iox2

#endif
