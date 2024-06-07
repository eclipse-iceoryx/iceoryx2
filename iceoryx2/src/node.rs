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
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_cal::monitoring::Monitoring;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Serialize, Deserialize)]
struct NodeDetails {
    id: UniqueSystemId,
    name: NodeName,
    config: Config,
}

pub struct Node<Service: service::Service> {
    details: NodeDetails,
    monitor: <Service::Monitoring as Monitoring>::Token,
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

    pub fn create<Service: service::Service>() -> Node<Service> {
        todo!()
    }
}
