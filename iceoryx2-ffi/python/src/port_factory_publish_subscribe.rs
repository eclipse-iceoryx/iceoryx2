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
use crate::cleanup_state::CleanupState;
use crate::duration::Duration;
use crate::error::NodeListFailure;
use crate::node_state::{
    AliveNodeView, AliveNodeViewType, DeadNodeView, DeadNodeViewType, NodeState,
};
use crate::parc::Parc;
use crate::port_factory_publisher::PortFactoryPublisher;
use crate::port_factory_subscriber::PortFactorySubscriber;
use crate::service_hash::ServiceHash;
use crate::service_name::ServiceName;
use crate::static_config_publish_subscribe::StaticConfigPublishSubscribe;
use crate::type_storage::TypeStorage;
use crate::unique_node_id::UniqueNodeId;

pub(crate) enum PortFactoryPublishSubscribeType {
    Ipc(
        iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            crate::IpcService,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
    ),
    Local(
        iceoryx2::service::port_factory::publish_subscribe::PortFactory<
            crate::LocalService,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
    ),
}

#[pyclass]
/// The factory for `MessagingPattern::PublishSubscribe`. It can acquire dynamic and static service
/// informations and create `Publisher` or `Subscriber` ports.
pub struct PortFactoryPublishSubscribe {
    pub(crate) value: Parc<PortFactoryPublishSubscribeType>,
    payload_type_details: TypeStorage,
    user_header_type_details: TypeStorage,
}

impl PortFactoryPublishSubscribe {
    pub(crate) fn new(
        value: PortFactoryPublishSubscribeType,
        payload_type_details: TypeStorage,
        user_header_type_details: TypeStorage,
    ) -> Self {
        Self {
            value: Parc::new(value),
            payload_type_details,
            user_header_type_details,
        }
    }
}

#[pymethods]
impl PortFactoryPublishSubscribe {
    #[getter]
    /// Returns the `ServiceName` of the service
    pub fn name(&self) -> ServiceName {
        match &*self.value.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => ServiceName(*v.name()),
            PortFactoryPublishSubscribeType::Local(v) => ServiceName(*v.name()),
        }
    }

    #[getter]
    /// Returns the `ServiceHash` of the `Service`
    pub fn service_hash(&self) -> ServiceHash {
        match &*self.value.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => ServiceHash(*v.service_hash()),
            PortFactoryPublishSubscribeType::Local(v) => ServiceHash(*v.service_hash()),
        }
    }

    #[getter]
    /// Returns the `AttributeSet` defined in the `Service`
    pub fn attributes(&self) -> AttributeSet {
        match &*self.value.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => AttributeSet(v.attributes().clone()),
            PortFactoryPublishSubscribeType::Local(v) => AttributeSet(v.attributes().clone()),
        }
    }

    #[getter]
    /// Returns the StaticConfig of the `Service`.
    /// Contains all settings that never change during the lifetime of the service.
    pub fn static_config(&self) -> StaticConfigPublishSubscribe {
        match &*self.value.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => {
                StaticConfigPublishSubscribe(*v.static_config())
            }
            PortFactoryPublishSubscribeType::Local(v) => {
                StaticConfigPublishSubscribe(*v.static_config())
            }
        }
    }

    #[getter]
    /// Returns a list of all `NodeState` of all the `Node`s which have opened the `Service`.
    pub fn nodes(&self) -> PyResult<Vec<NodeState>> {
        match &*self.value.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => {
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
                            ret_val.push(NodeState::Inaccessible(UniqueNodeId(n)))
                        }
                        iceoryx2::prelude::NodeState::Undefined(n) => {
                            ret_val.push(NodeState::Undefined(UniqueNodeId(n)))
                        }
                    }
                    CallbackProgression::Continue
                })
                .map_err(|e| NodeListFailure::new_err(format!("{e:?}")))?;
                Ok(ret_val)
            }
            PortFactoryPublishSubscribeType::Local(v) => {
                let mut ret_val = vec![];
                v.nodes(|state| {
                    match state {
                        iceoryx2::prelude::NodeState::Alive(n) => ret_val
                            .push(NodeState::Alive(AliveNodeView(AliveNodeViewType::Local(n)))),
                        iceoryx2::prelude::NodeState::Dead(n) => {
                            ret_val.push(NodeState::Dead(DeadNodeView(DeadNodeViewType::Local(n))))
                        }
                        iceoryx2::prelude::NodeState::Inaccessible(n) => {
                            ret_val.push(NodeState::Inaccessible(UniqueNodeId(n)))
                        }
                        iceoryx2::prelude::NodeState::Undefined(n) => {
                            ret_val.push(NodeState::Undefined(UniqueNodeId(n)))
                        }
                    }
                    CallbackProgression::Continue
                })
                .map_err(|e| NodeListFailure::new_err(format!("{e:?}")))?;
                Ok(ret_val)
            }
        }
    }

    /// Removes the stale system resources of all dead [`Node`]s connected to this service.
    ///
    /// If a [`Node`] cannot be cleaned up since the process has insufficient permissions or it
    /// is currently being cleaned up by another process then the [`Node`] is skipped.
    pub fn try_cleanup_dead_nodes(&self) -> CleanupState {
        match &*self.value.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => CleanupState(v.try_cleanup_dead_nodes()),
            PortFactoryPublishSubscribeType::Local(v) => CleanupState(v.try_cleanup_dead_nodes()),
        }
    }

    /// Removes the stale system resources of all dead [`Node`]s connected to this service.
    ///
    /// If a [`Node`] cannot be cleaned up since the process has insufficient permissions then the
    /// [`Node`] is skipped. If it is currently being cleaned up by another process then the
    /// cleaner will wait until the timeout as either passed or the cleaned was finished.
    ///
    /// The timeout is applied to every individual dead [`Node`] the function needs to wait on.
    pub fn blocking_cleanup_dead_nodes(&self, timeout: &Duration) -> CleanupState {
        match &*self.value.lock() {
            PortFactoryPublishSubscribeType::Ipc(v) => {
                CleanupState(v.blocking_cleanup_dead_nodes(timeout.0))
            }
            PortFactoryPublishSubscribeType::Local(v) => {
                CleanupState(v.blocking_cleanup_dead_nodes(timeout.0))
            }
        }
    }

    /// Returns a `PortFactoryPublisher` to create a new `Publisher` port
    pub fn publisher_builder(&self) -> PortFactoryPublisher {
        PortFactoryPublisher::new(
            self.value.clone(),
            self.payload_type_details.clone(),
            self.user_header_type_details.clone(),
        )
    }

    /// Returns a `PortFactorySubscriber` to create a new `Subscriber` port
    pub fn subscriber_builder(&self) -> PortFactorySubscriber {
        PortFactorySubscriber::new(
            self.value.clone(),
            self.payload_type_details.clone(),
            self.user_header_type_details.clone(),
        )
    }
}
