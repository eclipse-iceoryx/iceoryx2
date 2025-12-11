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
use iceoryx2_log::fatal_panic;
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
        Option<
            iceoryx2::service::port_factory::blackboard::PortFactory<
                crate::IpcService,
                CustomKeyMarker,
            >,
        >,
    ),
    Local(
        Option<
            iceoryx2::service::port_factory::blackboard::PortFactory<
                crate::LocalService,
                CustomKeyMarker,
            >,
        >,
    ),
}

#[pyclass]
/// The factory for `MessagingPattern::Blackboard`. It can acquire dynamic and static service
/// informations and create `Reader` or `Writer` ports.
pub struct PortFactoryBlackboard {
    pub(crate) value: Parc<PortFactoryBlackboardType>,
    key_type_storage: TypeStorage,
}

impl PortFactoryBlackboard {
    pub(crate) fn new(value: PortFactoryBlackboardType, key_type_storage: TypeStorage) -> Self {
        Self {
            value: Parc::new(value),
            key_type_storage,
        }
    }
}

#[pymethods]
impl PortFactoryBlackboard {
    #[getter]
    pub fn __key_type_details(&self) -> Option<Py<PyAny>> {
        self.key_type_storage.clone().value
    }

    #[getter]
    /// Returns the `ServiceName` of the service.
    pub fn name(&self) -> ServiceName {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => ServiceName(v.name().clone()),
            PortFactoryBlackboardType::Local(Some(v)) => ServiceName(v.name().clone()),
            _ => {
                fatal_panic!(from "PortFactoryBlackboard::name()", "Accessing a deleted PortFactoryBlackboard.")
            }
        }
    }

    #[getter]
    /// Returns the `ServiceId` of the `Service`.
    pub fn service_id(&self) -> ServiceId {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => ServiceId(*v.service_id()),
            PortFactoryBlackboardType::Local(Some(v)) => ServiceId(*v.service_id()),
            _ => {
                fatal_panic!(from "PortFactoryBlackboard::service_id()", "Accessing a deleted PortFactoryBlackboard.")
            }
        }
    }

    #[getter]
    /// Returns the `AttributeSet` defined in the `Service`.
    pub fn attributes(&self) -> AttributeSet {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => AttributeSet(v.attributes().clone()),
            PortFactoryBlackboardType::Local(Some(v)) => AttributeSet(v.attributes().clone()),
            _ => {
                fatal_panic!(from "PortFactoryBlackboard::attributes()", "Accessing a deleted PortFactoryBlackboard.")
            }
        }
    }

    #[getter]
    /// Returns the StaticConfig of the `Service`. Contains all settings that never change during
    /// the lifetime of the service.
    pub fn static_config(&self) -> StaticConfigBlackboard {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => {
                StaticConfigBlackboard(v.static_config().clone())
            }
            PortFactoryBlackboardType::Local(Some(v)) => {
                StaticConfigBlackboard(v.static_config().clone())
            }
            _ => {
                fatal_panic!(from "PortFactoryBlackboard::static_config()", "Accessing a deleted PortFactoryBlackboard.")
            }
        }
    }

    #[getter]
    /// Returns a list of all `NodeState` of all the `Node`s which have opened the `Service`.
    pub fn nodes(&self) -> PyResult<Vec<NodeState>> {
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => {
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
            PortFactoryBlackboardType::Local(Some(v)) => {
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
            _ => {
                fatal_panic!(from "PortFactoryBlackboard::nodes()", "Accessing a deleted PortFactoryBlackboard.")
            }
        }
    }

    /// Returns a `PortFactoryWriter` to create a new `Writer` port
    pub fn writer_builder(&self) -> PortFactoryWriter {
        PortFactoryWriter::new(self.value.clone(), self.key_type_storage.clone())
    }

    /// Returns a `PortFactoryReader` to create a new `Reader` port
    pub fn reader_builder(&self) -> PortFactoryReader {
        PortFactoryReader::new(self.value.clone(), self.key_type_storage.clone())
    }

    /// Releases the `PortFactoryBlackboard`.
    ///
    /// After this call the `PortFactoryBlackboard` is no longer usable!
    pub fn delete(&mut self) {
        match *self.value.lock() {
            PortFactoryBlackboardType::Ipc(ref mut v) => {
                v.take();
            }
            PortFactoryBlackboardType::Local(ref mut v) => {
                v.take();
            }
        }
    }

    pub fn __list_keys(&self) -> Vec<usize> {
        let mut keys = Vec::new();
        match &*self.value.lock() {
            PortFactoryBlackboardType::Ipc(Some(v)) => {
                v.__internal_list_keys(|key| {
                    keys.push(key as usize);
                    CallbackProgression::Continue
                });
            }
            PortFactoryBlackboardType::Local(Some(v)) => {
                v.__internal_list_keys(|key| {
                    keys.push(key as usize);
                    CallbackProgression::Continue
                });
            }
            _ => {
                fatal_panic!(from "PortFactoryBlackboard::list_keys()", "Accessing a deleted PortFactoryBlackboard.")
            }
        }
        keys
    }
}
