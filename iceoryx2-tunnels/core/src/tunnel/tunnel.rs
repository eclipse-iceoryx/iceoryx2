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

use crate::tunnel::ports::Ports;
use crate::tunnel::{discovery, ports};
use iceoryx2_tunnels_traits::{Discovery, Relay, RelayBuilder, RelayFactory, Transport};

use iceoryx2::node::{Node, NodeBuilder, NodeCreationFailure};
use iceoryx2::prelude::PortFactory;
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::{debug, fail, warn};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailedToCreateNodeNode,
    FailedToCreateService,
    FailedToCreateTransport,
    FailedToCreateSubscriber,
    FailedToCreateTracker,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    FailedToDiscoverOverSubscriber,
    FailedToDiscoverOverTracker,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationOrIngestionError {
    FailedToPropagatePublishSubscribePayload,
    FailedToIngestPublishSubscribePayload,
}

impl From<discovery::subscriber::CreationError> for CreationError {
    fn from(_: discovery::subscriber::CreationError) -> Self {
        CreationError::FailedToCreateSubscriber
    }
}

impl From<discovery::subscriber::DiscoveryError> for DiscoveryError {
    fn from(_: discovery::subscriber::DiscoveryError) -> Self {
        DiscoveryError::FailedToDiscoverOverSubscriber
    }
}

impl From<discovery::tracker::DiscoveryError> for DiscoveryError {
    fn from(_: discovery::tracker::DiscoveryError) -> Self {
        DiscoveryError::FailedToDiscoverOverTracker
    }
}

impl From<ports::publish_subscribe::PropagationError> for PropagationOrIngestionError {
    fn from(_: ports::publish_subscribe::PropagationError) -> Self {
        PropagationOrIngestionError::FailedToPropagatePublishSubscribePayload
    }
}

impl From<ports::publish_subscribe::IngestionError> for PropagationOrIngestionError {
    fn from(_: ports::publish_subscribe::IngestionError) -> Self {
        PropagationOrIngestionError::FailedToIngestPublishSubscribePayload
    }
}

impl From<NodeCreationFailure> for CreationError {
    fn from(_: NodeCreationFailure) -> Self {
        CreationError::FailedToCreateNodeNode
    }
}

impl From<PublishSubscribeOpenOrCreateError> for CreationError {
    fn from(_: PublishSubscribeOpenOrCreateError) -> Self {
        CreationError::FailedToCreateService
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub discovery_service: Option<String>,
}

/// A generic tunnel implementation that works with any implemented Transport.
pub struct Tunnel<S: Service, T: Transport> {
    node: Node<S>,
    transport: T,
    services: HashSet<ServiceId>,
    ports: HashMap<ServiceId, Ports<S>>,
    relays: HashMap<ServiceId, Box<dyn Relay>>,
    subscriber: Option<discovery::subscriber::DiscoverySubscriber<S>>,
    tracker: Option<discovery::tracker::DiscoveryTracker<S>>,
}

impl<S: Service, T: Transport + RelayFactory<T>> Tunnel<S, T> {
    /// Create a new tunnel instance that uses the specified Transport
    pub fn create(
        tunnel_config: &Config,
        iceoryx_config: &iceoryx2::config::Config,
        transport_config: &T::Config,
    ) -> Result<Self, CreationError> {
        let node = fail!(
            from "Tunnel::<S, T>::create",
            when NodeBuilder::new().config(iceoryx_config).create::<S>(),
            "failed to create node"
        );

        let transport = fail!(
            from "Tunnel::<S, T>::create",
            when Transport::create(transport_config),
            with CreationError::FailedToCreateTransport,
            "failed to instantiate the transport"
        );

        let (subscriber, tracker) = match &tunnel_config.discovery_service {
            Some(service_name) => {
                debug!("Discovery via Subscriber");
                let subscriber =
                    discovery::subscriber::DiscoverySubscriber::new(&node, service_name)?;
                (Some(subscriber), None)
            }
            None => {
                debug!("Discovery via Tracker");

                let tracker = discovery::tracker::DiscoveryTracker::new(iceoryx_config);
                (None, Some(tracker))
            }
        };

        Ok(Self {
            node,
            transport,
            services: HashSet::new(),
            ports: HashMap::new(),
            relays: HashMap::new(),
            subscriber: subscriber,
            tracker: tracker,
        })
    }

    /// Discover services via iceoryx2 and the transport
    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        if let Some(subscriber) = &mut self.subscriber {
            fail!(
                from "Tunnel::<S, T>::discover",
                when discovery::subscriber::DiscoverySubscriber::discover(subscriber, &mut |static_config| {
                    on_discovery::<S, T>(static_config, &mut self.node, &mut self.transport, &mut self.services, &mut self.ports, &mut self.relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Subscriber"
            );
        }
        if let Some(tracker) = &mut self.tracker {
            fail!(
                from "Tunnel::<S, T>::discover",
                when discovery::tracker::DiscoveryTracker::discover(tracker, &mut |static_config| {
                    on_discovery::<S, T>(static_config, &mut self.node, &mut self.transport, &mut self.services, &mut self.ports, &mut self.relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Tracker"
            );
        }

        Ok(())
    }

    /// Propagate payloads between iceoryx2 and the Transport
    pub fn propagate(&mut self) -> Result<(), PropagationOrIngestionError> {
        for (id, ports) in &self.ports {
            let relay = match self.relays.get(id) {
                Some(relay) => relay,
                None => {
                    warn!("No relay available for id {:?}. Skipping.", id);
                    return Ok(());
                }
            };

            match ports {
                Ports::PublishSubscribe(ports) => {
                    fail!(
                        from "Tunnel::<S, T>::propagate",
                        when ports.propagate(self.node.id(), relay),
                        "Failed to propagate PublishSubscribe payload"
                    );
                    fail!(
                        from "Tunnel::<S, T>::propagate",
                        when ports.ingest(relay),
                        "Failed to ingest PublishSubscribe payload"
                    );
                }
                Ports::Event(_) => todo!(),
            }
        }

        Ok(())
    }

    pub fn tunneled_services(&self) -> &HashSet<ServiceId> {
        &self.services
    }
}

fn on_discovery<S: Service, T: Transport + RelayFactory<T>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    transport: &T,
    services: &mut HashSet<ServiceId>,
    ports: &mut HashMap<ServiceId, Ports<S>>,
    relays: &mut HashMap<ServiceId, Box<dyn Relay>>,
) -> Result<(), CreationError> {
    match static_config.messaging_pattern() {
        MessagingPattern::PublishSubscribe(_) => {
            debug!("Discovered: PublishSubscribe({})", static_config.name());
            setup_publish_subscribe::<S, T>(static_config, node, transport, services, ports, relays)
        }
        MessagingPattern::Event(_) => {
            debug!("Discovered: Event({})", static_config.name());
            Ok(())
        }
        _ => {
            // Not supported. Nothing to do.
            debug!(
                "Unsupported Discovery: {}({})",
                static_config.messaging_pattern(),
                static_config.name()
            );
            Ok(())
        }
    }
}

fn setup_publish_subscribe<S: Service, T: Transport + RelayFactory<T>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    transport: &T,
    services: &mut HashSet<ServiceId>,
    ports: &mut HashMap<ServiceId, Ports<S>>,
    relays: &mut HashMap<ServiceId, Box<dyn Relay>>,
) -> Result<(), CreationError> {
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
            "{}", format!("Failed to open or create publish-subscribe service '{}'", static_config.name())
        )
    };

    // TODO: Use fail!
    let port = ports::publish_subscribe::Ports::new(&service).unwrap();

    // TODO: Use fail!
    let relay = transport
        .publish_subscribe(service.name())
        .create()
        .unwrap();

    services.insert(service.service_id().clone());
    ports.insert(service.service_id().clone(), Ports::PublishSubscribe(port));
    relays.insert(service.service_id().clone(), relay);

    Ok(())
}
