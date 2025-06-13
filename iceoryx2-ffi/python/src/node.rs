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

use iceoryx2::prelude::{ipc, local};
use pyo3::prelude::*;

use crate::{
    cleanup_state::CleanupState, config::Config, duration::Duration, error::NodeWaitFailure,
    node_id::NodeId, node_name::NodeName, parc::Parc, service_type::ServiceType,
    signal_handling_mode::SignalHandlingMode,
};

pub(crate) enum NodeType {
    Ipc(iceoryx2::node::Node<ipc::Service>),
    Local(iceoryx2::node::Node<local::Service>),
}

#[pyclass]
pub struct Node(pub(crate) Parc<NodeType>);

#[pymethods]
impl Node {
    pub fn __str__(&self) -> String {
        match &*self.0.lock() {
            NodeType::Ipc(node) => format!("{:?}", node),
            NodeType::Local(node) => format!("{:?}", node),
        }
    }

    #[getter]
    pub fn name(&self) -> NodeName {
        match &*self.0.lock() {
            NodeType::Ipc(node) => NodeName(node.name().clone()),
            NodeType::Local(node) => NodeName(node.name().clone()),
        }
    }

    #[getter]
    pub fn config(&self) -> Config {
        match &*self.0.lock() {
            NodeType::Ipc(node) => Config(Parc::new(node.config().clone())),
            NodeType::Local(node) => Config(Parc::new(node.config().clone())),
        }
    }

    #[getter]
    pub fn id(&self) -> NodeId {
        match &*self.0.lock() {
            NodeType::Ipc(node) => NodeId(*node.id()),
            NodeType::Local(node) => NodeId(*node.id()),
        }
    }

    #[staticmethod]
    pub fn list(config: &Config) -> PyResult<()> {
        todo!()
    }

    pub fn wait(&self, cycle_time: &Duration) -> PyResult<()> {
        match &*self.0.lock() {
            NodeType::Ipc(node) => node
                .wait(cycle_time.0)
                .map_err(|e| NodeWaitFailure::new_err(format!("{:?}", e)))?,
            NodeType::Local(node) => node
                .wait(cycle_time.0)
                .map_err(|e| NodeWaitFailure::new_err(format!("{:?}", e)))?,
        };

        Ok(())
    }

    #[getter]
    pub fn signal_handling_mode(&self) -> SignalHandlingMode {
        match &*self.0.lock() {
            NodeType::Ipc(node) => node.signal_handling_mode().into(),
            NodeType::Local(node) => node.signal_handling_mode().into(),
        }
    }

    #[staticmethod]
    pub fn cleanup_dead_nodes(service_type: &ServiceType, config: &Config) -> CleanupState {
        match service_type {
            ServiceType::Ipc => CleanupState(
                iceoryx2::prelude::Node::<ipc::Service>::cleanup_dead_nodes(&config.0.lock()),
            ),
            ServiceType::Local => CleanupState(
                iceoryx2::prelude::Node::<local::Service>::cleanup_dead_nodes(&config.0.lock()),
            ),
        }
    }
}
