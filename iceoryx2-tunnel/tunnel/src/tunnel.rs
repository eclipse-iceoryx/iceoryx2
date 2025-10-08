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

use core::fmt::Debug;
use std::collections::{HashMap, HashSet};

use crate::discovery;
use crate::ports::event::EventPorts;
use crate::ports::publish_subscribe::PublishSubscribePorts;
use iceoryx2::node::{Node, NodeBuilder, NodeId};
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::{debug, fail, warn};
use iceoryx2_tunnel_backend::traits::{
    Backend, Discovery, EventRelay, PublishSubscribeRelay, RelayBuilder, RelayFactory,
};
use iceoryx2_tunnel_backend::types::publish_subscribe::LoanFn;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailedToCreateNode,
    FailedToCreateBackend,
    FailedToCreateDiscoverySubscriber,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    FailedToDiscover,
    FailedToDiscoverOverSubscriber,
    FailedToDiscoverOverTracker,
    FailedToDiscoverOverBackend,
    FailedToCreatePublishSubscribePorts,
    FailedToCreatePublishSubscribeRelay,
    FailedToCreateEventPorts,
    FailedToCreateEventRelay,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagateError {
    FailedToPropagatePublishSubscribePayloadToBackend,
    FailedToPropagatePublishSubscribePayloadToIceoryx,
    FailedToPropagateEventToBackend,
    FailedToPropagateEventToIceoryx,
}

#[derive(Debug)]
pub(crate) struct Ports<S: Service> {
    pub(crate) publish_subscribe: HashMap<ServiceId, PublishSubscribePorts<S>>,
    pub(crate) event: HashMap<ServiceId, EventPorts<S>>,
}

impl<S: Service> Ports<S> {
    pub fn new() -> Self {
        Self {
            publish_subscribe: HashMap::new(),
            event: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Relays<S: Service, B: Backend<S>> {
    publish_subscribe: HashMap<ServiceId, B::PublishSubscribeRelay>,
    event: HashMap<ServiceId, B::EventRelay>,
}

impl<S: Service, B: Backend<S>> Relays<S, B> {
    pub fn new() -> Self {
        Self {
            publish_subscribe: HashMap::new(),
            event: HashMap::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    pub discovery_service: Option<String>,
}

#[derive(Debug)]
pub struct Tunnel<S: Service, B: for<'a> Backend<S>> {
    node: Node<S>,
    backend: B,
    ports: Ports<S>,
    relays: Relays<S, B>,
    subscriber: Option<discovery::subscriber::DiscoverySubscriber<S>>,
    tracker: Option<discovery::tracker::DiscoveryTracker<S>>,
}

impl<S: Service, B: for<'a> Backend<S>> Tunnel<S, B> {
    pub fn create(
        tunnel_config: &Config,
        iceoryx_config: &iceoryx2::config::Config,
        backend_config: &<B as Backend<S>>::Config,
    ) -> Result<Self, CreationError> {
        debug!(from "Tunnel::create", "Creating tunnel with configuration:\n{:?}\n{:?}\n{:?}", &tunnel_config, &iceoryx_config, &backend_config);

        let node = fail!(
            from "Tunnel::create",
            when NodeBuilder::new().config(iceoryx_config).create::<S>(),
            with CreationError::FailedToCreateNode,
            "Failed to create Node"
        );

        let backend = fail!(
            from "Tunnel::create",
            when Backend::create(backend_config),
            with CreationError::FailedToCreateBackend,
            "Failed to create provided Backend"
        );

        let (subscriber, tracker) = match &tunnel_config.discovery_service {
            Some(service_name) => {
                debug!(from "Tunnel::create", "Local Discovery via Subscriber");
                let subscriber = fail!(
                    from "Tunnel::create",
                    when discovery::subscriber::DiscoverySubscriber::create(&node, service_name),
                    with CreationError::FailedToCreateDiscoverySubscriber,
                    "Failed to create discovery subscriber"
                );
                (Some(subscriber), None)
            }
            None => {
                debug!(from "Tunnel::create","Local Discovery via Tracker");
                let tracker = discovery::tracker::DiscoveryTracker::create(iceoryx_config);
                (None, Some(tracker))
            }
        };

        Ok(Self {
            node,
            backend,
            ports: Ports::new(),
            relays: Relays::new(),
            subscriber: subscriber,
            tracker: tracker,
        })
    }

    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        fail!(
            from "Tunnel::discover",
            when self.discover_over_iceoryx(),
            with DiscoveryError::FailedToDiscover,
            "Failed to discover services over iceoryx"
        );
        fail!(
            from "Tunnel::discover",
            when self.discover_over_backend(),
            with DiscoveryError::FailedToDiscover,
            "Failed to discover services over backend"
        );

        Ok(())
    }

    pub fn discover_over_iceoryx(&mut self) -> Result<(), DiscoveryError> {
        let tunneled_services = self.tunneled_services();
        if let Some(subscriber) = &mut self.subscriber {
            fail!(
                from "Tunnel::discover_over_iceoryx",
                when subscriber.discover(&mut |static_config| {
                    on_discovery(static_config, &mut self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays)
                }),
                with DiscoveryError::FailedToDiscoverOverSubscriber,
                "Failed to discover services via subscriber to discovery service"
            );
        }
        if let Some(tracker) = &mut self.tracker {
            fail!(
                from "Tunnel::discover_over_iceoryx",
                when tracker.discover(&mut |static_config| {
                    on_discovery(static_config, &mut self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays)
                }),
                with DiscoveryError::FailedToDiscoverOverTracker,
                "Failed to discover services via discovery tracker"
            );
        }

        Ok(())
    }

    pub fn discover_over_backend(&mut self) -> Result<(), DiscoveryError> {
        let tunneled_services = self.tunneled_services();
        fail!(
            from "Tunnel::discover_over_backend",
            when self.backend.discovery().discover(&mut |static_config| {
                on_discovery(static_config, &self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays)
            }),
            with DiscoveryError::FailedToDiscoverOverBackend,
            "Failed to discover services via Backend"
        );
        Ok(())
    }

    pub fn propagate(&mut self) -> Result<(), PropagateError> {
        for (service_id, port) in &self.ports.publish_subscribe {
            match self.relays.publish_subscribe.get(service_id) {
                Some(relay) => {
                    fail!(
                        from "Tunnel::propagate",
                        when propagate_publish_subscribe_payloads::<S, B>(self.node.id(), port, relay),
                        "Failed to propagate publish subscribe payloads"
                    );
                }
                None => {
                    warn!(from "Tunnel::propagate", "No relay available for {:?}", service_id);
                    return Ok(());
                }
            };
        }

        for (service_id, port) in &self.ports.event {
            match self.relays.event.get(service_id) {
                Some(relay) => {
                    fail!(
                        from "Tunnel::propagate",
                        when propagate_events::<S, B>(port, relay),
                        "Failed to propagate publish subscribe payloads"
                    );
                }
                None => {
                    warn!(from "Tunnel::propagate", "No relay available for {:?}", service_id);
                    return Ok(());
                }
            };
        }

        Ok(())
    }

    pub fn tunneled_services(&self) -> HashSet<ServiceId> {
        self.ports
            .publish_subscribe
            .keys()
            .chain(self.ports.event.keys())
            .cloned()
            .collect()
    }
}

fn on_discovery<S: Service, B: Backend<S>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    services: &HashSet<ServiceId>,
    ports: &mut Ports<S>,
    relays: &mut Relays<S, B>,
) -> Result<(), DiscoveryError> {
    if services.contains(static_config.service_id()) {
        // Nothing to do.
        return Ok(());
    }

    debug!(
        from "Tunnel::on_discovery",
        "Discovered service {}({})",
        static_config.messaging_pattern(),
        static_config.name()
    );

    match static_config.messaging_pattern() {
        MessagingPattern::PublishSubscribe(_) => {
            setup_publish_subscribe(static_config, node, backend, ports, relays)
        }
        MessagingPattern::Event(_) => setup_event(static_config, node, backend, ports, relays),
        _ => {
            // Not supported. Nothing to do.
            debug!(
                from "Tunnel::on_discovery",
                "Skipping unsupported discovery {}({})",
                static_config.messaging_pattern(),
                static_config.name()
            );
            Ok(())
        }
    }
}

fn setup_publish_subscribe<S: Service, B: Backend<S>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    ports: &mut Ports<S>,
    relays: &mut Relays<S, B>,
) -> Result<(), DiscoveryError> {
    let service_id = static_config.service_id();

    let port = fail!(
        from "setup_publish_subscribe",
        when PublishSubscribePorts::new(static_config, &node),
        with DiscoveryError::FailedToCreatePublishSubscribePorts,
        "Failed to create publish-subscribe ports"
    );

    let relay = fail!(
        from "setup_publish_subscribe",
        when backend
            .relay_builder()
            .publish_subscribe(static_config)
            .create(),
        with DiscoveryError::FailedToCreatePublishSubscribeRelay,
        "Failed to create publish-subscribe relay"
    );

    ports.publish_subscribe.insert(service_id.clone(), port);
    relays.publish_subscribe.insert(service_id.clone(), relay);

    Ok(())
}

fn setup_event<S: Service, B: Backend<S>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    ports: &mut Ports<S>,
    relays: &mut Relays<S, B>,
) -> Result<(), DiscoveryError> {
    let service_id = static_config.service_id();

    let port = fail!(
        from "setup_event",
        when EventPorts::new(static_config, &node),
        with DiscoveryError::FailedToCreateEventPorts,
        "Failed to create event ports"
    );

    let relay = fail!(
        from "setup_event",
        when backend
            .relay_builder()
            .event(static_config)
            .create(),
        with DiscoveryError::FailedToCreateEventRelay,
        "Failed to create event relay"
    );

    ports.event.insert(service_id.clone(), port);
    relays.event.insert(service_id.clone(), relay);

    Ok(())
}

fn propagate_publish_subscribe_payloads<S: Service, B: Backend<S>>(
    node_id: &NodeId,
    port: &PublishSubscribePorts<S>,
    relay: &B::PublishSubscribeRelay,
) -> Result<(), PropagateError> {
    fail!(
        from "propagate_publish_subscribe_payloads",
        when port.receive(node_id, |sample| {
            debug!(
                from "propagate_publish_subscribe_payloads",
                "Propagating from {}({})",
                port.static_config.messaging_pattern(),
                port.static_config.name()
            );

            relay.send(sample)
        }),
        with PropagateError::FailedToPropagatePublishSubscribePayloadToBackend,
        "Failed to receive publish-subscribe payload for propagation"
    );
    fail!(
        from "propagate_publish_subscribe_payloads",
        when port.send(|loan: &mut LoanFn<_, _>| {
            relay.receive::<_>(&mut |size| {
            debug!(
                from "propagate_publish_subscribe_payloads",
                "Ingesting into {}({})",
                port.static_config.messaging_pattern(),
                port.static_config.name()
            );

            loan(size)})
        }),
        with PropagateError::FailedToPropagatePublishSubscribePayloadToIceoryx,
        "Failed to ingest publish-subscribe payload received from backend"
    );

    Ok(())
}

fn propagate_events<S: Service, B: Backend<S>>(
    port: &EventPorts<S>,
    relay: &B::EventRelay,
) -> Result<(), PropagateError> {
    fail!(
        from "propagate_events",
        when port.receive(|id| {
            debug!(
                from "propagate_events",
                "Propagating from {}({})",
                port.static_config.messaging_pattern(),
                port.static_config.name()
            );

            relay.send(id)
        }),
        with PropagateError::FailedToPropagateEventToBackend,
        "Failed to receive events for propagation"
    );

    fail!(
        from "propagate_events",
        when relay.receive(&mut |id| {
            debug!(
                from "propagate_events",
                "Ingesting into {}({})",
                port.static_config.messaging_pattern(),
                port.static_config.name()
            );

            port.send(id)
        }),
        with PropagateError::FailedToPropagateEventToIceoryx,
        "Failed to ingest event received from backend"
    );

    Ok(())
}
