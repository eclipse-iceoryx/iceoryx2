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

use crate::config::Config;
use crate::node_name::NodeName;
use crate::service;
use iceoryx2_bb_elementary::math::ToB64;
use iceoryx2_bb_log::{fail, fatal_panic};
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_system_types::path::Path;
use iceoryx2_cal::monitoring::*;
use iceoryx2_cal::named_concept::NamedConceptConfiguration;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub enum NodeCreationFailure {
    InternalError,
    InsufficientPermissions,
}

#[derive(Serialize, Deserialize)]
struct NodeDetails {
    id: UniqueSystemId,
    name: NodeName,
    config: Config,
}

pub struct Node<Service: service::Service> {
    details: NodeDetails,
    _monitor: <Service::Monitoring as Monitoring>::Token,
    _service: PhantomData<Service>,
}

impl<Service: service::Service> Node<Service> {
    pub fn name(&self) -> &NodeName {
        &self.details.name
    }

    pub fn config(&self) -> &Config {
        &self.details.config
    }

    pub fn id(&self) -> &UniqueSystemId {
        &self.details.id
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

    pub fn create<Service: service::Service>(self) -> Result<Node<Service>, NodeCreationFailure> {
        let msg = "Unable to create node";
        let node_id = fail!(from self, when UniqueSystemId::new(),
                                with NodeCreationFailure::InternalError,
                                "{msg} since the unique node id could not be generated.");
        let monitor_name = node_id.value().to_b64();
        let monitor_name = fatal_panic!(from self, when FileName::new(monitor_name.as_bytes()),
                                "This should never happen! {msg} since the UniqueSystemId is not a valid file name.");
        let config = if let Some(ref config) = self.config {
            config.clone()
        } else {
            Config::get_global_config().clone()
        };
        let monitor_config = <Service::Monitoring as NamedConceptMgmt>::Configuration::default()
            .prefix(
                FileName::new(config.global.prefix.as_bytes())
                    .expect("Global config contains valid global.prefix."),
            )
            .suffix(
                FileName::new(config.global.node.monitor_suffix.as_bytes())
                    .expect("Global config contains valid ."),
            )
            .path_hint(config.global.get_absolute_node_dir());

        let token_result = <Service::Monitoring as Monitoring>::Builder::new(&monitor_name).token();

        let token = match token_result {
            Ok(token) => token,
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
        };

        Ok(Node {
            _service: PhantomData,
            _monitor: token,
            details: NodeDetails {
                id: node_id,
                name: if let Some(name) = self.name {
                    name
                } else {
                    NodeName::new("").expect("An empty NodeName is always valid.")
                },
                config,
            },
        })
    }
}
