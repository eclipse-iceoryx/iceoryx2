// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use core::{fmt::Debug, hash::Hash};

use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_log::fail;
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    node::node_name::NodeName,
    port::port_name::PortName,
    service::{self, service_name::ServiceName},
};

// TODO: better name
pub mod blub;
pub mod recommended;

// TODO: documentation, tests

// contract: unique_value member is unique within on process
// system-wide unique when UniqueIdGenerator implementation guarantees this, e.g. iceoryx2_bb_posix::UniqueSystemId
// that increments a static atomic counter and the pid
#[repr(C)]
#[derive(
    Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, ZeroCopySend,
)]
pub struct UniqueId {
    payload_value: u64,
    unique_value: u64,
}

impl UniqueId {
    // safety: see contract
    pub unsafe fn from_value(value: u128) -> Self {
        Self {
            payload_value: (value >> 64) as u64,
            unique_value: value as u64,
        }
    }

    pub fn value(&self) -> u128 {
        (self.payload_value as u128) << 64 | (self.unique_value as u128)
    }

    pub fn payload_value(&self) -> u64 {
        self.payload_value
    }

    pub fn unique_value(&self) -> u64 {
        self.unique_value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniqueIdGeneratorError {
    GenerationError,
    NotImplemented,
}

impl core::fmt::Display for UniqueIdGeneratorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UniqueIdGeneratorError::{self:?}")
    }
}

impl core::error::Error for UniqueIdGeneratorError {}

pub struct UniqueIdBuilder {
    entity: Entity,
    config: Option<Config>,
}

impl UniqueIdBuilder {
    pub fn new(entity: Entity) -> Self {
        Self {
            entity: entity,
            config: None,
        }
    }

    pub fn config(mut self, value: &Config) -> Self {
        self.config = Some(value.clone());
        self
    }

    pub fn create<Service: service::Service>(mut self) -> Result<UniqueId, UniqueIdGeneratorError> {
        Service::UniqueSystemId::generate::<Service>(self)
    }
}

pub trait UniqueIdGenerator: From<UniqueId> {
    fn generate<Service: service::Service>(
        builder: UniqueIdBuilder,
    ) -> Result<UniqueId, UniqueIdGeneratorError>;

    fn pid(&self) -> Result<iceoryx2_bb_posix::process::ProcessId, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::pid()", with UniqueIdGeneratorError::NotImplemented,
            "pid() is not implemented");
    }

    fn creation_time(&self) -> Result<iceoryx2_bb_posix::clock::Time, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::creation_time()",
            with UniqueIdGeneratorError::NotImplemented, "creation_time() not implemented");
    }
}

pub enum Entity {
    Node(NodeName),
    PubSubService(ServiceName),
    ReqResService(ServiceName),
    EventService(ServiceName),
    BlackboardService(ServiceName),
    Publisher(PortName),
    Subscriber(PortName),
    Client(PortName),
    Server(PortName),
    Notifier(PortName),
    Listener(PortName),
    Writer(PortName),
    Reader(PortName),
}
