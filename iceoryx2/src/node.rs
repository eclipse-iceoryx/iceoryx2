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

use std::marker::PhantomData;

use crate::node_name::NodeName;
use crate::service;
use crate::service::config_scheme::{node_details_path, node_monitoring_config};
use crate::{config::Config, service::config_scheme::node_details_config};
use iceoryx2_bb_log::{fail, fatal_panic, warn};
use iceoryx2_bb_posix::directory::{Directory, DirectoryRemoveError};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::named_concept::NamedConceptRemoveError;
use iceoryx2_cal::{
    monitoring::*, named_concept::NamedConceptListError, serialize::*, static_storage::*,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeCreationFailure {
    InsufficientPermissions,
    InternalError,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeListFailure {
    InsufficientPermissions,
    Interrupt,
    InternalError,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeCleanupFailure {
    Interrupt,
    InternalError,
    InsufficientPermissions,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum NodeReadStorageFailure {
    ReadError,
    Corrupted,
    InternalError,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeDetails {
    name: NodeName,
    config: Config,
}

#[derive(Debug, Clone)]
pub enum NodeState<Service: service::Service> {
    Alive(AliveNodeView<Service>),
    Dead(DeadNodeView<Service>),
}

#[derive(Debug, Clone)]
pub struct AliveNodeView<Service: service::Service> {
    id: UniqueSystemId,
    details: Option<NodeDetails>,
    _service: PhantomData<Service>,
}

impl<Service: service::Service> AliveNodeView<Service> {
    pub fn id(&self) -> &UniqueSystemId {
        &self.id
    }

    pub fn details(&self) -> &Option<NodeDetails> {
        &self.details
    }
}

#[derive(Debug, Clone)]
pub struct DeadNodeView<Service: service::Service>(AliveNodeView<Service>);

impl<Service: service::Service> DeadNodeView<Service> {
    pub fn id(&self) -> &UniqueSystemId {
        &self.0.id()
    }

    pub fn details(&self) -> &Option<NodeDetails> {
        &self.0.details()
    }

    pub fn remove_stale_resources(&self) -> Result<bool, NodeCleanupFailure> {
        let msg = "Unable to remove stale resources";
        let monitor_name = fatal_panic!(from self, when FileName::new(self.id().value().to_string().as_bytes()),
                                "This should never happen! {msg} since the UniqueSystemId is not a valid file name.");

        let _cleaner =
            match <Service::Monitoring as Monitoring>::Builder::new(&monitor_name).cleaner() {
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

fn remove_node_details_directory<Service: service::Service>(
    origin: &str,
    details: &NodeDetails,
    monitor_name: &FileName,
) -> Result<(), NodeCleanupFailure> {
    let msg = "Unable to remove node resources";

    match Directory::remove(&node_details_path::<Service>(
        &details.config,
        &monitor_name,
    )) {
        Ok(()) => Ok(()),
        Err(DirectoryRemoveError::InsufficientPermissions) => {
            fail!(from origin, with NodeCleanupFailure::InsufficientPermissions,
                        "{} since the node config directory could not be removed due to insufficient of permissions.", msg);
        }
        Err(e) => {
            fail!(from origin, with NodeCleanupFailure::InternalError,
                        "{} since the node config directory could not be removed due to an internal error ({:?}).",
                        msg, e);
        }
    }
}

fn acquire_all_node_detail_storages<Service: service::Service>(
    origin: &str,
    config: &<Service::StaticStorage as NamedConceptMgmt>::Configuration,
) -> Result<Vec<FileName>, NodeCleanupFailure> {
    let msg = "Unable to list all node detail storages";
    match <Service::StaticStorage as NamedConceptMgmt>::list_cfg(&config) {
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

    remove_node_details_directory::<Service>(&origin, details, &monitor_name)?;
    Ok(true)
}

#[derive(Debug)]
pub struct Node<Service: service::Service> {
    id: UniqueSystemId,
    details: NodeDetails,
    _monitor: <Service::Monitoring as Monitoring>::Token,
    _details_storage: Service::StaticStorage,
}

impl<Service: service::Service> Drop for Node<Service> {
    fn drop(&mut self) {
        warn!(from self, when remove_node::<Service>(self.id, &self.details),
            "Unable to remove node resources.");
    }
}

impl<Service: service::Service> Node<Service> {
    pub fn name(&self) -> &NodeName {
        &self.details.name
    }

    pub fn config(&self) -> &Config {
        &self.details.config
    }

    pub fn id(&self) -> &UniqueSystemId {
        &self.id
    }

    pub fn list() -> Result<Vec<NodeState<Service>>, NodeListFailure> {
        Self::list_with_custom_config(Config::get_global_config())
    }

    pub fn list_with_custom_config(
        config: &Config,
    ) -> Result<Vec<NodeState<Service>>, NodeListFailure> {
        let monitoring_config = node_monitoring_config::<Service>(&config);
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

    fn list_all_nodes(
        config: &<Service::Monitoring as NamedConceptMgmt>::Configuration,
    ) -> Result<Vec<FileName>, NodeListFailure> {
        let result = <Service::Monitoring as NamedConceptMgmt>::list_cfg(config);

        if result.is_ok() {
            return Ok(result.unwrap());
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

        if result.is_ok() {
            return Ok(result.unwrap());
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
        let result = <Service::Monitoring as Monitoring>::Builder::new(name).monitor();

        if result.is_ok() {
            return Ok(Self::state_from_monitor(&result.unwrap())?);
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
        let details_config = node_details_config::<Service>(config, &node_name);
        let result = <Service::StaticStorage as StaticStorage>::Builder::new(
            &FileName::new(b"node").unwrap(),
        )
        .config(&details_config)
        .has_ownership(false)
        .open();

        if result.is_ok() {
            return Ok(Some(result.unwrap()));
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
        let node_storage = Self::open_node_storage(config, node_name)?;
        if node_storage.is_none() {
            return Ok(None);
        }
        let node_storage = node_storage.unwrap();

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

#[derive(Debug)]
pub struct NodeBuilder {
    name: Option<NodeName>,
    config: Option<Config>,
}

impl NodeBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            config: None,
        }
    }

    pub fn name(mut self, value: NodeName) -> Self {
        self.name = Some(value);
        self
    }

    pub fn config(mut self, value: Config) -> Self {
        self.config = Some(value);
        self
    }

    fn create_token<Service: service::Service>(
        &self,
        config: &Config,
        monitor_name: &FileName,
    ) -> Result<<Service::Monitoring as Monitoring>::Token, NodeCreationFailure> {
        let msg = "Unable to create token for new node";
        let token_result = <Service::Monitoring as Monitoring>::Builder::new(&monitor_name)
            .config(&node_monitoring_config::<Service>(&config))
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

        let details_config = node_details_config::<Service>(&details.config, &monitor_name);
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
            id: node_id,
            _monitor: token,
            _details_storage: details_storage,
            details,
        })
    }
}
