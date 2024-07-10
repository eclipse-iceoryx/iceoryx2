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

#ifndef IOX2_NODE_STATE_HPP_
#define IOX2_NODE_STATE_HPP_

#include "iox/function.hpp"
#include "iox/optional.hpp"
#include "node_details.hpp"
#include "node_failure_enums.hpp"
#include "node_id.hpp"
#include "service_type.hpp"

namespace iox2 {

template <ServiceType>
class AliveNodeView {
   public:
    const NodeId& id() const;
    const iox::optional<NodeDetails> details() const;
};

template <ServiceType>
class DeadNodeView {
   public:
    const NodeId& id() const;
    const iox::optional<NodeDetails> details() const;
    iox::expected<bool, NodeCleanupFailure> remove_stale_resources();
};

template <ServiceType T>
class NodeState {
   public:
    NodeState& if_alive(const iox::function<void(AliveNodeView<T>&)>& callback);
    NodeState& is_dead(const iox::function<void(DeadNodeView<T>&)>& callback);
    NodeState& is_inaccessible(const iox::function<void(NodeId&)>& callback);
    NodeState& is_undefined(const iox::function<void(NodeId&)>& callback);
};
}  // namespace iox2

#endif
