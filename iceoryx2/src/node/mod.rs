// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

//! The [`Node`](crate::node::Node) is the central entry point of iceoryx2. It is the owner of all communication
//! entities and provides additional memory to them to perform reference counting amongst other
//! things.
//!
//! It allows also the system to monitor the state of processes and cleanup stale resources of
//! dead processes.
//!
//! # Create a [`Node`](crate::node::Node)
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node = NodeBuilder::new()
//!                 .name(&NodeName::new("my_little_node")?)
//!                 .create::<zero_copy::Service>()?;
//!
//! println!("created node {:?}", node);
//! # Ok(())
//! # }
//! ```
//!
//! # List all existing [`Node`](crate::node::Node)s
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node_state_list = Node::<zero_copy::Service>::list(Config::get_global_config())?;
//!
//! for node_state in node_state_list {
//!     println!("found node {:?}", node_state);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Cleanup stale resources of all dead [`Node`](crate::node::Node)s
//!
//! ```
//! use iceoryx2::prelude::*;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let node_state_list = Node::<zero_copy::Service>::list(Config::get_global_config())?;
//!
//! for node_state in node_state_list {
//!     if let NodeState::<zero_copy::Service>::Dead(view) = node_state {
//!         println!("cleanup resources of dead node {:?}", view);
//!         view.remove_stale_resources()?;
//!     }
//! }
//! # Ok(())
//! # }
//! ```

/// The name for a node.
pub mod node_name;

#[doc(hidden)]
pub mod testing;

use crate::node::node_name::NodeName;
use crate::service;
use crate::service::builder::Builder;
use crate::service::config_scheme::{node_details_path, node_monitoring_config};
use crate::service::service_name::ServiceName;
use crate::{config::Config, service::config_scheme::node_details_config};
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::named_concept::{NamedConceptPathHintRemoveError, NamedConceptRemoveError};
use iceoryx2_cal::{
    monitoring::*, named_concept::NamedConceptListError, serialize::*, static_storage::*,
};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::sync::Arc;

/// The failures that can occur when a [`Node`] is created with the [`NodeBuilder`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeCreationFailure {
    InsufficientPermissions,
    InternalError,
}

impl std::fmt::Display for NodeCreationFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "NodeCreationFailure::{:?}", self)
    }
}

impl std::error::Error for NodeCreationFailure {}

/// The failures that can occur when a list of [`NodeState`]s is created with [`Node::list()`].
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeListFailure {
    InsufficientPermissions,
    Interrupt,
    InternalError,
}

impl std::fmt::Display for NodeListFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "NodeListFailure::{:?}", self)
    }
}

impl std::error::Error for NodeListFailure {}

/// Failures of [`DeadNodeView::remove_stale_resources()`] that occur when the stale resources of
/// a dead [`Node`] are removed.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeCleanupFailure {
    Interrupt,
    InternalError,
    InsufficientPermissions,
}

impl std::fmt::Display for NodeCleanupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::write!(f, "NodeCleanupFailure::{:?}", self)
    }
}

impl std::error::Error for NodeCleanupFailure {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NodeReadStorageFailure {
    ReadError,
    Corrupted,
    InternalError,
}

/// Optional detailed informations that a [`Node`] can have. They can only be obtained when the
/// process has sufficient access permissions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeDetails {
    name: NodeName,
    config: Config,
}

impl NodeDetails {
    /// Returns the [`NodeName`]. Multiple [`Node`]s are allowed to have the same [`NodeName`], it
    /// is not unique!
    pub fn name(&self) -> &NodeName {
        &self.name
    }

    /// Returns the [`Config`] the [`Node`] uses to create all entities.
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// The current state of the [`Node`]. If the [`Node`] is dead all of its resources can be removed
/// with [`DeadNodeView::remove_stale_resources()`].
#[derive(Debug, Clone)]
pub enum NodeState<Service: service::Service> {
    Alive(AliveNodeView<Service>),
    Dead(DeadNodeView<Service>),
}

/// Contains all available details of a [`Node`].
pub trait NodeView {
    /// Returns the [`UniqueSystemId`] of the [`Node`].
    fn id(&self) -> &UniqueSystemId;
    /// Returns the [`NodeDetails`].
    fn details(&self) -> &Option<NodeDetails>;
}

/// All the informations of a [`Node`] that is alive.
#[derive(Debug, Clone)]
pub struct AliveNodeView<Service: service::Service> {
    id: UniqueSystemId,
    details: Option<NodeDetails>,
    _service: PhantomData<Service>,
}

impl<Service: service::Service> NodeView for AliveNodeView<Service> {
    fn id(&self) -> &UniqueSystemId {
        &self.id
    }

    fn details(&self) -> &Option<NodeDetails> {
        &self.details
    }
}

/// All the informations and management operations belonging to a dead [`Node`].
#[derive(Debug, Clone)]
pub struct DeadNodeView<Service: service::Service>(AliveNodeView<Service>);

impl<Service: service::Service> NodeView for DeadNodeView<Service> {
    fn id(&self) -> &UniqueSystemId {
        self.0.id()
    }

    fn details(&self) -> &Option<NodeDetails> {
        self.0.details()
    }
}

impl<Service: service::Service> DeadNodeView<Service> {
    /// Removes all stale resources of a dead [`Node`].
    pub fn remove_stale_resources(self) -> Result<bool, NodeCleanupFailure> {
        let msg = "Unable to remove stale resources";
        let monitor_name = fatal_panic!(from self, when FileName::new(self.id().value().to_string().as_bytes()),
                                "This should never happen! {msg} since the UniqueSystemId is not a valid file name.");

        let config = if let Some(d) = self.details() {
            d.config()
        } else {
            Config::get_global_config()
        };

        let _cleaner = match <Service::Monitoring as Monitoring>::Builder::new(&monitor_name)
            .config(&node_monitoring_config::<Service>(config))
            .cleaner()
        {
            Ok(cleaner) => cleaner,
            Err(MonitoringCreateCleanerError::AlreadyOwnedByAnotherInstance)
            | Err(MonitoringCreateCleanerError::DoesNotExist) => return Ok(false),
            Err(MonitoringCreateCleanerError::Interrupt) => {
                fail!(from self, with NodeCleanupFailure::Interrupt,
                    "{} since an interrupt signal was received.", msg);
            }
            Err(MonitoringCreateCleanerError::InternalError) => {
                fail!(from self, with NodeCleanupFailure::InternalError,
                    "{} due to an internal error while acquiring monitoring cleaner.", msg);
            }
            Err(MonitoringCreateCleanerError::InstanceStillAlive) => {
                fatal_panic!(from self,
                        "This should never happen! {} since the Node is still alive.", msg);
            }
        };

        if let Some(details) = self.details() {
            remove_node::<Service>(*self.id(), details)
        } else {
            Ok(true)
        }
    }
}

fn acquire_all_node_detail_storages<Service: service::Service>(
    origin: &str,
    config: &<Service::StaticStorage as NamedConceptMgmt>::Configuration,
) -> Result<Vec<FileName>, NodeCleanupFailure> {
    let msg = "Unable to list all node detail storages";
    match <Service::StaticStorage as NamedConceptMgmt>::list_cfg(config) {
        Ok(v) => Ok(v),
        Err(NamedConceptListError::InsufficientPermissions) => {
            fail!(from origin, with NodeCleanupFailure::InsufficientPermissions,
                "{} due to insufficient permissions.", msg);
        }
        Err(NamedConceptListError::InternalError) => {
            fail!(from origin, with NodeCleanupFailure::InternalError,
                "{} due to an internal error.", msg);
        }
    }
}

fn remove_detail_storages<Service: service::Service>(
    origin: &str,
    storages: Vec<FileName>,
    config: &<Service::StaticStorage as NamedConceptMgmt>::Configuration,
) -> Result<(), NodeCleanupFailure> {
    let msg = "Unable to remove node detail storage";
    for entry in storages {
        match unsafe { <Service::StaticStorage as NamedConceptMgmt>::remove_cfg(&entry, config) } {
            Ok(_) => (),
            Err(NamedConceptRemoveError::InsufficientPermissions) => {
                fail!(from origin, with NodeCleanupFailure::InsufficientPermissions,
                    "{} {} due to insufficient permissions.", msg, entry);
            }
            Err(NamedConceptRemoveError::InternalError) => {
                fail!(from origin, with NodeCleanupFailure::InsufficientPermissions,
                    "{} {} due to an internal failure.", msg, entry);
            }
        }
    }

    Ok(())
}

fn remove_node_details_directory<Service: service::Service>(
    config: &Config,
    monitor_name: &FileName,
) -> Result<(), NodeCleanupFailure> {
    let origin = format!(
        "remove_node_details_directory({:?}, {:?})",
        config, monitor_name
    );
    let msg = "Unable to remove node details directory";
    let path = node_details_path(config, monitor_name);
    match <Service::StaticStorage as NamedConceptMgmt>::remove_path_hint(&path) {
        Ok(()) => Ok(()),
        Err(NamedConceptPathHintRemoveError::InsufficientPermissions) => {
            fail!(from origin, with NodeCleanupFailure::InsufficientPermissions,
                "{} due to insufficient permissions.", msg);
        }
        Err(NamedConceptPathHintRemoveError::InternalError) => {
            fail!(from origin, with NodeCleanupFailure::InternalError,
                "{} due to an internal error.", msg);
        }
    }
}

fn remove_node<Service: service::Service>(
    id: UniqueSystemId,
    details: &NodeDetails,
) -> Result<bool, NodeCleanupFailure> {
    let origin = format!(
        "remove_node<{}>({:?})",
        core::any::type_name::<Service>(),
        id
    );
    let msg = "Unable to remove node resources";
    let monitor_name = fatal_panic!(from origin, when FileName::new(id.value().to_string().as_bytes()),
                                "This should never happen! {msg} since the UniqueSystemId is not a valid file name.");

    let details_config = node_details_config::<Service>(&details.config, &monitor_name);
    let detail_storages = acquire_all_node_detail_storages::<Service>(&origin, &details_config)?;
    remove_detail_storages::<Service>(&origin, detail_storages, &details_config)?;
    remove_node_details_directory::<Service>(details.config(), &monitor_name)?;

    Ok(true)
}

#[derive(Debug)]
pub(crate) struct SharedNode<Service: service::Service> {
    id: UniqueSystemId,
    details: NodeDetails,
    token: UnsafeCell<Option<<Service::Monitoring as Monitoring>::Token>>,
    _details_storage: Service::StaticStorage,
}

impl<Service: service::Service> SharedNode<Service> {
    pub(crate) fn config(&self) -> &Config {
        &self.details.config
    }
}

impl<Service: service::Service> Drop for SharedNode<Service> {
    fn drop(&mut self) {
        if self.token.get_mut().is_some() {
            warn!(from self, when remove_node::<Service>(self.id, &self.details),
                "Unable to remove node resources.");
        }
    }
}

/// The [`Node`] is the entry point to the whole iceoryx2 infrastructure and owns all entities.
/// As soon as a process crashes other processes can detect dead [`Node`]s via [`Node::list()`]
/// and clean up the stale resources - the entities that
/// were created via the [`Node`].
///
/// Can be created via the [`NodeBuilder`].
#[derive(Debug)]
pub struct Node<Service: service::Service> {
    shared: Arc<SharedNode<Service>>,
}

unsafe impl<Service: service::Service> Send for Node<Service> {}
unsafe impl<Service: service::Service> Sync for Node<Service> {}

impl<Service: service::Service> Node<Service> {
    /// Returns the [`NodeName`].
    pub fn name(&self) -> &NodeName {
        &self.shared.details.name
    }

    /// Returns the [`Config`] that the [`Node`] will use to create any iceoryx2 entity.
    pub fn config(&self) -> &Config {
        &self.shared.details.config
    }

    /// Returns the [`UniqueSystemId`] of the [`Node`].
    pub fn id(&self) -> &UniqueSystemId {
        &self.shared.id
    }

    pub fn service(&self, name: &ServiceName) -> Builder<Service> {
        Builder::new(name, self.shared.clone())
    }

    /// Returns a list of [`NodeState`] of all [`Node`]s in the system under a given [`Config`].
    pub fn list(config: &Config) -> Result<Vec<NodeState<Service>>, NodeListFailure> {
        let monitoring_config = node_monitoring_config::<Service>(config);
        let mut nodes = vec![];

        for node_name in &Self::list_all_nodes(&monitoring_config)? {
            let id_value = core::str::from_utf8(node_name.as_bytes()).unwrap();
            let id_value = id_value.parse::<u128>().unwrap();

            let details = match Self::get_node_details(config, node_name) {
                Ok(v) => v,
                Err(_) => None,
            };

            let node_view = AliveNodeView::<Service> {
                id: id_value.into(),
                details,
                _service: PhantomData,
            };

            match Self::get_node_state(&monitoring_config, node_name)? {
                State::DoesNotExist => (),
                State::Alive => nodes.push(NodeState::Alive(node_view)),
                State::Dead => nodes.push(NodeState::Dead(DeadNodeView(node_view))),
            };
        }

        Ok(nodes)
    }

    /// # Safety
    ///
    ///  * only for internal testing purposes
    ///  * shall be called at most once
    ///
    pub(crate) unsafe fn staged_death(&mut self) -> <Service::Monitoring as Monitoring>::Token {
        (*self.shared.token.get()).take().unwrap()
    }

    fn list_all_nodes(
        config: &<Service::Monitoring as NamedConceptMgmt>::Configuration,
    ) -> Result<Vec<FileName>, NodeListFailure> {
        let result = <Service::Monitoring as NamedConceptMgmt>::list_cfg(config);

        if let Ok(result) = result {
            return Ok(result);
        }

        let msg = "Unable to list all nodes";
        let origin = format!("Node::list_all_nodes({:?})", config);
        match result.err().unwrap() {
            NamedConceptListError::InsufficientPermissions => {
                fail!(from origin, with NodeListFailure::InsufficientPermissions,
                        "{} due to insufficient permissions while listing all nodes.", msg);
            }
            NamedConceptListError::InternalError => {
                fail!(from origin, with NodeListFailure::InternalError,
                        "{} due to an internal failure while listing all nodes.", msg);
            }
        }
    }

    fn state_from_monitor(
        monitor: &<Service::Monitoring as Monitoring>::Monitor,
    ) -> Result<State, NodeListFailure> {
        let result = monitor.state();

        if let Ok(result) = result {
            return Ok(result);
        }

        let msg = "Unable to acquire node state from monitor";
        let origin = format!("Node::state_from_monitor({:?})", monitor);

        match result.err().unwrap() {
            MonitoringStateError::Interrupt => {
                fail!(from origin, with NodeListFailure::Interrupt,
                    "{} due to an interrupt signal while acquiring the nodes state.", msg);
            }
            MonitoringStateError::InternalError => {
                fail!(from origin, with NodeListFailure::InternalError,
                    "{} due to an internal error while acquiring the nodes state.", msg);
            }
        }
    }

    fn get_node_state(
        config: &<Service::Monitoring as NamedConceptMgmt>::Configuration,
        name: &FileName,
    ) -> Result<State, NodeListFailure> {
        let result = <Service::Monitoring as Monitoring>::Builder::new(name)
            .config(config)
            .monitor();

        if let Ok(result) = result {
            return Self::state_from_monitor(&result);
        }

        let msg = "Unable to acquire node monitor";
        let origin = format!("Node::get_node_state({:?}, {:?})", config, name);
        match result.err().unwrap() {
            MonitoringCreateMonitorError::InsufficientPermissions => {
                fail!(from origin, with NodeListFailure::InsufficientPermissions,
                        "{} due to insufficient permissions while acquiring the node state.", msg);
            }
            MonitoringCreateMonitorError::Interrupt => {
                fail!(from origin, with NodeListFailure::Interrupt,
                        "{} since an interrupt was received while acquiring the node state.", msg);
            }
            MonitoringCreateMonitorError::InternalError => {
                fail!(from origin, with NodeListFailure::InternalError,
                        "{} since an internal failure occurred while acquiring the node state.", msg);
            }
        }
    }

    fn open_node_storage(
        config: &Config,
        node_name: &FileName,
    ) -> Result<Option<Service::StaticStorage>, NodeReadStorageFailure> {
        let details_config = node_details_config::<Service>(config, node_name);
        let result = <Service::StaticStorage as StaticStorage>::Builder::new(
            &FileName::new(b"node").unwrap(),
        )
        .config(&details_config)
        .has_ownership(false)
        .open();

        if let Ok(result) = result {
            return Ok(Some(result));
        }

        let msg = "Unable to open node config storage";
        let origin = format!("open_node_storage({:?}, {:?})", config, node_name);

        match result.err().unwrap() {
            StaticStorageOpenError::DoesNotExist => Ok(None),
            StaticStorageOpenError::Read => {
                fail!(from origin, with NodeReadStorageFailure::ReadError,
                    "{} since the node config storage could not be read.", msg);
            }
            StaticStorageOpenError::IsLocked => {
                fail!(from origin, with NodeReadStorageFailure::Corrupted,
                    "{} since the node config storage seems to be uninitialized but the state should always be present.", msg);
            }
            StaticStorageOpenError::InternalError => {
                fail!(from origin, with NodeReadStorageFailure::InternalError,
                    "{} due to an internal failure while opening the node config storage.", msg);
            }
        }
    }

    fn get_node_details(
        config: &Config,
        node_name: &FileName,
    ) -> Result<Option<NodeDetails>, NodeReadStorageFailure> {
        let node_storage = if let Some(n) = Self::open_node_storage(config, node_name)? {
            n
        } else {
            return Ok(None);
        };

        let mut read_content =
            String::from_utf8(vec![b' '; node_storage.len() as usize]).expect("");

        let origin = format!("get_node_details({:?}, {:?})", config, node_name);
        let msg = "Unable to read node details";

        if node_storage
            .read(unsafe { read_content.as_mut_vec() }.as_mut_slice())
            .is_err()
        {
            fail!(from origin, with NodeReadStorageFailure::ReadError,
                "{} since the content of the node config storage could not be read.", msg);
        }

        let node_details = fail!(from origin,
                    when Service::ConfigSerializer::deserialize::<NodeDetails>(unsafe { read_content.as_mut_vec()}),
                    with NodeReadStorageFailure::Corrupted,
                "{} since the contents of the node config storage is corrupted.", msg);

        Ok(Some(node_details))
    }
}

/// Creates a [`Node`].
///
/// ```
/// use iceoryx2::prelude::*;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let node = NodeBuilder::new()
///                 .name(&NodeName::new("my_little_node")?)
///                 .create::<zero_copy::Service>()?;
///
/// // do things with your cool new node
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct NodeBuilder {
    name: Option<NodeName>,
    config: Option<Config>,
}

impl NodeBuilder {
    /// Creates a new [`NodeBuilder`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`NodeName`] of the to be created [`Node`].
    pub fn name(mut self, value: &NodeName) -> Self {
        self.name = Some(value.clone());
        self
    }

    /// Sets the config of the [`Node`] that will be used to create all entities owned by the
    /// [`Node`].
    pub fn config(mut self, value: &Config) -> Self {
        self.config = Some(value.clone());
        self
    }

    /// Creates a new [`Node`] for a specific [`service::Service`]. All entities owned by the
    /// [`Node`] will have the same [`service::Service`].
    pub fn create<Service: service::Service>(self) -> Result<Node<Service>, NodeCreationFailure> {
        let msg = "Unable to create node";
        let node_id = fail!(from self, when UniqueSystemId::new(),
                                with NodeCreationFailure::InternalError,
                                "{msg} since the unique node id could not be generated.");
        let monitor_name = fatal_panic!(from self, when FileName::new(node_id.value().to_string().as_bytes()),
                                "This should never happen! {msg} since the UniqueSystemId is not a valid file name.");
        let config = if let Some(ref config) = self.config {
            config.clone()
        } else {
            Config::get_global_config().clone()
        };

        let (details_storage, details) =
            self.create_node_details_storage::<Service>(&config, &monitor_name)?;
        let token = self.create_token::<Service>(&config, &monitor_name)?;

        Ok(Node {
            shared: Arc::new(SharedNode {
                id: node_id,
                token: UnsafeCell::new(Some(token)),
                _details_storage: details_storage,
                details,
            }),
        })
    }

    fn create_token<Service: service::Service>(
        &self,
        config: &Config,
        monitor_name: &FileName,
    ) -> Result<<Service::Monitoring as Monitoring>::Token, NodeCreationFailure> {
        let msg = "Unable to create token for new node";
        let token_result = <Service::Monitoring as Monitoring>::Builder::new(monitor_name)
            .config(&node_monitoring_config::<Service>(config))
            .token();

        match token_result {
            Ok(token) => Ok(token),
            Err(MonitoringCreateTokenError::InsufficientPermissions) => {
                fail!(from self, with NodeCreationFailure::InsufficientPermissions,
                    "{msg} due to insufficient permissions to create a monitor token.");
            }
            Err(MonitoringCreateTokenError::AlreadyExists) => {
                fatal_panic!(from self,
                    "This should never happen! {msg} since a node with the same UniqueNodeId already exists.");
            }
            Err(MonitoringCreateTokenError::InternalError) => {
                fail!(from self, with NodeCreationFailure::InternalError,
                    "{msg} since the monitor token could not be created.");
            }
        }
    }

    fn create_node_details_storage<Service: service::Service>(
        &self,
        config: &Config,
        monitor_name: &FileName,
    ) -> Result<(Service::StaticStorage, NodeDetails), NodeCreationFailure> {
        let msg = "Unable to create node details storage";
        let details = NodeDetails {
            name: if let Some(ref name) = self.name {
                name.clone()
            } else {
                NodeName::new("").expect("An empty NodeName is always valid.")
            },
            config: config.clone(),
        };

        let details_config = node_details_config::<Service>(&details.config, monitor_name);
        let serialized_details = match <Service::ConfigSerializer>::serialize(&details) {
            Ok(serialized_details) => serialized_details,
            Err(SerializeError::InternalError) => {
                fail!(from self, with NodeCreationFailure::InternalError,
                    "{msg} since the node details could not be serialized.");
            }
        };

        match <Service::StaticStorage as StaticStorage>::Builder::new(
            &FileName::new(b"node").unwrap(),
        )
        .config(&details_config)
        .has_ownership(false)
        .create(&serialized_details)
        {
            Ok(node_details) => Ok((node_details, details)),
            Err(StaticStorageCreateError::InsufficientPermissions) => {
                fail!(from self, with NodeCreationFailure::InsufficientPermissions,
                    "{msg} due to insufficient permissions to create the node details file.");
            }
            Err(StaticStorageCreateError::AlreadyExists) => {
                fatal_panic!(from self,
                    "This should never happen! {msg} since the node details file already exists.");
            }
            Err(e) => {
                fail!(from self, with NodeCreationFailure::InternalError,
                    "{msg} due to an unknown failure while creating the node details file {:?}.", e);
            }
        }
    }
}
