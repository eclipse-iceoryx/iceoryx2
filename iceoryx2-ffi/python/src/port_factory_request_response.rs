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

use iceoryx2::prelude::{ipc_threadsafe, local_threadsafe, CallbackProgression, PortFactory};
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::attribute_set::AttributeSet;
use crate::error::NodeListFailure;
use crate::node_id::NodeId;
use crate::node_state::{
    AliveNodeView, AliveNodeViewType, DeadNodeView, DeadNodeViewType, NodeState,
};
use crate::service_id::ServiceId;
use crate::service_name::ServiceName;
use crate::static_config_request_response::StaticConfigRequestResponse;

pub(crate) enum PortFactoryRequestResponseType {
    Ipc(
        iceoryx2::service::port_factory::request_response::PortFactory<
            ipc_threadsafe::Service,
            [CustomPayloadMarker],
            CustomHeaderMarker,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
    ),
    Local(
        iceoryx2::service::port_factory::request_response::PortFactory<
            local_threadsafe::Service,
            [CustomPayloadMarker],
            CustomHeaderMarker,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
    ),
}

#[pyclass]
/// The factory for `MessagingPattern::RequestResponse`. It can acquire dynamic and static service
/// informations and create `Client` or `Server` ports.
pub struct PortFactoryRequestResponse(pub(crate) PortFactoryRequestResponseType);

#[pymethods]
impl PortFactoryRequestResponse {
    #[getter]
    /// Returns the `ServiceName` of the service
    pub fn name(&self) -> ServiceName {
        match &self.0 {
            PortFactoryRequestResponseType::Ipc(v) => ServiceName(v.name().clone()),
            PortFactoryRequestResponseType::Local(v) => ServiceName(v.name().clone()),
        }
    }

    #[getter]
    /// Returns the `ServiceId` of the `Service`
    pub fn service_id(&self) -> ServiceId {
        match &self.0 {
            PortFactoryRequestResponseType::Ipc(v) => ServiceId(v.service_id().clone()),
            PortFactoryRequestResponseType::Local(v) => ServiceId(v.service_id().clone()),
        }
    }

    #[getter]
    /// Returns the `AttributeSet` defined in the `Service`
    pub fn attributes(&self) -> AttributeSet {
        match &self.0 {
            PortFactoryRequestResponseType::Ipc(v) => AttributeSet(v.attributes().clone()),
            PortFactoryRequestResponseType::Local(v) => AttributeSet(v.attributes().clone()),
        }
    }

    #[getter]
    /// Returns the StaticConfig of the `Service`.
    /// Contains all settings that never change during the lifetime of the service.
    pub fn static_config(&self) -> StaticConfigRequestResponse {
        match &self.0 {
            PortFactoryRequestResponseType::Ipc(v) => {
                StaticConfigRequestResponse(v.static_config().clone())
            }
            PortFactoryRequestResponseType::Local(v) => {
                StaticConfigRequestResponse(v.static_config().clone())
            }
        }
    }

    #[getter]
    /// Returns a list of all `NodeState` of all the `Node`s which have opened the `Service`.
    pub fn nodes(&self) -> PyResult<Vec<NodeState>> {
        match &self.0 {
            PortFactoryRequestResponseType::Ipc(v) => {
                let mut ret_val = vec![];
                v.nodes(|state| {
                    match state {
                        iceoryx2::prelude::NodeState::Alive(n) => {
                            ret_val.push(NodeState::Alive(AliveNodeView(AliveNodeViewType::Ipc(n))))
                        }
                        iceoryx2::prelude::NodeState::Dead(n) => {
                            ret_val.push(NodeState::Dead(DeadNodeView(DeadNodeViewType::Ipc(n))))
                        }
                        iceoryx2::prelude::NodeState::Inaccessible(n) => {
                            ret_val.push(NodeState::Inaccessible(NodeId(n)))
                        }
                        iceoryx2::prelude::NodeState::Undefined(n) => {
                            ret_val.push(NodeState::Undefined(NodeId(n)))
                        }
                    }
                    CallbackProgression::Continue
                })
                .map_err(|e| NodeListFailure::new_err(format!("{:?}", e)))?;
                Ok(ret_val)
            }
            PortFactoryRequestResponseType::Local(v) => {
                let mut ret_val = vec![];
                v.nodes(|state| {
                    match state {
                        iceoryx2::prelude::NodeState::Alive(n) => ret_val
                            .push(NodeState::Alive(AliveNodeView(AliveNodeViewType::Local(n)))),
                        iceoryx2::prelude::NodeState::Dead(n) => {
                            ret_val.push(NodeState::Dead(DeadNodeView(DeadNodeViewType::Local(n))))
                        }
                        iceoryx2::prelude::NodeState::Inaccessible(n) => {
                            ret_val.push(NodeState::Inaccessible(NodeId(n)))
                        }
                        iceoryx2::prelude::NodeState::Undefined(n) => {
                            ret_val.push(NodeState::Undefined(NodeId(n)))
                        }
                    }
                    CallbackProgression::Continue
                })
                .map_err(|e| NodeListFailure::new_err(format!("{:?}", e)))?;
                Ok(ret_val)
            }
        }
    }
}
