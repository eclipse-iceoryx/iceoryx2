// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_log::fatal_panic;
use pyo3::prelude::*;

use crate::{
    attribute_set::AttributeSet,
    messaging_pattern::MessagingPattern,
    node_id::NodeId,
    node_state::{AliveNodeView, AliveNodeViewType, DeadNodeView, DeadNodeViewType, NodeState},
    service_id::ServiceId,
    service_name::ServiceName,
};

#[derive(Debug)]
pub(crate) enum ServiceDetailsType {
    Ipc(iceoryx2::service::ServiceDetails<crate::IpcService>),
    Local(iceoryx2::service::ServiceDetails<crate::LocalService>),
}

#[pyclass(str = "{0:#?}")]
/// Represents all the `Service` information that one can acquire with `Service::list()`.
pub struct ServiceDetails(pub(crate) ServiceDetailsType);

#[pymethods]
impl ServiceDetails {
    /// A list of all `Node`s that are registered at the `Service`
    pub fn nodes(&self) -> Vec<NodeState> {
        let mut ret_val = vec![];
        match &self.0 {
            ServiceDetailsType::Ipc(v) => {
                if let Some(details) = &v.dynamic_details {
                    for node in &details.nodes {
                        match node {
                            iceoryx2::node::NodeState::Alive(v) => ret_val.push(NodeState::Alive(
                                AliveNodeView(AliveNodeViewType::Ipc(v.clone())),
                            )),
                            iceoryx2::node::NodeState::Dead(v) => ret_val.push(NodeState::Dead(
                                DeadNodeView(DeadNodeViewType::Ipc(v.clone())),
                            )),
                            iceoryx2::node::NodeState::Inaccessible(v) => {
                                ret_val.push(NodeState::Inaccessible(NodeId(*v)))
                            }
                            iceoryx2::node::NodeState::Undefined(v) => {
                                ret_val.push(NodeState::Undefined(NodeId(*v)))
                            }
                        }
                    }
                }
            }
            ServiceDetailsType::Local(v) => {
                if let Some(details) = &v.dynamic_details {
                    for node in &details.nodes {
                        match node {
                            iceoryx2::node::NodeState::Alive(v) => ret_val.push(NodeState::Alive(
                                AliveNodeView(AliveNodeViewType::Local(v.clone())),
                            )),
                            iceoryx2::node::NodeState::Dead(v) => ret_val.push(NodeState::Dead(
                                DeadNodeView(DeadNodeViewType::Local(v.clone())),
                            )),
                            iceoryx2::node::NodeState::Inaccessible(v) => {
                                ret_val.push(NodeState::Inaccessible(NodeId(*v)))
                            }
                            iceoryx2::node::NodeState::Undefined(v) => {
                                ret_val.push(NodeState::Undefined(NodeId(*v)))
                            }
                        }
                    }
                }
            }
        }

        ret_val
    }

    /// Returns the attributes of the `Service`
    pub fn attributes(&self) -> AttributeSet {
        match &self.0 {
            ServiceDetailsType::Ipc(v) => AttributeSet(v.static_details.attributes().clone()),
            ServiceDetailsType::Local(v) => AttributeSet(v.static_details.attributes().clone()),
        }
    }

    /// Returns the unique `ServiceId` of the `Service`
    pub fn service_id(&self) -> ServiceId {
        match &self.0 {
            ServiceDetailsType::Ipc(v) => ServiceId(v.static_details.service_id().clone()),
            ServiceDetailsType::Local(v) => ServiceId(v.static_details.service_id().clone()),
        }
    }

    /// Returns the `ServiceName`
    pub fn name(&self) -> ServiceName {
        match &self.0 {
            ServiceDetailsType::Ipc(v) => ServiceName(v.static_details.name().clone()),
            ServiceDetailsType::Local(v) => ServiceName(v.static_details.name().clone()),
        }
    }

    /// Returns the `Service`s underlying `MessagingPattern`.
    pub fn messaging_pattern(&self) -> MessagingPattern {
        match &self.0 {
            ServiceDetailsType::Ipc(v) => {
                static_config_messaging_pattern_to_python(v.static_details.messaging_pattern())
            }
            ServiceDetailsType::Local(v) => {
                static_config_messaging_pattern_to_python(v.static_details.messaging_pattern())
            }
        }
    }
}

fn static_config_messaging_pattern_to_python(
    value: &iceoryx2::service::static_config::messaging_pattern::MessagingPattern,
) -> MessagingPattern {
    match value {
        iceoryx2::service::static_config::messaging_pattern::MessagingPattern::RequestResponse(
            _,
        ) => MessagingPattern::RequestResponse,
        iceoryx2::service::static_config::messaging_pattern::MessagingPattern::PublishSubscribe(
            _,
        ) => MessagingPattern::PublishSubscribe,
        iceoryx2::service::static_config::messaging_pattern::MessagingPattern::Event(_) => {
            MessagingPattern::Event
        }
        _ => {
            fatal_panic!(from "ServiceDetails::messaging_pattern()", "Unknown messaging pattern in translation." )
        }
    }
}
