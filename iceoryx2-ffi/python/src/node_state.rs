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

use iceoryx2::node::NodeView;
use pyo3::prelude::*;

use crate::{
    config::Config, error::NodeCleanupFailure, file_name::FileName, node_id::NodeId,
    node_name::NodeName, parc::Parc,
};

#[derive(Clone)]
pub(crate) enum AliveNodeViewType {
    Ipc(iceoryx2::node::AliveNodeView<crate::IpcService>),
    Local(iceoryx2::node::AliveNodeView<crate::LocalService>),
}

#[derive(Clone)]
pub(crate) enum DeadNodeViewType {
    Ipc(iceoryx2::node::DeadNodeView<crate::IpcService>),
    Local(iceoryx2::node::DeadNodeView<crate::LocalService>),
}

#[pyclass]
#[derive(Clone)]
/// Contains details of a `Node`.
pub struct NodeDetails(pub(crate) iceoryx2::node::NodeDetails);

#[pymethods]
impl NodeDetails {
    #[getter]
    /// Returns the executable `FileName` of the `Node`s owner process.
    pub fn executable(&self) -> FileName {
        FileName(self.0.executable().clone())
    }

    #[getter]
    /// Returns a reference of the `NodeName`.
    pub fn name(&self) -> NodeName {
        NodeName(self.0.name().clone())
    }

    #[getter]
    /// Returns a reference to the `Config` the `Node` uses.
    pub fn config(&self) -> Config {
        Config(Parc::new(self.0.config().clone()))
    }
}

#[pyclass]
#[derive(Clone)]
/// Contains all details of a `Node` that is alive.
pub struct AliveNodeView(pub(crate) AliveNodeViewType);

#[pymethods]
impl AliveNodeView {
    #[getter]
    /// Returns the `NodeId`.
    pub fn id(&self) -> NodeId {
        match &self.0 {
            AliveNodeViewType::Ipc(n) => NodeId(*n.id()),
            AliveNodeViewType::Local(n) => NodeId(*n.id()),
        }
    }

    #[getter]
    /// Returns optional `NodeDetails` that contains further information about the `Node`.
    /// Can only be acquired when the process has the access right to read it.
    pub fn details(&self) -> Option<NodeDetails> {
        match &self.0 {
            AliveNodeViewType::Ipc(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
            AliveNodeViewType::Local(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
        }
    }
}

#[pyclass]
#[derive(Clone)]
/// Contains all details of a `Node` that is dead.
pub struct DeadNodeView(pub(crate) DeadNodeViewType);

#[pymethods]
impl DeadNodeView {
    #[getter]
    /// Returns the `NodeId`.
    pub fn id(&self) -> NodeId {
        match &self.0 {
            DeadNodeViewType::Ipc(n) => NodeId(*n.id()),
            DeadNodeViewType::Local(n) => NodeId(*n.id()),
        }
    }

    #[getter]
    /// Returns optional `NodeDetails` that contains further information about the `Node`.
    /// Can only be acquired when the process has the access right to read it.
    pub fn details(&self) -> Option<NodeDetails> {
        match &self.0 {
            DeadNodeViewType::Ipc(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
            DeadNodeViewType::Local(n) => n.details().as_ref().map(|d| NodeDetails(d.clone())),
        }
    }

    /// Removes all stale resources of the dead `Node`. On error it emits a `NodeCleanupFailure`.
    /// It returns true if the stale resources could be removed, otherwise false.
    pub fn remove_stale_resources(&self) -> PyResult<bool> {
        let result = match &self.0 {
            DeadNodeViewType::Ipc(n) => n
                .clone()
                .remove_stale_resources()
                .map_err(|e| NodeCleanupFailure::new_err(format!("{e:?}")))?,
            DeadNodeViewType::Local(n) => n
                .clone()
                .remove_stale_resources()
                .map_err(|e| NodeCleanupFailure::new_err(format!("{e:?}")))?,
        };

        Ok(result)
    }
}

#[pyclass]
#[derive(Clone)]
/// Describes the state of a `Node`.
pub enum NodeState {
    /// The `Node`s process is still alive.
    Alive(AliveNodeView),
    /// The `Node`s process died without cleaning up the `Node`s resources. Another process has
    /// now the responsibility to cleanup all the stale resources.
    Dead(DeadNodeView),
    /// The process does not have sufficient permissions to identify the `Node` as dead or alive.
    Inaccessible(NodeId),
    /// The `Node` is in an undefined state, meaning that certain elements are missing,
    /// misconfigured or inconsistent. This can only happen due to an implementation failure or
    /// when the corresponding `Node` resources were altered.
    Undefined(NodeId),
}
