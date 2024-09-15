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

use iceoryx2::node::NodeId as IceoryxNodeId;
use iceoryx2::node::NodeState as IceoryxNodeState;
use iceoryx2::node::NodeView as IceoryxNodeView;
use iceoryx2::service::attribute::AttributeSet as IceoryxAttributeSet;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern as IceoryxMessagingPattern;
use iceoryx2::service::Service as IceoryxService;
use iceoryx2::service::ServiceDetails as IceoryxServiceDetails;
use iceoryx2::service::ServiceDynamicDetails as IceoryxServiceDynamicDetails;

#[derive(serde::Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceDescriptor {
    PublishSubscribe(String),
    Event(String),
    Undefined(String),
}

impl<T> From<IceoryxServiceDetails<T>> for ServiceDescriptor
where
    T: IceoryxService,
{
    fn from(service: IceoryxServiceDetails<T>) -> Self {
        match service.static_details.messaging_pattern() {
            IceoryxMessagingPattern::PublishSubscribe(_) => {
                ServiceDescriptor::PublishSubscribe(service.static_details.name().to_string())
            }
            IceoryxMessagingPattern::Event(_) => {
                ServiceDescriptor::Event(service.static_details.name().to_string())
            }
            _ => ServiceDescriptor::Undefined("Undefined".to_string()),
        }
    }
}

pub type ServiceList = Vec<ServiceDescriptor>;

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
    id: IceoryxNodeId,
    name: Option<String>,
    executable: Option<String>,
}

impl<T> From<&IceoryxNodeState<T>> for NodeDescriptor
where
    T: IceoryxService,
{
    fn from(node_state: &IceoryxNodeState<T>) -> Self {
        match node_state {
            IceoryxNodeState::Alive(view) => NodeDescriptor {
                state: NodeState::Alive,
                id: *view.id(),
                name: view
                    .details()
                    .as_ref()
                    .map(|details| details.name().as_str().to_string()),
                executable: view
                    .details()
                    .as_ref()
                    .map(|details| details.executable().to_string()),
            },
            IceoryxNodeState::Dead(view) => NodeDescriptor {
                state: NodeState::Dead,
                id: *view.id(),
                name: view
                    .details()
                    .as_ref()
                    .map(|details| details.name().as_str().to_string()),
                executable: view
                    .details()
                    .as_ref()
                    .map(|details| details.executable().to_string()),
            },
            IceoryxNodeState::Inaccessible(node_id) => NodeDescriptor {
                state: NodeState::Inaccessible,
                id: *node_id,
                name: None,
                executable: None,
            },
            IceoryxNodeState::Undefined(node_id) => NodeDescriptor {
                state: NodeState::Undefined,
                id: *node_id,
                name: None,
                executable: None,
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
