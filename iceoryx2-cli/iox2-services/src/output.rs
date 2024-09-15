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

use iceoryx2::node::NodeId;
use iceoryx2::node::NodeState;
use iceoryx2::node::NodeView;
use iceoryx2::service::attribute::AttributeSet;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::Service;
use iceoryx2::service::ServiceDetails;
use iceoryx2::service::ServiceDynamicDetails;

#[derive(serde::Serialize, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceDescriptor {
    PublishSubscribe(String),
    Event(String),
    Undefined(String),
}

impl<T> From<ServiceDetails<T>> for ServiceDescriptor
where
    T: iceoryx2::service::Service,
{
    fn from(service: ServiceDetails<T>) -> Self {
        match service.static_details.messaging_pattern() {
            MessagingPattern::PublishSubscribe(_) => {
                ServiceDescriptor::PublishSubscribe(service.static_details.name().to_string())
            }
            MessagingPattern::Event(_) => {
                ServiceDescriptor::Event(service.static_details.name().to_string())
            }
            _ => ServiceDescriptor::Undefined("Undefined".to_string()),
        }
    }
}

pub type ServiceList = Vec<ServiceDescriptor>;

#[derive(serde::Serialize)]
pub enum ServiceNodeState {
    Alive,
    Dead,
    Inaccessible,
    Undefined,
}

#[derive(serde::Serialize)]
pub struct ServiceNodeDetails {
    state: ServiceNodeState,
    id: NodeId,
    name: Option<String>,
    executable: Option<String>,
}

impl<T> From<&NodeState<T>> for ServiceNodeDetails
where
    T: Service,
{
    fn from(node_state: &NodeState<T>) -> Self {
        match node_state {
            NodeState::Alive(view) => ServiceNodeDetails {
                state: ServiceNodeState::Alive,
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
            NodeState::Dead(view) => ServiceNodeDetails {
                state: ServiceNodeState::Dead,
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
            NodeState::Inaccessible(node_id) => ServiceNodeDetails {
                state: ServiceNodeState::Inaccessible,
                id: *node_id,
                name: None,
                executable: None,
            },
            NodeState::Undefined(node_id) => ServiceNodeDetails {
                state: ServiceNodeState::Undefined,
                id: *node_id,
                name: None,
                executable: None,
            },
        }
    }
}

#[derive(serde::Serialize)]
pub struct ServiceNodeList {
    num: usize,
    details: Vec<ServiceNodeDetails>,
}

impl<T> From<&ServiceDynamicDetails<T>> for ServiceNodeList
where
    T: Service,
{
    fn from(details: &ServiceDynamicDetails<T>) -> Self {
        ServiceNodeList {
            num: details.nodes.len(),
            details: details.nodes.iter().map(ServiceNodeDetails::from).collect(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct ServiceDescription {
    pub service_id: String,
    pub service_name: String,
    pub attributes: AttributeSet,
    pub pattern: MessagingPattern,
    pub nodes: Option<ServiceNodeList>,
}

impl<T> From<&ServiceDetails<T>> for ServiceDescription
where
    T: Service,
{
    fn from(service: &ServiceDetails<T>) -> Self {
        let config = &service.static_details;

        ServiceDescription {
            service_id: config.service_id().as_str().to_string(),
            service_name: config.name().as_str().to_string(),
            attributes: config.attributes().clone(),
            pattern: config.messaging_pattern().clone(),
            nodes: service.dynamic_details.as_ref().map(ServiceNodeList::from),
        }
    }
}
