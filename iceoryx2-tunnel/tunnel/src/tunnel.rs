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

use crate::ports::event::EventPorts;
use crate::ports::publish_subscribe::PublishSubscribePorts;
use crate::{discovery, ports};
use iceoryx2::node::{Node, NodeBuilder, NodeCreationFailure, NodeId};
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
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
    FailedToCreateNodeNode,
    FailedToCreateService,
    FailedToCreateBackend,
    FailedToCreateSubscriber,
    FailedToCreateTracker,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    FailedToDiscoverOverSubscriber,
    FailedToDiscoverOverTracker,
    FailedToDiscoverOverBackend,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagateError {
    Error,
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

impl From<ports::publish_subscribe::SendError> for PropagateError {
    fn from(_: ports::publish_subscribe::SendError) -> Self {
        PropagateError::Error
    }
}

impl From<ports::publish_subscribe::ReceiveError> for PropagateError {
    fn from(_: ports::publish_subscribe::ReceiveError) -> Self {
        PropagateError::Error
    }
}

impl From<ports::event::NotifyError> for PropagateError {
    fn from(_: ports::event::NotifyError) -> Self {
        PropagateError::Error
    }
}

impl From<ports::event::WaitError> for PropagateError {
    fn from(_: ports::event::WaitError) -> Self {
        PropagateError::Error
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
        let node = fail!(
            from "Tunnel::create",
            when NodeBuilder::new().config(iceoryx_config).create::<S>(),
            "failed to create node"
        );

        let backend = fail!(
            from "Tunnel::create",
            when Backend::create(backend_config),
            with CreationError::FailedToCreateBackend,
            "failed to instantiate the backend"
        );

        let (subscriber, tracker) = match &tunnel_config.discovery_service {
            Some(service_name) => {
                debug!(from "Tunnel::create", "Local Discovery via Subscriber");
                let subscriber =
                    discovery::subscriber::DiscoverySubscriber::create(&node, service_name)?;
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
        // TODO: Handle errors
        self.discover_over_iceoryx().unwrap();
        self.discover_over_backend().unwrap();

        Ok(())
    }

    pub fn discover_over_iceoryx(&mut self) -> Result<(), DiscoveryError> {
        let tunneled_services = self.tunneled_services();
        if let Some(subscriber) = &mut self.subscriber {
            fail!(
                from "Tunnel::discover_over_iceoryx",
                when subscriber.discover(&mut |static_config| {
                    on_discovery(static_config, &mut self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Subscriber"
            );
        }
        if let Some(tracker) = &mut self.tracker {
            fail!(
                from "Tunnel::discover_over_iceoryx",
                when tracker.discover(&mut |static_config| {
                    on_discovery(static_config, &mut self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Tracker"
            );
        }

        Ok(())
    }

    pub fn discover_over_backend(&mut self) -> Result<(), DiscoveryError> {
        let tunneled_services = self.tunneled_services();
        fail!(
            from "Tunnel::discover_over_backend",
            when self.backend.discovery().discover(&mut |static_config| {
                on_discovery(static_config, &self.node, &self.backend, &tunneled_services, &mut self.ports, &mut self.relays).unwrap();
                Ok(())
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
                    warn!(from "Tunnel::propagate", "No relay available for id {:?}. Skipping.", service_id);
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
                    warn!(from "Tunnel::propagate", "No relay available for id {:?}. Skipping.", service_id);
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
) -> Result<(), CreationError> {
    if services.contains(static_config.service_id()) {
        // Nothing to do.
        return Ok(());
    }

    match static_config.messaging_pattern() {
        MessagingPattern::PublishSubscribe(_) => {
            debug!(
                from "Tunnel::on_discovery",
                "Discovered PublishSubscribe({})",
                static_config.name()
            );
            setup_publish_subscribe(static_config, node, backend, ports, relays)
        }
        MessagingPattern::Event(_) => {
            debug!(
                from "Tunnel::on_discovery",
                "Discovered Event({})", static_config.name());
            setup_event(static_config, node, backend, ports, relays)
        }
        _ => {
            // Not supported. Nothing to do.
            debug!(
                from "Tunnel::on_discovery",
                "Unsupported Discovery: {}({})",
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
) -> Result<(), CreationError> {
    let service_id = static_config.service_id();

    // TODO: Use fail!
    let port = PublishSubscribePorts::new(static_config, &node).unwrap();

    // TODO: Use fail!
    let relay = backend
        .relay_builder()
        .publish_subscribe(static_config)
        .create()
        .unwrap();

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
) -> Result<(), CreationError> {
    let service_id = static_config.service_id();

    // TODO: Use fail!
    let port = EventPorts::new(static_config, &node).unwrap();

    // TODO: Use fail!
    let relay = backend
        .relay_builder()
        .event(static_config)
        .create()
        .unwrap();

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
            // TODO: Handle error properly
            relay.send(sample).unwrap();
        }),
        "Failed to receive and propagate samples"
    );
    fail!(
        from "propagate_publish_subscribe_payloads",
        when port.send(|loan: &mut LoanFn<_>| {
            // TODO: Handle error properly
            relay.receive(&mut |size| loan(size)).unwrap()
        }),
        "Failed to send ingested samples"
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
            // TODO: Handle error properly
            relay.send(id).unwrap();
        }),
        "Failed to wait for events"
    );

    fail!(
        from "propagate_events",
        when relay.receive(&mut |id| {
            port.send(id).unwrap();
        }).map_err(|_| PropagateError::Error),
        "Failed to receive events from relay"
    );

    Ok(())
}
