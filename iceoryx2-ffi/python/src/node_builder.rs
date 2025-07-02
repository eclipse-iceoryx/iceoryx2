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

#[derive(Default)]
#[pyclass(str = "{0:?}")]
/// Creates a new `Node`.
pub struct NodeBuilder(iceoryx2::prelude::NodeBuilder);

#[pymethods]
impl NodeBuilder {
    #[staticmethod]
    /// Instantiates a new `NodeBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    /// The `NodeName` that shall be assigned to the `Node`. It does not
    /// have to be unique. If no `NodeName` is defined then the `Node`
    /// does not have a name.
    pub fn name(&mut self, value: &NodeName) -> Self {
        let this = self.0.clone();
        let this = this.name(&value.0);
        Self(this)
    }

    /// Defines the `SignalHandlingMode` for the `Node`. It affects the `Node.wait()` call
    /// that returns any received signal via its `NodeWaitFailure`
    pub fn signal_handling_mode(&mut self, value: &SignalHandlingMode) -> Self {
        let this = self.0.clone();
        let this = this.signal_handling_mode((value.clone()).into());
        Self(this)
    }

    /// The `Config` that shall be used for the `Node`. If no `Config`
    /// is specified the `config.global_config()` is used.
    pub fn config(&mut self, config: &Config) -> Self {
        let this = self.0.clone();
        let this = this.config(&config.0.lock());
        Self(this)
    }

    /// Creates a new `Node` for a specified `ServiceType`.
    /// Emits `NodeCreationFailure` on failure.
    pub fn create(&mut self, service_type: &ServiceType) -> PyResult<Node> {
        let this = self.0.clone();
        match service_type {
            ServiceType::Ipc => Ok(Node(Parc::new(NodeType::Ipc(
                this.create::<crate::IpcService>()
                    .map_err(|e| NodeCreationFailure::new_err(format!("{e:?}")))?,
            )))),
            ServiceType::Local => Ok(Node(Parc::new(NodeType::Local(
                this.create::<crate::LocalService>()
                    .map_err(|e| NodeCreationFailure::new_err(format!("{e:?}")))?,
            )))),
        }
    }
}
