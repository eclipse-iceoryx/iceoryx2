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

use core::mem::MaybeUninit;
use std::collections::{HashMap, HashSet};

use crate::tunnel::discovery::DiscoverySubscriber;
use crate::{Discovery, Relay, RelayBuilder, RelayFactory, Transport};

use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::port::publisher::{Publisher, PublisherCreateError};
use iceoryx2::port::subscriber::{Subscriber, SubscriberCreateError};
use iceoryx2::port::ReceiveError;
use iceoryx2::prelude::{AllocationStrategy, PortFactory};
use iceoryx2::sample_mut_uninit::SampleMutUninit;
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::{debug, fail, fatal_panic, warn};
use iceoryx2_services_discovery::service_discovery::Discovery as DiscoveryEvent;
use iceoryx2_services_discovery::service_discovery::{SyncError, Tracker};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Discovery,
    Connection,
    Propagation,
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

impl From<ReceiveError> for Error {
    fn from(_: ReceiveError) -> Self {
        Error::Discovery
    }
}

#[derive(Debug, Default)]
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
    node: Node<S>,
    transport: T,
    services: HashSet<ServiceId>,
    ports: HashMap<ServiceId, Ports<S>>,
    relays: HashMap<ServiceId, Box<dyn Relay>>,
    subscriber: Option<DiscoverySubscriber<S>>,
    tracker: Option<Tracker<S>>,
}

impl<S: Service, T: Transport + RelayFactory<T>> Tunnel<S, T> {
    /// Create a new tunnel instance that uses the specified Transport
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
                debug!("Discovery via Subscriber");
                let subscriber = create_discovery_subscriber::<S>(&node, service_name)?;
                (Some(subscriber), None)
            }
            None => {
                debug!("Discovery via Tracker");

                let tracker = Tracker::new(iceoryx_config);
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
    pub fn discovery(&mut self) -> Result<(), Error> {
        let node = &mut self.node;
        let transport = &mut self.transport;
        let services = &mut self.services;
        let ports = &mut self.ports;
        let relays = &mut self.relays;

        if let Some(subscriber) = &mut self.subscriber {
            fail!(
                from "Tunnel::<S, T>::discover",
                when DiscoverySubscriber::discover(subscriber, &mut |static_config| {
                    on_discovery::<S, T>(static_config, node, transport, services, ports, relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Subscriber"
            );
        }
        if let Some(tracker) = &mut self.tracker {
            fail!(
                from "Tunnel::<S, T>::discover",
                when Tracker::discover(tracker, &mut |static_config| {
                    on_discovery::<S, T>(static_config, node, transport, services, ports, relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Tracker"
            );
        }

        Ok(())
    }

    /// Propagate payloads between iceoryx2 and the Transport
    pub fn propagate(&mut self) -> Result<(), Error> {
        for (id, ports) in &self.ports {
            let relay = match self.relays.get(id) {
                Some(relay) => relay,
                None => {
                    warn!("No relay available for id: {:?}", id);
                    return Ok(());
                }
            };

            match ports {
                Ports::PublishSubscribe(publisher, subscriber) => {
                    propagate_publish_subscribe_payload(self.node.id(), subscriber, relay).unwrap();
                    ingest_publish_subscribe_payload(publisher, relay).unwrap();
                }
                Ports::Event(_, _) => todo!(),
            }
        }

        Ok(())
    }

    pub fn tunneled_services(&self) -> &HashSet<ServiceId> {
        &self.services
    }
}

/// Create a discovery subscriber for the given service name
fn create_discovery_subscriber<S: Service>(
    node: &Node<S>,
    service_name: &str,
) -> Result<DiscoverySubscriber<S>, Error> {
    let service_name = fail!(
        from "Tunnel::<S, T>::create_discovery_subscriber",
        when service_name.try_into(),
        with Error::Error,
        "{}", &format!("Failed to create ServiceName '{}'", service_name)
    );

    let service = fail!(
        from "Tunnel::<S, T>::create_discovery_subscriber",
        when node.service_builder(&service_name)
                .publish_subscribe::<DiscoveryEvent>()
                .open_or_create(),
        with Error::Error,
        "{}", &format!("Failed to open DiscoveryService with ServiceName '{}'", service_name)
    );

    let subscriber = fail!(
        from "Tunnel::<S, T>::create_discovery_subscriber",
        when service.subscriber_builder().create(),
        with Error::Error,
        "{}", &format!("Failed to create DiscoverySubscriber with ServiceName '{}'", service_name)
    );

    Ok(DiscoverySubscriber(subscriber))
}

fn on_discovery<S: Service, T: Transport + RelayFactory<T>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    transport: &T,
    services: &mut HashSet<ServiceId>,
    ports: &mut HashMap<ServiceId, Ports<S>>,
    relays: &mut HashMap<ServiceId, Box<dyn Relay>>,
) -> Result<(), Error> {
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
            "{}", format!("Failed to open or create publish-subscribe service '{}'", static_config.name())
        )
    };

    let publisher = fail!(
        from "Tunnel::setup_publish_subscribe()",
        when service
            .publisher_builder()
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create(),
        "{}", &format!("Failed to create Publisher for '{}'", service.name())
    );

    let subscriber = fail!(
        from "Tunnel::setup_publish_subscribe()",
        when service.subscriber_builder().create(),
        "{}", &format!("Failed to create Subscriber for '{}'", service.name())
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

fn propagate_publish_subscribe_payload<S: Service>(
    node_id: &iceoryx2::node::NodeId,
    subscriber: &Subscriber<S, [CustomPayloadMarker], CustomHeaderMarker>,
    relay: &Box<dyn Relay>,
) -> Result<bool, Error> {
    match unsafe { subscriber.receive_custom_payload() } {
        Ok(Some(sample)) => {
            if sample.header().node_id() == *node_id {
                // Ignore samples published by the gateway itself to prevent loopback.
                return Ok(true);
            }
            let ptr = sample.payload().as_ptr() as *const u8;
            let len = sample.len();

            relay.propagate(ptr, len);
            Ok(true)
        }
        Ok(None) => Ok(false),
        Err(e) => fatal_panic!("Failed to receive custom payload: {}", e),
    }
}

fn ingest_publish_subscribe_payload<S: Service>(
    publisher: &Publisher<S, [CustomPayloadMarker], CustomHeaderMarker>,
    relay: &Box<dyn Relay>,
) -> Result<(), Error> {
    let payload_size = 1; // TODO: Get from Service
    let mut loaned_sample: Option<
        SampleMutUninit<S, [MaybeUninit<CustomPayloadMarker>], CustomHeaderMarker>,
    > = None;

    let ingested = relay.ingest(&mut |number_of_bytes| {
        let number_of_elements = number_of_bytes / payload_size;

        let (ptr, len) = match unsafe { publisher.loan_custom_payload(number_of_elements) } {
            Ok(mut sample) => {
                let payload = sample.payload_mut();
                let ptr = payload.as_mut_ptr() as *mut u8;
                let len = payload.len();

                loaned_sample = Some(sample);

                (ptr, len)
            }
            Err(e) => fatal_panic!("Failed to loan custom payload: {e}"),
        };

        (ptr, len)
    });

    if ingested {
        if let Some(sample) = loaned_sample {
            let sample = unsafe { sample.assume_init() };
            fail!(
                from "ingest_publish_subscribe_payload",
                when sample.send(),
                with Error::Propagation,
                "Failed to send ingested payload"
            );
        }
    }

    Ok(())
}
