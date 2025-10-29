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
use iceoryx2::service::builder::CustomKeyMarker;
use pyo3::prelude::*;

use crate::attribute_set::AttributeSet;
use crate::error::NodeListFailure;
use crate::node_id::NodeId;
use crate::node_state::{
    AliveNodeView, AliveNodeViewType, DeadNodeView, DeadNodeViewType, NodeState,
};
use crate::parc::Parc;
use crate::port_factory_reader::PortFactoryReader;
use crate::port_factory_writer::PortFactoryWriter;
use crate::service_id::ServiceId;
use crate::service_name::ServiceName;
use crate::static_config_blackboard::StaticConfigBlackboard;
use crate::type_storage::TypeStorage;

pub(crate) enum PortFactoryBlackboardType {
    Ipc(
        iceoryx2::service::port_factory::blackboard::PortFactory<
            crate::IpcService,
            CustomKeyMarker,
        >,
    ),
    Local(
        iceoryx2::service::port_factory::blackboard::PortFactory<
            crate::LocalService,
            CustomKeyMarker,
        >,
    ),
}

#[pyclass]
pub struct PortFactoryBlackboard {
    pub(crate) value: Parc<PortFactoryBlackboardType>,
    key_type_details: TypeStorage,
}

impl PortFactoryBlackboard {
    pub(crate) fn new(value: PortFactoryBlackboardType, key_type_details: TypeStorage) -> Self {
        Self {
            value: Parc::new(value),
            key_type_details,
        }
    }
}

#[pymethods]
impl PortFactoryBlackboard {
    #[getter]
    pub fn name(&self) -> ServiceName {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(v) => ServiceName(v.name().clone()),
            PortFactoryBlackboardType::Local(v) => ServiceName(v.name().clone()),
        }
    }

    #[getter]
    /// Returns the `ServiceId` of the `Service`
    pub fn service_id(&self) -> ServiceId {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(v) => ServiceId(v.service_id().clone()),
            PortFactoryBlackboardType::Local(v) => ServiceId(v.service_id().clone()),
        }
    }

    #[getter]
    /// Returns the `AttributeSet` defined in the `Service`
    pub fn attributes(&self) -> AttributeSet {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(v) => AttributeSet(v.attributes().clone()),
            PortFactoryBlackboardType::Local(v) => AttributeSet(v.attributes().clone()),
        }
    }

    #[getter]
    /// Returns the StaticConfig of the `Service`.
    /// Contains all settings that never change during the lifetime of the service.
    pub fn static_config(&self) -> StaticConfigBlackboard {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(v) => StaticConfigBlackboard(v.static_config().clone()),
            PortFactoryBlackboardType::Local(v) => {
                StaticConfigBlackboard(v.static_config().clone())
            }
        }
    }

    #[getter]
    /// Returns a list of all `NodeState` of all the `Node`s which have opened the `Service`.
    pub fn nodes(&self) -> PyResult<Vec<NodeState>> {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(v) => {
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
            PortFactoryBlackboardType::Local(v) => {
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

    /// Returns a `PortFactoryWriter` to create a new `Writer` port
    pub fn writer_builder(&self) -> PortFactoryWriter {
        PortFactoryWriter::new(self.value.clone(), self.key_type_details.clone())
    }

    /// Returns a `PortFactoryReader` to create a new `Reader` port
    pub fn reader_builder(&self) -> PortFactoryReader {
        PortFactoryReader::new(self.value.clone(), self.key_type_details.clone())
    }
}
