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

use core::ops::Deref;

use iceoryx2::node::NodeDetails as IceoryxNodeDetails;
use iceoryx2::node::NodeId as IceoryxNodeId;
use iceoryx2::node::NodeState as IceoryxNodeState;
use iceoryx2::node::NodeView as IceoryxNodeView;
use iceoryx2::service::attribute::AttributeSet as IceoryxAttributeSet;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern as IceoryxMessagingPattern;
use iceoryx2::service::Service as IceoryxService;
use iceoryx2::service::ServiceDetails as IceoryxServiceDetails;
use iceoryx2::service::ServiceDynamicDetails as IceoryxServiceDynamicDetails;
use iceoryx2_pal_posix::posix::pid_t;

#[derive(serde::Serialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceDescriptor {
    PublishSubscribe(String),
    Event(String),
    RequestResponse(String),
    Undefined(String),
}

impl<T> From<&IceoryxServiceDetails<T>> for ServiceDescriptor
where
    T: IceoryxService,
{
    fn from(service: &IceoryxServiceDetails<T>) -> Self {
        match service.static_details.messaging_pattern() {
            IceoryxMessagingPattern::PublishSubscribe(_) => {
                ServiceDescriptor::PublishSubscribe(service.static_details.name().to_string())
            }
            IceoryxMessagingPattern::Event(_) => {
                ServiceDescriptor::Event(service.static_details.name().to_string())
            }
            IceoryxMessagingPattern::RequestResponse(_) => {
                ServiceDescriptor::RequestResponse(service.static_details.name().to_string())
            }
            _ => ServiceDescriptor::Undefined("Undefined".to_string()),
        }
    }
}

#[derive(serde::Serialize)]
pub struct ServiceDescription {
    pub service_id: String,
    pub service_name: String,
    pub attributes: IceoryxAttributeSet,
    pub pattern: IceoryxMessagingPattern,
    pub nodes: Option<NodeList>,
}

impl<T> From<&IceoryxServiceDetails<T>> for ServiceDescription
where
    T: IceoryxService,
{
    fn from(service: &IceoryxServiceDetails<T>) -> Self {
        let config = &service.static_details;

        ServiceDescription {
            service_id: config.service_id().as_str().to_string(),
            service_name: config.name().as_str().to_string(),
            attributes: config.attributes().clone(),
            pattern: config.messaging_pattern().clone(),
            nodes: service.dynamic_details.as_ref().map(NodeList::from),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub struct NodeIdString(String);

impl From<&IceoryxNodeId> for NodeIdString {
    fn from(id: &IceoryxNodeId) -> Self {
        NodeIdString(format!("{:032x}", id.value()))
    }
}

impl AsRef<str> for NodeIdString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for NodeIdString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<str> for NodeIdString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for NodeIdString {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

#[derive(serde::Serialize)]
pub enum NodeState {
    Alive,
    Dead,
    Inaccessible,
    Undefined,
}

#[derive(serde::Serialize)]
pub struct NodeDescriptor {
    state: NodeState,
    id: NodeIdString,
    pid: pid_t,
    executable: Option<String>,
    name: Option<String>,
}

impl<T> From<&IceoryxNodeState<T>> for NodeDescriptor
where
    T: IceoryxService,
{
    fn from(node: &IceoryxNodeState<T>) -> Self {
        match node {
            IceoryxNodeState::Alive(view) => NodeDescriptor {
                state: NodeState::Alive,
                id: NodeIdString::from(view.id()),
                pid: view.id().pid().value(),
                executable: view
                    .details()
                    .as_ref()
                    .map(|details| details.executable().to_string()),
                name: view
                    .details()
                    .as_ref()
                    .map(|details| details.name().as_str().to_string()),
            },
            IceoryxNodeState::Dead(view) => NodeDescriptor {
                state: NodeState::Dead,
                id: NodeIdString::from(view.id()),
                pid: view.id().pid().value(),
                executable: view
                    .details()
                    .as_ref()
                    .map(|details| details.executable().to_string()),
                name: view
                    .details()
                    .as_ref()
                    .map(|details| details.name().as_str().to_string()),
            },
            IceoryxNodeState::Inaccessible(node_id) => NodeDescriptor {
                state: NodeState::Inaccessible,
                id: NodeIdString::from(node_id),
                pid: node_id.pid().value(),
                executable: None,
                name: None,
            },
            IceoryxNodeState::Undefined(node_id) => NodeDescriptor {
                state: NodeState::Undefined,
                id: NodeIdString::from(node_id),
                pid: node_id.pid().value(),
                executable: None,
                name: None,
            },
        }
    }
}

#[derive(serde::Serialize)]
pub struct NodeDescription {
    state: NodeState,
    id: NodeIdString,
    pid: pid_t,
    #[serde(flatten)]
    details: Option<IceoryxNodeDetails>,
}

impl<T> From<&IceoryxNodeState<T>> for NodeDescription
where
    T: IceoryxService,
{
    fn from(node: &IceoryxNodeState<T>) -> Self {
        match node {
            IceoryxNodeState::Alive(view) => NodeDescription {
                state: NodeState::Alive,
                id: NodeIdString::from(view.id()),
                pid: view.id().pid().value(),
                details: view.details().clone(),
            },
            IceoryxNodeState::Dead(view) => NodeDescription {
                state: NodeState::Dead,
                id: NodeIdString::from(view.id()),
                pid: view.id().pid().value(),
                details: view.details().clone(),
            },
            IceoryxNodeState::Inaccessible(node_id) => NodeDescription {
                state: NodeState::Inaccessible,
                id: NodeIdString::from(node_id),
                pid: node_id.pid().value(),
                details: None,
            },
            IceoryxNodeState::Undefined(node_id) => NodeDescription {
                state: NodeState::Undefined,
                id: NodeIdString::from(node_id),
                pid: node_id.pid().value(),
                details: None,
            },
        }
    }
}

#[derive(serde::Serialize)]
pub struct NodeList {
    pub num: usize,
    pub details: Vec<NodeDescriptor>,
}

impl<T> From<&IceoryxServiceDynamicDetails<T>> for NodeList
where
    T: IceoryxService,
{
    fn from(details: &IceoryxServiceDynamicDetails<T>) -> Self {
        NodeList {
            num: details.nodes.len(),
            details: details.nodes.iter().map(NodeDescriptor::from).collect(),
        }
    }
}
