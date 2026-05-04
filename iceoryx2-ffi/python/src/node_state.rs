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
    config::Config, duration::Duration, error::NodeCleanupFailure, file_name::FileName,
    node_name::NodeName, parc::Parc, unique_node_id::UniqueNodeId,
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

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
/// Contains details of a `Node`.
pub struct NodeDetails(pub(crate) iceoryx2::node::NodeDetails);

#[pymethods]
impl NodeDetails {
    #[getter]
    /// Returns the executable `FileName` of the `Node`s owner process.
    pub fn executable(&self) -> FileName {
        FileName(*self.0.executable())
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

#[pyclass(from_py_object)]
#[derive(Clone)]
/// Contains all details of a `Node` that is alive.
pub struct AliveNodeView(pub(crate) AliveNodeViewType);

#[pymethods]
impl AliveNodeView {
    #[getter]
    /// Returns the `UniqueNodeId`.
    pub fn id(&self) -> UniqueNodeId {
        match &self.0 {
            AliveNodeViewType::Ipc(n) => UniqueNodeId(*n.id()),
            AliveNodeViewType::Local(n) => UniqueNodeId(*n.id()),
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

#[pyclass(from_py_object)]
#[derive(Clone)]
/// Contains all details of a `Node` that is dead.
pub struct DeadNodeView(pub(crate) DeadNodeViewType);

#[pymethods]
impl DeadNodeView {
    #[getter]
    /// Returns the `UniqueNodeId`.
    pub fn id(&self) -> UniqueNodeId {
        match &self.0 {
            DeadNodeViewType::Ipc(n) => UniqueNodeId(*n.id()),
            DeadNodeViewType::Local(n) => UniqueNodeId(*n.id()),
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

    /// Removes all stale resources of a dead `Node`. If another instance
    /// is already removing the dead `Node` it waits until the other instance
    /// has cleaned up the dead `Node` completely. If the other cleanup instance
    /// crashes, it will take over the ownership and continue the cleanup.
    /// If the process does not have the permission to cleanup all resources it
    /// aborts with an error.
    ///
    /// If the provided timeout is expired it will return.
    pub fn blocking_remove_stale_resources(
        &self,
        timeout: &Duration,
        py: Python<'_>,
    ) -> PyResult<()> {
        py.detach(move || {
            match &self.0 {
                DeadNodeViewType::Ipc(n) => n
                    .clone()
                    .blocking_remove_stale_resources(timeout.0)
                    .map_err(|e| NodeCleanupFailure::new_err(format!("{e:?}")))?,
                DeadNodeViewType::Local(n) => n
                    .clone()
                    .blocking_remove_stale_resources(timeout.0)
                    .map_err(|e| NodeCleanupFailure::new_err(format!("{e:?}")))?,
            };

            Ok(())
        })
    }

    /// Removes all stale resources of the dead `Node`. On error it emits a `NodeCleanupFailure`.
    /// It returns true if the stale resources could be removed, otherwise false.
    pub fn try_remove_stale_resources(&self) -> PyResult<()> {
        match &self.0 {
            DeadNodeViewType::Ipc(n) => n
                .clone()
                .try_remove_stale_resources()
                .map_err(|e| NodeCleanupFailure::new_err(format!("{e:?}")))?,
            DeadNodeViewType::Local(n) => n
                .clone()
                .try_remove_stale_resources()
                .map_err(|e| NodeCleanupFailure::new_err(format!("{e:?}")))?,
        };

        Ok(())
    }
}

#[pyclass(from_py_object)]
#[derive(Clone)]
/// Describes the state of a `Node`.
pub enum NodeState {
    /// The `Node`s process is still alive.
    Alive(AliveNodeView),
    /// The `Node`s process died without cleaning up the `Node`s resources. Another process has
    /// now the responsibility to cleanup all the stale resources.
    Dead(DeadNodeView),
    /// The process does not have sufficient permissions to identify the `Node` as dead or alive.
    Inaccessible(UniqueNodeId),
    /// The `Node` is in an undefined state, meaning that certain elements are missing,
    /// misconfigured or inconsistent. This can only happen due to an implementation failure or
    /// when the corresponding `Node` resources were altered.
    Undefined(UniqueNodeId),
}
