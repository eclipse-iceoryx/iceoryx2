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
use pyo3::prelude::*;

use crate::{
    attribute_set::AttributeSet,
    error::NodeListFailure,
    node_id::NodeId,
    node_state::{AliveNodeView, AliveNodeViewType, DeadNodeView, DeadNodeViewType, NodeState},
    parc::Parc,
    port_factory_listener::PortFactoryListener,
    port_factory_notifier::PortFactoryNotifier,
    service_id::ServiceId,
    service_name::ServiceName,
    static_config_event::StaticConfigEvent,
};

pub(crate) enum PortFactoryEventType {
    Ipc(iceoryx2::service::port_factory::event::PortFactory<crate::IpcService>),
    Local(iceoryx2::service::port_factory::event::PortFactory<crate::LocalService>),
}

#[pyclass]
/// The factory for `MessagingPattern::Event`. It can acquire dynamic and static service
/// informations and create `Notifier` or `Listener` ports.
pub struct PortFactoryEvent(pub(crate) Parc<PortFactoryEventType>);

#[pymethods]
impl PortFactoryEvent {
    #[getter]
    /// Returns the `ServiceName` of the service
    pub fn name(&self) -> ServiceName {
        match &*self.0.lock() {
            PortFactoryEventType::Ipc(v) => ServiceName(v.name().clone()),
            PortFactoryEventType::Local(v) => ServiceName(v.name().clone()),
        }
    }

    #[getter]
    /// Returns the `ServiceId` of the `Service`
    pub fn service_id(&self) -> ServiceId {
        match &*self.0.lock() {
            PortFactoryEventType::Ipc(v) => ServiceId(v.service_id().clone()),
            PortFactoryEventType::Local(v) => ServiceId(v.service_id().clone()),
        }
    }

    #[getter]
    /// Returns the `AttributeSet` defined in the `Service`
    pub fn attributes(&self) -> AttributeSet {
        match &*self.0.lock() {
            PortFactoryEventType::Ipc(v) => AttributeSet(v.attributes().clone()),
            PortFactoryEventType::Local(v) => AttributeSet(v.attributes().clone()),
        }
    }

    #[getter]
    /// Returns the StaticConfig of the `Service`.
    /// Contains all settings that never change during the lifetime of the service.
    pub fn static_config(&self) -> StaticConfigEvent {
        match &*self.0.lock() {
            PortFactoryEventType::Ipc(v) => StaticConfigEvent(*v.static_config()),
            PortFactoryEventType::Local(v) => StaticConfigEvent(*v.static_config()),
        }
    }

    #[getter]
    /// Returns a list of all `NodeState` of all the `Node`s which have opened the `Service`.
    pub fn nodes(&self) -> PyResult<Vec<NodeState>> {
        match &*self.0.lock() {
            PortFactoryEventType::Ipc(v) => {
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
            PortFactoryEventType::Local(v) => {
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

    /// Returns a `PortFactoryListener` to create a new `Listener` port
    pub fn listener_builder(&self) -> PortFactoryListener {
        PortFactoryListener::new(self.0.clone())
    }

    /// Returns a `PortFactoryNotifier` to create a new `Notifier` port
    pub fn notifier_builder(&self) -> PortFactoryNotifier {
        PortFactoryNotifier::new(self.0.clone())
    }
}
