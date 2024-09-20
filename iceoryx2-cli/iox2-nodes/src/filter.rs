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

use crate::cli::NodeIdentifier;
use crate::cli::OutputFilter;
use crate::cli::StateFilter;
use iceoryx2::node::NodeState;
use iceoryx2::node::NodeView;
use iceoryx2::service::ipc::Service;
use iceoryx2_cli_utils::output::NodeIdString;
use iceoryx2_cli_utils::Filter;

impl Filter<NodeState<Service>> for NodeIdentifier {
    fn matches(&self, node: &NodeState<Service>) -> bool {
        match self {
            NodeIdentifier::Name(ref name) => match node {
                NodeState::Alive(view) => view
                    .details()
                    .as_ref()
                    .map(|details| details.name().as_str() == name)
                    .unwrap_or(false),
                NodeState::Dead(view) => view
                    .details()
                    .as_ref()
                    .map(|details| details.name().as_str() == name)
                    .unwrap_or(false),
                NodeState::Inaccessible(_) | NodeState::Undefined(_) => false,
            },
            NodeIdentifier::Id(ref id) => match node {
                NodeState::Alive(view) => NodeIdString::from(view.id()) == **id,
                NodeState::Dead(view) => NodeIdString::from(view.id()) == **id,
                NodeState::Inaccessible(node_id) => NodeIdString::from(node_id) == **id,
                NodeState::Undefined(node_id) => NodeIdString::from(node_id) == **id,
            },
            NodeIdentifier::Pid(pid) => match node {
                NodeState::Alive(view) => view.id().pid().value() == *pid,
                NodeState::Dead(view) => view.id().pid().value() == *pid,
                NodeState::Inaccessible(node_id) => node_id.pid().value() == *pid,
                NodeState::Undefined(node_id) => node_id.pid().value() == *pid,
            },
        }
    }
}

impl Filter<NodeState<Service>> for StateFilter {
    fn matches(&self, node: &NodeState<Service>) -> bool {
        matches!(
            (self, node),
            (StateFilter::Alive, NodeState::Alive(_))
                | (StateFilter::Dead, NodeState::Dead(_))
                | (StateFilter::Inaccessible, NodeState::Inaccessible(_))
                | (StateFilter::Undefined, NodeState::Undefined(_))
                | (StateFilter::All, _)
        )
    }
}

impl Filter<NodeState<Service>> for OutputFilter {
    fn matches(&self, node: &NodeState<Service>) -> bool {
        self.state.matches(node)
    }
}
