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

use iceoryx2::{
    node::NodeView,
    prelude::{ipc, local},
};
use pyo3::prelude::*;

use crate::{
    config::Config, error::NodeCleanupFailure, file_name::FileName, node_id::NodeId,
    node_name::NodeName, parc::Parc,
};

#[derive(Clone)]
pub(crate) enum AliveNodeViewType {
    Ipc(iceoryx2::node::AliveNodeView<ipc::Service>),
    Local(iceoryx2::node::AliveNodeView<local::Service>),
}

#[derive(Clone)]
pub(crate) enum DeadNodeViewType {
    Ipc(iceoryx2::node::DeadNodeView<ipc::Service>),
    Local(iceoryx2::node::DeadNodeView<local::Service>),
}

#[pyclass]
#[derive(Clone)]
pub struct NodeDetails(pub(crate) iceoryx2::node::NodeDetails);

#[pymethods]
impl NodeDetails {
    #[getter]
    pub fn executable(&self) -> FileName {
        FileName(self.0.executable().clone())
    }

    #[getter]
    pub fn name(&self) -> NodeName {
        NodeName(self.0.name().clone())
    }

    #[getter]
    pub fn config(&self) -> Config {
        Config(Parc::new(self.0.config().clone()))
    }
}

#[pyclass]
#[derive(Clone)]
pub struct AliveNodeView(pub(crate) AliveNodeViewType);

#[pymethods]
impl AliveNodeView {
    #[getter]
    pub fn id(&self) -> NodeId {
        match &self.0 {
            AliveNodeViewType::Ipc(n) => NodeId(n.id().clone()),
            AliveNodeViewType::Local(n) => NodeId(n.id().clone()),
        }
    }

    #[getter]
    pub fn details(&self) -> Option<NodeDetails> {
        match &self.0 {
            AliveNodeViewType::Ipc(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
            AliveNodeViewType::Local(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DeadNodeView(pub(crate) DeadNodeViewType);

#[pymethods]
impl DeadNodeView {
    #[getter]
    pub fn id(&self) -> NodeId {
        match &self.0 {
            DeadNodeViewType::Ipc(n) => NodeId(n.id().clone()),
            DeadNodeViewType::Local(n) => NodeId(n.id().clone()),
        }
    }

    #[getter]
    pub fn details(&self) -> Option<NodeDetails> {
        match &self.0 {
            DeadNodeViewType::Ipc(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
            DeadNodeViewType::Local(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
        }
    }

    pub fn remove_stale_resources(&self) -> PyResult<bool> {
        let result = match &self.0 {
            DeadNodeViewType::Ipc(n) => n
                .clone()
                .remove_stale_resources()
                .map_err(|e| NodeCleanupFailure::new_err(format!("{:?}", e)))?,
            DeadNodeViewType::Local(n) => n
                .clone()
                .remove_stale_resources()
                .map_err(|e| NodeCleanupFailure::new_err(format!("{:?}", e)))?,
        };

        Ok(result)
    }
}

#[pyclass]
#[derive(Clone)]
pub enum NodeState {
    Alive(AliveNodeView),
    Dead(DeadNodeView),
    Inaccessible(NodeId),
    Undefined(NodeId),
}
