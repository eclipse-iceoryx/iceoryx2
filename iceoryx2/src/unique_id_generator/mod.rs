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

// TODO: example?

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

#[doc(hidden)]
pub mod recommended;
pub mod unique_system_id;

/// 128-Byte ID that provides a `payload_value` and a `unique_value`. The latter is unique, at least
/// within a single process. Further guarantees, such as a system-wide uniqueness, depend on the
/// [`UniqueIdGenerator`] concept implementation. The `payload_value` can be used to provide additional
/// information with the ID, as for instance the time when the unique ID was created.
#[repr(C)]
#[derive(
    Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize, ZeroCopySend,
)]
pub struct UniqueId {
    payload_value: u64,
    unique_value: u64,
}

impl UniqueId {
    /// Creates a new unique ID from a given raw ID.
    ///
    /// # Safety
    ///
    /// The user must ensure that the raw ID is valid, i.e. [`UniqueId::unique_value()`] must
    /// return a unique value for the created ID.
    pub unsafe fn from_raw_id(value: u128) -> Self {
        Self {
            payload_value: (value >> 64) as u64,
            unique_value: value as u64,
        }
    }

    /// Returns the underlying raw value of the ID.
    pub fn value(&self) -> u128 {
        (self.payload_value as u128) << 64 | (self.unique_value as u128)
    }

    /// Returns the payload part of the ID.
    pub fn payload_value(&self) -> u64 {
        self.payload_value
    }

    /// Returns the unique part of the ID.
    pub fn unique_value(&self) -> u64 {
        self.unique_value
    }
}

/// Describes failures related to the [`UniqueIdGenerator`] trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniqueIdGeneratorError {
    /// The unique ID could not be generated.
    GenerationError,
    /// The trait implementation does not provide the function.
    NotImplemented,
}

impl core::fmt::Display for UniqueIdGeneratorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UniqueIdGeneratorError::{self:?}")
    }
}

impl core::error::Error for UniqueIdGeneratorError {}

/// Generates [`UniqueId`]s whose [`UniqueId::unique_value()`]s are unique, at least within a single process.
pub trait UniqueIdGenerator: From<UniqueId> {
    /// Generates a [`UniqueId`] for a specific [`service::Service].
    fn generate<Service: service::Service>(
        entity: Entity,
        config: &Config,
    ) -> Result<UniqueId, UniqueIdGeneratorError>;

    /// Returns the [`ProcessId`](iceoryx2_bb_posix::process::ProcessId) that was used to create the [`UniqueId`].
    fn pid(&self) -> Result<iceoryx2_bb_posix::process::ProcessId, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::pid()", with UniqueIdGeneratorError::NotImplemented,
            "pid() is not implemented");
    }

    /// Returns the [`Time`](iceoryx2_bb_posix::clock::Time) when the [`UniqueId`] was created.
    fn creation_time(&self) -> Result<iceoryx2_bb_posix::clock::Time, UniqueIdGeneratorError> {
        fail!(from "UniqueIdGenerator::creation_time()",
            with UniqueIdGeneratorError::NotImplemented, "creation_time() not implemented");
    }
}

/// Identifies the kind of entity for which a unique ID can be generated.
pub enum Entity {
    /// Identifies a [`Node`](crate::node::Node)
    Node(NodeName),
    /// Identifies a publish-subscribe service.
    PubSubService(ServiceName),
    /// Identifies a request-response service.
    ReqResService(ServiceName),
    /// Identifies an event service.
    EventService(ServiceName),
    /// Identifies a blackboard service.
    BlackboardService(ServiceName),
    /// Identifies a [`Publisher`](crate::port::publisher::Publisher)
    Publisher(PortName),
    /// Identifies a [`Subscriber`](crate::port::subscriber::Subscriber)
    Subscriber(PortName),
    /// Identifies a [`Client`](crate::port::client::Client)
    Client(PortName),
    /// Identifies a [`Server`](crate::port::server::Server)
    Server(PortName),
    /// Identifies a [`Notifier`](crate::port::notifier::Notifier)
    Notifier(PortName),
    /// Identifies a [`Listener`](crate::port::listener::Listener)
    Listener(PortName),
    /// Identifies a [`Writer`](crate::port::writer::Writer)
    Writer(PortName),
    /// Identifies a [`Reader`](crate::port::reader::Reader)
    Reader(PortName),
}
