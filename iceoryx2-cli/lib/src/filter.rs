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

use crate::output::NodeIdString;
use clap::ValueEnum;
use core::fmt::Debug;
use core::str::FromStr;
use iceoryx2::node::NodeState;
use iceoryx2::node::NodeView;
use iceoryx2::service::ipc::Service;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::ServiceDetails;
use iceoryx2_pal_posix::posix::pid_t;

pub trait Filter<T>: Debug {
    fn matches(&self, item: &T) -> bool;
}

#[derive(Clone, Debug)]
pub enum NodeIdentifier {
    Name(String),
    Id(String),
    Pid(pid_t),
}

impl NodeIdentifier {
    fn is_valid_id(s: &str) -> bool {
        s.len() == 32 && s.chars().all(|c| c.is_ascii_hexdigit())
    }
}

impl FromStr for NodeIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(pid) = s.parse::<pid_t>() {
            Ok(NodeIdentifier::Pid(pid))
        } else if Self::is_valid_id(s) {
            Ok(NodeIdentifier::Id(s.to_string()))
        } else {
            Ok(NodeIdentifier::Name(s.to_string()))
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "PascalCase")]
#[derive(Default)]
pub enum StateFilter {
    Alive,
    Dead,
    Inaccessible,
    Undefined,
    #[default]
    All,
}

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

#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "PascalCase")]
#[derive(Default)]
pub enum MessagingPatternFilter {
    PublishSubscribe,
    Event,
    RequestResponse,
    #[default]
    All,
}

impl Filter<ServiceDetails<Service>> for MessagingPatternFilter {
    fn matches(&self, service: &ServiceDetails<Service>) -> bool {
        match self {
            MessagingPatternFilter::All => true,
            MessagingPatternFilter::PublishSubscribe => {
                matches!(
                    service.static_details.messaging_pattern(),
                    MessagingPattern::PublishSubscribe(_)
                )
            }
            MessagingPatternFilter::Event => {
                matches!(
                    service.static_details.messaging_pattern(),
                    MessagingPattern::Event(_)
                )
            }
            MessagingPatternFilter::RequestResponse => {
                matches!(
                    service.static_details.messaging_pattern(),
                    MessagingPattern::RequestResponse(_)
                )
            }
        }
    }
}
