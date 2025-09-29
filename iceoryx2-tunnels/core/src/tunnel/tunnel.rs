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

use std::collections::{HashMap, HashSet};

use crate::{Discovery, Relay, RelayBuilder, RelayFactory, Transport};

use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::port::publisher::{Publisher, PublisherCreateError};
use iceoryx2::port::subscriber::{Subscriber, SubscriberCreateError};
use iceoryx2::prelude::{AllocationStrategy, PortFactory};
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::{fail, info};
use iceoryx2_services_discovery::service_discovery::Discovery as DiscoveryEvent;
use iceoryx2_services_discovery::service_discovery::{SyncError, Tracker};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Discovery,
    Connection,
    Error,
}

impl From<super::discovery::Error> for Error {
    fn from(_: super::discovery::Error) -> Self {
        Error::Discovery
    }
}

impl From<PublishSubscribeOpenOrCreateError> for Error {
    fn from(_: PublishSubscribeOpenOrCreateError) -> Self {
        Error::Connection
    }
}

impl From<PublisherCreateError> for Error {
    fn from(_: PublisherCreateError) -> Self {
        Error::Connection
    }
}

impl From<SubscriberCreateError> for Error {
    fn from(_: SubscriberCreateError) -> Self {
        Error::Connection
    }
}

impl From<SyncError> for Error {
    fn from(_: SyncError) -> Self {
        Error::Discovery
    }
}

#[derive(Default)]
pub struct Config {
    pub discovery_service: Option<String>,
}

pub(crate) enum Ports<S: Service> {
    PublishSubscribe(
        Publisher<S, [CustomPayloadMarker], CustomHeaderMarker>,
        Subscriber<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ),
    Event(Notifier<S>, Listener<S>),
}

/// A generic tunnel implementation that works with any implemented Transport.
pub struct Tunnel<S: Service, T: Transport> {
    iceoryx_config: iceoryx2::config::Config,
    node: Node<S>,
    transport: T,
    services: HashSet<ServiceId>,
    ports: HashMap<ServiceId, Ports<S>>,
    relays: HashMap<ServiceId, Box<dyn Relay>>,
    subscriber: Option<Subscriber<S, DiscoveryEvent, ()>>,
    tracker: Option<Tracker<S>>,
}

impl<S: Service, T: Transport + RelayFactory<T>> Tunnel<S, T> {
    /// create a new tunnel instance using the given transport
    pub fn create(
        tunnel_config: &Config,
        iceoryx_config: &iceoryx2::config::Config,
        transport_config: &T::Config,
    ) -> Result<Self, Error> {
        let node = NodeBuilder::new().config(iceoryx_config).create::<S>();
        let node = fail!(
            from "Tunnel::<S, T>::create",
            when node,
            with Error::Error,
            "failed to create node"
        );

        let transport = Transport::create(transport_config);
        let transport = fail!(
            from "Tunnel::<S, T>::create",
            when transport,
            with Error::Error,
            "failed to instantiate the transport"
        );

        let (subscriber, tracker) = match &tunnel_config.discovery_service {
            Some(service_name) => {
                let service_name = fail!(
                    from "Tunnel::<S, T>::create",
                    when service_name.as_str().try_into(),
                    with Error::Error,
                    "failed to create service name for discovery service"
                );

                let service = fail!(
                    from "Tunnel::<S, T>::create",
                    when node.service_builder(&service_name)
                            .publish_subscribe::<DiscoveryEvent>()
                            .open_or_create(),
                    with Error::Error,
                    "failed to open or create iceoryx discovery service"
                );
                let subscriber = fail!(
                    from "Tunnel::<S, T>::create",
                    when service.subscriber_builder().create(),
                    with Error::Error,
                    "failed to create subscriber to iceoryx discovery service"
                );

                info!("CONFIGURE DiscoveryService {}", service_name);
                (Some(subscriber), None)
            }
            None => {
                let tracker = Tracker::new(iceoryx_config);
                (None, Some(tracker))
            }
        };

        Ok(Self {
            iceoryx_config: iceoryx_config.clone(),
            node,
            transport,
            services: HashSet::new(),
            ports: HashMap::new(),
            relays: HashMap::new(),
            subscriber: subscriber,
            tracker: tracker,
        })
    }

    /// discover services across the extended system
    pub fn discovery(&mut self) -> Result<(), Error> {
        let node = &mut self.node;
        let transport = &mut self.transport;
        let services = &mut self.services;
        let ports = &mut self.ports;
        let relays = &mut self.relays;

        if let Some(tracker) = &mut self.tracker {
            fail!(
                from "Tunnel::<S, T>::discover",
                when tracker.discover(&mut |static_config| {
                    match static_config.messaging_pattern(){
                        MessagingPattern::PublishSubscribe(_) => {
                            info!("Discovered: PublishSubscribe({})", static_config.name());
                            setup_publish_subscribe::<S, T>(static_config, node, transport, services, ports, relays).unwrap();
                            Ok(())
                        },
                        MessagingPattern::Event(_) => {
                            info!("Discovered: Event({})", static_config.name());
                            Ok(())
                        },
                        _ => {
                            // Not supported. Nothing to do.
                            info!("Unsupported Discovery: {}({})", static_config.messaging_pattern(), static_config.name());
                            Ok(())
                        },
                    }
                }),
                "failed to discover services"
            );
        }

        Ok(())
    }

    /// propagate payloads data over all relays
    pub fn propagate(&mut self) -> Result<(), Error> {
        for (id, ports) in &self.ports {
            let relay = &self.relays.get(id);

            match ports {
                Ports::PublishSubscribe(publisher, subscriber) => todo!(),
                Ports::Event(notifier, listener) => todo!(),
            }
        }

        Ok(())
    }

    pub fn tunneled_services(&self) -> &HashSet<ServiceId> {
        &self.services
    }
}

fn setup_publish_subscribe<S: Service, T: Transport + RelayFactory<T>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    transport: &T,
    services: &mut HashSet<ServiceId>,
    ports: &mut HashMap<ServiceId, Ports<S>>,
    relays: &mut HashMap<ServiceId, Box<dyn Relay>>,
) -> Result<(), Error> {
    let service_id = static_config.service_id();

    if services.contains(service_id) {
        return Ok(());
    }

    let port_config = static_config.publish_subscribe();
    let service = unsafe {
        fail!(
            from "Tunnel::setup_publish_subscribe()",
            when node.service_builder(static_config.name())
                    .publish_subscribe::<[CustomPayloadMarker]>()
                    .user_header::<CustomHeaderMarker>()
                    .__internal_set_user_header_type_details(
                        &port_config.message_type_details().user_header,
                    )
                    .__internal_set_payload_type_details(
                        &port_config.message_type_details().payload,
                    )
                    .enable_safe_overflow(port_config.has_safe_overflow())
                    .history_size(port_config.history_size())
                    .max_nodes(port_config.max_nodes())
                    .max_publishers(port_config.max_publishers())
                    .max_subscribers(port_config.max_subscribers())
                    .subscriber_max_buffer_size(port_config.subscriber_max_buffer_size())
                    .subscriber_max_borrowed_samples(
                        port_config.subscriber_max_borrowed_samples(),
                    )
                    .open_or_create(),
            "failed to open or create publish-subscribe service"
        )
    };

    let publisher = fail!(
        from "Tunnel::setup_publish_subscribe()",
        when service
            .publisher_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create(),
        "failed to create publisher"
    );

    let subscriber = fail!(
        from "Tunnel::setup_publish_subscribe()",
        when service.subscriber_builder().create(),
        "failed to create subscriber"
    );

    // TODO: How to use fail! when the concrete error type is not known?
    let relay = transport
        .publish_subscribe(service.name())
        .create()
        .unwrap();

    services.insert(service.service_id().clone());
    ports.insert(
        service.service_id().clone(),
        Ports::PublishSubscribe(publisher, subscriber),
    );
    relays.insert(service.service_id().clone(), relay);

    Ok(())
}
