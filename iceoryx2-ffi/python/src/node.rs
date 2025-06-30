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
    cleanup_state::CleanupState,
    config::Config,
    duration::Duration,
    error::{NodeListFailure, NodeWaitFailure},
    node_id::NodeId,
    node_name::NodeName,
    node_state::{AliveNodeView, AliveNodeViewType, DeadNodeView, DeadNodeViewType, NodeState},
    parc::Parc,
    service_builder::{ServiceBuilder, ServiceBuilderType},
    service_name::ServiceName,
    service_type::ServiceType,
    signal_handling_mode::SignalHandlingMode,
};

pub(crate) enum NodeType {
    Ipc(iceoryx2::node::Node<crate::IpcService>),
    Local(iceoryx2::node::Node<crate::LocalService>),
}

#[pyclass]
/// The central entry point of iceoryx2. Represents a node of the iceoryx2
/// system. One process can have arbitrary many nodes but usually it should be
/// only one node per process.
/// Can be created via the `NodeBuilder`.
pub struct Node(pub(crate) Parc<NodeType>);

#[pymethods]
impl Node {
    pub fn __str__(&self) -> String {
        match &*self.0.lock() {
            NodeType::Ipc(node) => format!("{node:?}"),
            NodeType::Local(node) => format!("{node:?}"),
        }
    }

    #[getter]
    /// Returns the name of the node inside a `NodeName`.
    pub fn name(&self) -> NodeName {
        match &*self.0.lock() {
            NodeType::Ipc(node) => NodeName(node.name().clone()),
            NodeType::Local(node) => NodeName(node.name().clone()),
        }
    }

    #[getter]
    /// Returns the `Config` that the `Node` will use to create any iceoryx2 entity.
    pub fn config(&self) -> Config {
        match &*self.0.lock() {
            NodeType::Ipc(node) => Config(Parc::new(node.config().clone())),
            NodeType::Local(node) => Config(Parc::new(node.config().clone())),
        }
    }

    #[getter]
    /// Returns the unique id of the `Node`.
    pub fn id(&self) -> NodeId {
        match &*self.0.lock() {
            NodeType::Ipc(node) => NodeId(*node.id()),
            NodeType::Local(node) => NodeId(*node.id()),
        }
    }

    #[staticmethod]
    /// Returns a list of `NodeState`s of all `Node`s under a provided config.
    /// On failure it emits a `NodeListFailure`.
    pub fn list(service_type: &ServiceType, config: &Config) -> PyResult<Vec<NodeState>> {
        let mut states = vec![];

        match service_type {
            ServiceType::Ipc => {
                iceoryx2::prelude::Node::<crate::IpcService>::list(&config.0.lock(), |state| {
                    match state {
                        iceoryx2::node::NodeState::Alive(a) => {
                            states.push(NodeState::Alive(AliveNodeView(AliveNodeViewType::Ipc(a))))
                        }
                        iceoryx2::node::NodeState::Dead(a) => {
                            states.push(NodeState::Dead(DeadNodeView(DeadNodeViewType::Ipc(a))))
                        }
                        iceoryx2::node::NodeState::Inaccessible(a) => {
                            states.push(NodeState::Inaccessible(NodeId(a)))
                        }
                        iceoryx2::node::NodeState::Undefined(a) => {
                            states.push(NodeState::Undefined(NodeId(a)))
                        }
                    }

                    iceoryx2::prelude::CallbackProgression::Continue
                })
                .map_err(|e| NodeListFailure::new_err(format!("{e:?}")))?
            }
            ServiceType::Local => {
                iceoryx2::prelude::Node::<crate::LocalService>::list(&config.0.lock(), |state| {
                    match state {
                        iceoryx2::node::NodeState::Alive(a) => states
                            .push(NodeState::Alive(AliveNodeView(AliveNodeViewType::Local(a)))),
                        iceoryx2::node::NodeState::Dead(a) => {
                            states.push(NodeState::Dead(DeadNodeView(DeadNodeViewType::Local(a))))
                        }
                        iceoryx2::node::NodeState::Inaccessible(a) => {
                            states.push(NodeState::Inaccessible(NodeId(a)))
                        }
                        iceoryx2::node::NodeState::Undefined(a) => {
                            states.push(NodeState::Undefined(NodeId(a)))
                        }
                    }

                    iceoryx2::prelude::CallbackProgression::Continue
                })
                .map_err(|e| NodeListFailure::new_err(format!("{e:?}")))?
            }
        };

        Ok(states)
    }

    /// Instantiates a `ServiceBuilder` for a service with the provided name.
    pub fn service_builder(&self, name: &ServiceName) -> ServiceBuilder {
        match &*self.0.lock() {
            NodeType::Ipc(node) => {
                ServiceBuilder(ServiceBuilderType::Ipc(node.service_builder(&name.0)))
            }
            NodeType::Local(node) => {
                ServiceBuilder(ServiceBuilderType::Local(node.service_builder(&name.0)))
            }
        }
    }

    /// Waits for a given `cycle_time`.
    /// On failure it emits a `NodeWaitFailure`.
    pub fn wait(&self, cycle_time: &Duration) -> PyResult<()> {
        match &*self.0.lock() {
            NodeType::Ipc(node) => node
                .wait(cycle_time.0)
                .map_err(|e| NodeWaitFailure::new_err(format!("{e:?}")))?,
            NodeType::Local(node) => node
                .wait(cycle_time.0)
                .map_err(|e| NodeWaitFailure::new_err(format!("{e:?}")))?,
        };

        Ok(())
    }

    #[getter]
    /// Returns the `SignalHandlingMode` with which the `Node` was created.
    pub fn signal_handling_mode(&self) -> SignalHandlingMode {
        match &*self.0.lock() {
            NodeType::Ipc(node) => node.signal_handling_mode().into(),
            NodeType::Local(node) => node.signal_handling_mode().into(),
        }
    }

    #[staticmethod]
    /// Removes the stale system resources of all dead `Node`s. The dead `Node`s are also
    /// removed from all registered `Service`s.
    ///
    /// If a `Node` cannot be cleaned up since the process has insufficient permissions then
    /// the `Node` is skipped.
    pub fn cleanup_dead_nodes(service_type: &ServiceType, config: &Config) -> CleanupState {
        match service_type {
            ServiceType::Ipc => CleanupState(
                iceoryx2::prelude::Node::<crate::IpcService>::cleanup_dead_nodes(&config.0.lock()),
            ),
            ServiceType::Local => CleanupState(
                iceoryx2::prelude::Node::<crate::LocalService>::cleanup_dead_nodes(
                    &config.0.lock(),
                ),
            ),
        }
    }
}
