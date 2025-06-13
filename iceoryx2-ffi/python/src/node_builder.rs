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

use iceoryx2::prelude::*;
use pyo3::prelude::*;

use crate::{
    config::Config,
    error::NodeCreationFailure,
    node::{Node, NodeType},
    node_name::NodeName,
    parc::Parc,
    service_type::ServiceType,
    signal_handling_mode::SignalHandlingMode,
};

#[pyclass(str = "{0:?}")]
/// Represent the name for a `Node`.
pub struct NodeBuilder(iceoryx2::prelude::NodeBuilder);

#[pymethods]
impl NodeBuilder {
    #[staticmethod]
    pub fn new() -> Self {
        Self {
            0: iceoryx2::prelude::NodeBuilder::new(),
        }
    }

    pub fn name(&mut self, value: &NodeName) -> Self {
        let this = self.0.clone();
        let this = this.name(&value.0);
        Self(this)
    }

    pub fn signal_handling_mode(&mut self, value: &SignalHandlingMode) -> Self {
        let this = self.0.clone();
        let this = this.signal_handling_mode((value.clone()).into());
        Self(this)
    }

    pub fn config(&mut self, config: &Config) -> Self {
        let this = self.0.clone();
        let this = this.config(&config.0.lock());
        Self(this)
    }

    pub fn create(&mut self, service_type: &ServiceType) -> PyResult<Node> {
        let this = self.0.clone();
        match service_type {
            ServiceType::Ipc => Ok(Node(Parc::new(NodeType::Ipc(
                this.create::<ipc::Service>()
                    .map_err(|e| NodeCreationFailure::new_err(format!("{:?}", e)))?,
            )))),
            ServiceType::Local => Ok(Node(Parc::new(NodeType::Local(
                this.create::<local::Service>()
                    .map_err(|e| NodeCreationFailure::new_err(format!("{:?}", e)))?,
            )))),
        }
    }
}
