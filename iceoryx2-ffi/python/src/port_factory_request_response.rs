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

use iceoryx2::prelude::{CallbackProgression, PortFactory};
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use pyo3::prelude::*;

use crate::attribute_set::AttributeSet;
use crate::error::NodeListFailure;
use crate::node_id::NodeId;
use crate::node_state::{
    AliveNodeView, AliveNodeViewType, DeadNodeView, DeadNodeViewType, NodeState,
};
use crate::parc::Parc;
use crate::port_factory_client::PortFactoryClient;
use crate::port_factory_server::PortFactoryServer;
use crate::service_id::ServiceId;
use crate::service_name::ServiceName;
use crate::static_config_request_response::StaticConfigRequestResponse;
use crate::type_storage::TypeStorage;

#[derive(Clone)]
pub(crate) enum PortFactoryRequestResponseType {
    Ipc(
        iceoryx2::service::port_factory::request_response::PortFactory<
            crate::IpcService,
            [CustomPayloadMarker],
            CustomHeaderMarker,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
    ),
    Local(
        iceoryx2::service::port_factory::request_response::PortFactory<
            crate::LocalService,
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
pub struct PortFactoryRequestResponse {
    pub(crate) value: Parc<PortFactoryRequestResponseType>,
    pub(crate) request_payload_type_details: TypeStorage,
    pub(crate) response_payload_type_details: TypeStorage,
    pub(crate) request_header_type_details: TypeStorage,
    pub(crate) response_header_type_details: TypeStorage,
}

#[pymethods]
impl PortFactoryRequestResponse {
    #[getter]
    /// Returns the `ServiceName` of the service
    pub fn name(&self) -> ServiceName {
        match &*self.value.lock() {
            PortFactoryRequestResponseType::Ipc(v) => ServiceName(v.name().clone()),
            PortFactoryRequestResponseType::Local(v) => ServiceName(v.name().clone()),
        }
    }

    #[getter]
    /// Returns the `ServiceId` of the `Service`
    pub fn service_id(&self) -> ServiceId {
        match &*self.value.lock() {
            PortFactoryRequestResponseType::Ipc(v) => ServiceId(v.service_id().clone()),
            PortFactoryRequestResponseType::Local(v) => ServiceId(v.service_id().clone()),
        }
    }

    #[getter]
    /// Returns the `AttributeSet` defined in the `Service`
    pub fn attributes(&self) -> AttributeSet {
        match &*self.value.lock() {
            PortFactoryRequestResponseType::Ipc(v) => AttributeSet(v.attributes().clone()),
            PortFactoryRequestResponseType::Local(v) => AttributeSet(v.attributes().clone()),
        }
    }

    #[getter]
    /// Returns the StaticConfig of the `Service`.
    /// Contains all settings that never change during the lifetime of the service.
    pub fn static_config(&self) -> StaticConfigRequestResponse {
        match &*self.value.lock() {
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
        match &*self.value.lock() {
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
                .map_err(|e| NodeListFailure::new_err(format!("{e:?}")))?;
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
                .map_err(|e| NodeListFailure::new_err(format!("{e:?}")))?;
                Ok(ret_val)
            }
        }
    }

    /// Returns a `PortFactoryServer` to create a new `Server` port
    pub fn server_builder(&self) -> PortFactoryServer {
        PortFactoryServer::new(
            self.value.clone(),
            self.request_payload_type_details.clone(),
            self.response_payload_type_details.clone(),
            self.request_header_type_details.clone(),
            self.response_header_type_details.clone(),
        )
    }

    /// Returns a `PortFactoryClient` to create a new `Client` port
    pub fn client_builder(&self) -> PortFactoryClient {
        PortFactoryClient::new(
            self.value.clone(),
            self.request_payload_type_details.clone(),
            self.response_payload_type_details.clone(),
            self.request_header_type_details.clone(),
            self.response_header_type_details.clone(),
        )
    }
}
