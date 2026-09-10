// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use alloc::format;

use crate::port::port_name::PortName;
use crate::service::service_name::ServiceName;
use crate::{config::Config, node::node_name::NodeName, unique_id_generator::*};
use iceoryx2_bb_derive_macros::ZeroCopySend;
use iceoryx2_bb_elementary_traits::zero_copy_send::ZeroCopySend;
use iceoryx2_bb_lock_free::mpmc::robust_unique_index_set::OwnerId;
use iceoryx2_log::fail;

macro_rules! generate_id {
    { $(#[$documentation:meta])*
        $id_name:ident
        $entity:ident
        $name_type:ident} => {
        $(#[$documentation])*
        #[repr(C)]
        #[derive(
            Debug,
            Eq,
            Hash,
            PartialEq,
            Clone,
            Copy,
            PartialOrd,
            Ord,
            ZeroCopySend,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $id_name(pub(crate) UniqueId);

        impl core::fmt::Display for $id_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:x}", self.0.value())
            }
        }

        impl $id_name {
            pub(crate) fn new<Service: crate::service::Service>(
                name: $name_type,
                config: &Config
            ) -> Result<Self, UniqueIdGeneratorGenerateError> {
                let id = fail!(from format!("{}::new()", stringify!($id_name)),
                    when Service::UniqueId::generate::<Service>(Entity::$entity(name), config),
                    with UniqueIdGeneratorGenerateError::GenerationError,
                    ": Unable to generate required {}.", stringify!($id_name));
                Ok(Self(id))
            }

            /// Returns the underlying raw value of the ID
            pub fn value(&self) -> u128 {
                self.0.value()
            }
        }
    };
}

generate_id! {
    /// The system-wide unique id of a [`Publisher`](crate::port::publisher::Publisher).
    UniquePublisherId
    Publisher
    PortName
}
generate_id! {
    /// The system-wide unique id of a [`Subscriber`](crate::port::subscriber::Subscriber).
    UniqueSubscriberId
    Subscriber
    PortName
}
generate_id! {
    /// The system-wide unique id of a [`Notifier`](crate::port::notifier::Notifier).
    UniqueNotifierId
    Notifier
    PortName
}
generate_id! {
    /// The system-wide unique id of a [`Listener`](crate::port::listener::Listener).
    UniqueListenerId
    Listener
    PortName
}
generate_id! {
    /// The system-wide unique id of a [`Client`](crate::port::client::Client).
    UniqueClientId
    Client
    PortName
}
generate_id! {
    /// The system-wide unique id of a [`Server`](crate::port::server::Server).
    UniqueServerId
    Server
    PortName
}
generate_id! {
    /// The system-wide unique id of a [`Reader`](crate::port::reader::Reader).
    UniqueReaderId
    Reader
    PortName
}
generate_id! {
    /// The system-wide unique id of a [`Writer`](crate::port::writer::Writer).
    UniqueWriterId
    Writer
    PortName
}

/// Enum that contains the unique port id
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UniquePortId {
    /// The system-wide unique id of a [`Publisher`](crate::port::publisher::Publisher).
    Publisher(UniquePublisherId),
    /// The system-wide unique id of a [`Subscriber`](crate::port::subscriber::Subscriber).
    Subscriber(UniqueSubscriberId),
    /// The system-wide unique id of a [`Notifier`](crate::port::notifier::Notifier).
    Notifier(UniqueNotifierId),
    /// The system-wide unique id of a [`Listener`](crate::port::listener::Listener).
    Listener(UniqueListenerId),
    /// The system-wide unique id of a [`Client`](crate::port::client::Client).
    Client(UniqueClientId),
    /// The system-wide unique id of a [`Server`](crate::port::server::Server).
    Server(UniqueServerId),
    /// The system-wide unique id of a [`Reader`](crate::port::reader::Reader).
    Reader(UniqueReaderId),
    /// The system-wide unique id of a [`Writer`](crate::port::writer::Writer).
    Writer(UniqueWriterId),
}

impl UniquePortId {
    /// Returns the underlying value of the [`UniquePortId`]
    pub fn value(&self) -> u128 {
        match self {
            UniquePortId::Publisher(v) => v.value(),
            UniquePortId::Subscriber(v) => v.value(),
            UniquePortId::Notifier(v) => v.value(),
            UniquePortId::Listener(v) => v.value(),
            UniquePortId::Client(v) => v.value(),
            UniquePortId::Server(v) => v.value(),
            UniquePortId::Reader(v) => v.value(),
            UniquePortId::Writer(v) => v.value(),
        }
    }
}

/// The system-wide unique id of a [`Service`](crate::service::Service).
#[repr(C)]
#[derive(
    Debug,
    Eq,
    Hash,
    PartialEq,
    Clone,
    Copy,
    PartialOrd,
    Ord,
    ZeroCopySend,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct UniqueServiceId(pub(crate) UniqueId);

impl core::fmt::Display for UniqueServiceId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0.value())
    }
}

impl UniqueServiceId {
    pub(crate) fn from_publish_subscribe_service<Service: crate::service::Service>(
        name: &ServiceName,
        config: &Config,
    ) -> Result<Self, UniqueIdGeneratorGenerateError> {
        let id = fail!(
            from "UniqueServiceId::from_publish_subscribe_service()",
            when Service::UniqueId::generate::<Service>(Entity::PubSubService(*name), config),
            with UniqueIdGeneratorGenerateError::GenerationError,
            ": Unable to generate required UniqueServiceId."
        );
        Ok(Self(id))
    }

    pub(crate) fn from_request_response_service<Service: crate::service::Service>(
        name: &ServiceName,
        config: &Config,
    ) -> Result<Self, UniqueIdGeneratorGenerateError> {
        let id = fail!(
            from "UniqueServiceId::from_request_response_service()",
            when Service::UniqueId::generate::<Service>(Entity::ReqResService(*name), config),
            with UniqueIdGeneratorGenerateError::GenerationError,
            ": Unable to generate required UniqueServiceId."
        );
        Ok(Self(id))
    }

    pub(crate) fn from_event_service<Service: crate::service::Service>(
        name: &ServiceName,
        config: &Config,
    ) -> Result<Self, UniqueIdGeneratorGenerateError> {
        let id = fail!(
            from "UniqueServiceId::from_event_service()",
            when Service::UniqueId::generate::<Service>(Entity::EventService(*name), config),
            with UniqueIdGeneratorGenerateError::GenerationError,
            ": Unable to generate required UniqueServiceId."
        );
        Ok(Self(id))
    }

    pub(crate) fn from_blackboard_service<Service: crate::service::Service>(
        name: &ServiceName,
        config: &Config,
    ) -> Result<Self, UniqueIdGeneratorGenerateError> {
        let id = fail!(
            from "UniqueServiceId::from_blackboard_service()",
            when Service::UniqueId::generate::<Service>(Entity::BlackboardService(*name), config),
            with UniqueIdGeneratorGenerateError::GenerationError,
            ": Unable to generate required UniqueServiceId."
        );
        Ok(Self(id))
    }

    /// Returns the underlying raw value of the ID
    pub fn value(&self) -> u128 {
        self.0.value()
    }
}

/// The system-wide unique id of a [`Node`](crate::node::Node).
#[repr(C)]
#[derive(
    Debug,
    Eq,
    Hash,
    PartialEq,
    Clone,
    Copy,
    PartialOrd,
    Ord,
    ZeroCopySend,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct UniqueNodeId(pub(crate) UniqueId);

impl core::fmt::Display for UniqueNodeId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:x}", self.0.value())
    }
}

impl UniqueNodeId {
    pub(crate) fn new<Service: crate::service::Service>(
        name: NodeName,
        config: &Config,
    ) -> Result<Self, UniqueIdGeneratorGenerateError> {
        let id = fail!(from "UniqueNodeId::new()",
                when Service::UniqueId::generate::<Service>(Entity::Node(name), config),
                with UniqueIdGeneratorGenerateError::GenerationError,
                "Unable to generate required UniqueNodeId.");
        Ok(Self(id))
    }

    /// Returns the underlying raw value of the ID
    pub fn value(&self) -> u128 {
        self.0.value()
    }

    /// Returns the [`ProcessId`](iceoryx2_bb_posix::process::ProcessId) of the process that created the id.
    pub fn pid<Service: crate::service::Service>(
        &self,
    ) -> Result<iceoryx2_bb_posix::process::ProcessId, UniqueIdGeneratorDetailsError> {
        Service::UniqueId::pid(self.0)
    }

    /// Returns the [`Time`](iceoryx2_bb_posix::clock::Time) the id was created.
    pub fn creation_time<Service: crate::service::Service>(
        &self,
    ) -> Result<iceoryx2_bb_posix::clock::Time, UniqueIdGeneratorDetailsError> {
        Service::UniqueId::creation_time(self.0)
    }

    pub(crate) fn owner_id(&self) -> OwnerId {
        OwnerId::new(self.0.unique_value()).expect("The unique node id is never 0.")
    }
}
