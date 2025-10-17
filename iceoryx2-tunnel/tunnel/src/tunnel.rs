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

use iceoryx2::node::{Node, NodeBuilder, NodeId};
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::{fail, info, trace, warn};
use iceoryx2_tunnel_backend::traits::{
    Backend, Discovery, EventRelay, PublishSubscribeRelay, RelayBuilder, RelayFactory,
};
use iceoryx2_tunnel_backend::types::publish_subscribe::LoanFn;

use crate::discovery;
use crate::ports::event::EventPorts;
use crate::ports::publish_subscribe::PublishSubscribePorts;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Node,
    ServiceName,
    Backend,
    DiscoverySubscriber,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    DiscoveryOverBackend,
    DiscoveryOverService,
    DiscoveryOverTracker,
    PublishSubscribePortCreation,
    PublishSubscribeRelayCreation,
    EventPortsCreation,
    EventRelayCreation,
    DiscoveryAnnouncement,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagateError {
    PayloadPropagation,
    PayloadIngestion,
    EventPropagation,
    EventIngestion,
}

impl core::fmt::Display for PropagateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PropagateError::{self:?}")
    }
}

impl core::error::Error for PropagateError {}

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

#[derive(Debug, Default)]
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
pub struct Tunnel<S: Service, B: for<'a> Backend<S> + Debug> {
    node: Node<S>,
    backend: B,
    ports: Ports<S>,
    relays: Relays<S, B>,
    subscriber: Option<discovery::subscriber::DiscoverySubscriber<S>>,
    tracker: Option<discovery::tracker::DiscoveryTracker<S>>,
}

impl<S: Service, B: for<'a> Backend<S> + Debug> Tunnel<S, B> {
    pub fn create(
        tunnel_config: &Config,
        iceoryx_config: &iceoryx2::config::Config,
        backend_config: &<B as Backend<S>>::Config,
    ) -> Result<Self, CreationError> {
        let origin = format!(
            "Tunnel<{}, {}>::create()",
            core::any::type_name::<S>(),
            core::any::type_name::<B>()
        );

        trace!(
            from origin,
            "Creating Tunnel:\n{:?}\n{:?}\n{:?}",
            &tunnel_config, &iceoryx_config, &backend_config);

        let node = fail!(
            from origin,
            when NodeBuilder::new().config(iceoryx_config).create::<S>(),
            with CreationError::Node,
            "Failed to create Node"
        );

        let backend = fail!(
            from origin,
            when Backend::create(backend_config),
            with CreationError::Backend,
            "Failed to create provided Backend"
        );

        let (subscriber, tracker) = match &tunnel_config.discovery_service {
            Some(service_name) => {
                info!(from origin, "Local Discovery via Subscriber");

                let service_name = fail!(
                    from origin,
                    when service_name.as_str().try_into(),
                    with CreationError::ServiceName,
                    "Failed to create service name {}", service_name
                );

                let subscriber = fail!(
                    from origin,
                    when discovery::subscriber::DiscoverySubscriber::create(&node, service_name),
                    with CreationError::DiscoverySubscriber,
                    "Failed to create discovery subscriber"
                );

                (Some(subscriber), None)
            }
            None => {
                info!(from origin,"Local Discovery via Tracker");
                let tracker = discovery::tracker::DiscoveryTracker::create(iceoryx_config);
                (None, Some(tracker))
            }
        };

        Ok(Self {
            node,
            backend,
            ports: Ports::new(),
            relays: Relays::new(),
            subscriber,
            tracker,
        })
    }

    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        self.discover_over_iceoryx()?;
        self.discover_over_backend()?;

        Ok(())
    }

    pub fn discover_over_iceoryx(&mut self) -> Result<(), DiscoveryError> {
        let tunneled_services = self.tunneled_services();
        if let Some(subscriber) = &mut self.subscriber {
            fail!(
                from self,
                when subscriber.discover(|static_config| {
                    on_discovery(static_config, &self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays)
                }),
                with DiscoveryError::DiscoveryOverService,
                "Failed to discover services via subscriber to discovery service"
            );
        }
        if let Some(tracker) = &mut self.tracker {
            fail!(
                from self,
                when tracker.discover(|static_config| {
                    on_discovery(static_config, &self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays)
                }),
                with DiscoveryError::DiscoveryOverTracker,
                "Failed to discover services via discovery tracker"
            );
        }

        Ok(())
    }

    pub fn discover_over_backend(&mut self) -> Result<(), DiscoveryError> {
        let tunneled_services = self.tunneled_services();
        fail!(
            from self,
            when self.backend.discovery().discover(|static_config| {
                on_discovery(static_config, &self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays)
            }),
            with DiscoveryError::DiscoveryOverBackend,
            "Failed to discover services via Backend"
        );
        Ok(())
    }

    pub fn propagate(&mut self) -> Result<(), PropagateError> {
        for (service_id, port) in &self.ports.publish_subscribe {
            match self.relays.publish_subscribe.get(service_id) {
                Some(relay) => {
                    propagate_publish_subscribe_payloads::<S, B>(self.node.id(), port, relay)?;
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
                    propagate_events::<S, B>(port, relay)?;
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

fn on_discovery<S: Service, B: Backend<S> + Debug>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    services: &HashSet<ServiceId>,
    ports: &mut Ports<S>,
    relays: &mut Relays<S, B>,
) -> Result<(), DiscoveryError> {
    let origin = format!(
        "Tunnel<{}, {}>::on_discovery()",
        core::any::type_name::<S>(),
        core::any::type_name::<B>()
    );

    if services.contains(static_config.service_id()) {
        // Nothing to do.
        return Ok(());
    }

    info!(
        from origin,
        "Discovered {}({})",
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
            info!(
                from origin,
                "Unsupported discovery {}({})",
                static_config.messaging_pattern(),
                static_config.name()
            );
            Ok(())
        }
    }
}

fn setup_publish_subscribe<S: Service, B: Backend<S> + Debug>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    ports: &mut Ports<S>,
    relays: &mut Relays<S, B>,
) -> Result<(), DiscoveryError> {
    let origin = format!(
        "Tunnel<{}, {}>::setup_publish_subscribe()",
        core::any::type_name::<S>(),
        core::any::type_name::<B>()
    );

    let service_id = static_config.service_id();

    let port = fail!(
        from origin,
        when PublishSubscribePorts::new(static_config, node),
        with DiscoveryError::PublishSubscribePortCreation,
        "Failed to create publish-subscribe ports"
    );
    ports.publish_subscribe.insert(*service_id, port);

    let relay = fail!(
        from origin,
        when backend
            .relay_builder()
            .publish_subscribe(static_config)
            .create(),
        with DiscoveryError::PublishSubscribeRelayCreation,
        "Failed to create publish-subscribe relay"
    );
    relays.publish_subscribe.insert(*service_id, relay);

    fail!(
        from origin,
        when backend.discovery().announce(static_config),
        with DiscoveryError::DiscoveryAnnouncement,
        "Failed to announce service over backend"
    );

    Ok(())
}

fn setup_event<S: Service, B: Backend<S> + Debug>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    ports: &mut Ports<S>,
    relays: &mut Relays<S, B>,
) -> Result<(), DiscoveryError> {
    let origin = format!(
        "Tunnel<{}, {}>::setup_event()",
        core::any::type_name::<S>(),
        core::any::type_name::<B>()
    );

    let service_id = static_config.service_id();

    let port = fail!(
        from origin,
        when EventPorts::new(static_config, node),
        with DiscoveryError::EventPortsCreation,
        "Failed to create event ports"
    );
    ports.event.insert(*service_id, port);

    let relay = fail!(
        from origin,
        when backend
            .relay_builder()
            .event(static_config)
            .create(),
        with DiscoveryError::EventRelayCreation,
        "Failed to create event relay"
    );
    relays.event.insert(*service_id, relay);

    fail!(
        from origin,
        when backend.discovery().announce(static_config),
        with DiscoveryError::DiscoveryAnnouncement,
        "Failed to announce service over backend"
    );

    Ok(())
}

fn propagate_publish_subscribe_payloads<S: Service, B: Backend<S> + Debug>(
    node_id: &NodeId,
    port: &PublishSubscribePorts<S>,
    relay: &B::PublishSubscribeRelay,
) -> Result<(), PropagateError> {
    let origin = format!(
        "Tunnel<{}, {}>::propagate_publish_subscribe_payloads()",
        core::any::type_name::<S>(),
        core::any::type_name::<B>()
    );

    let propagated = fail!(
        from origin,
        when port.receive(node_id, |sample| {
            relay.send(sample)
        }),
        with PropagateError::PayloadPropagation,
        "Failed to receive publish-subscribe payload for propagation"
    );
    if propagated {
        info!(
            from origin,
            "Propagated {}({})",
            port.static_config.messaging_pattern(),
            port.static_config.name()
        );
    }

    let ingested = fail!(
        from origin,
        when port.send(|loan: &mut LoanFn<_, _>| {
            relay.receive::<_>(&mut |size| {
            loan(size)})
        }),
        with PropagateError::PayloadIngestion,
        "Failed to ingest publish-subscribe payload received from backend"
    );
    if ingested {
        info!(
            from origin,
            "Ingested {}({})",
            port.static_config.messaging_pattern(),
            port.static_config.name()
        );
    }

    Ok(())
}

fn propagate_events<S: Service, B: Backend<S> + Debug>(
    port: &EventPorts<S>,
    relay: &B::EventRelay,
) -> Result<(), PropagateError> {
    let origin = format!(
        "Tunnel<{}, {}>::propagate_events()",
        core::any::type_name::<S>(),
        core::any::type_name::<B>()
    );

    let propagated = fail!(
        from origin,
        when port.receive(|id| {
            relay.send(id)
        }),
        with PropagateError::EventPropagation,
        "Failed to receive events for propagation"
    );
    if propagated {
        info!(
            from origin,
            "Propagated {}({})",
            port.static_config.messaging_pattern(),
            port.static_config.name()
        );
    }

    let ingested = fail!(
        from origin,
        when port.send(|| {
            relay.receive()
        }),
        with PropagateError::EventIngestion,
        "Failed to ingest event received from backend"
    );
    if ingested {
        info!(
            from origin,
            "Ingested {}({})",
            port.static_config.messaging_pattern(),
            port.static_config.name()
        );
    }

    Ok(())
}
