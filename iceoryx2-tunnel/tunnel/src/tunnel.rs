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

use crate::ports::Ports;
use crate::{discovery, ports};
use iceoryx2::node::{Node, NodeBuilder, NodeCreationFailure};
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::Service;
use iceoryx2_bb_log::{debug, fail, warn};
use iceoryx2_tunnel_backend::traits::{
    Backend, Discovery, PublishSubscribeRelay, RelayBuilder, RelayFactory,
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
pub enum RelayError {
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

impl From<ports::publish_subscribe::ReceiveError> for RelayError {
    fn from(_: ports::publish_subscribe::ReceiveError) -> Self {
        RelayError::Error
    }
}

impl From<ports::publish_subscribe::SendError> for RelayError {
    fn from(_: ports::publish_subscribe::SendError) -> Self {
        RelayError::Error
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

/// Struct to store different relay types by service id
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
    services: HashSet<ServiceId>,
    ports: HashMap<ServiceId, Ports<S>>, // TODO: Organize by port type
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
            services: HashSet::new(),
            ports: HashMap::new(),
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
        if let Some(subscriber) = &mut self.subscriber {
            fail!(
                from "Tunnel::discover_over_iceoryx",
                when subscriber.discover(&mut |static_config| {
                    on_discovery(static_config, &mut self.node, &self.backend, &mut self.services, &mut self.ports, &mut self.relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Subscriber"
            );
        }
        if let Some(tracker) = &mut self.tracker {
            fail!(
                from "Tunnel::discover_over_iceoryx",
                when tracker.discover(&mut |static_config| {
                    on_discovery(static_config, &mut self.node, &self.backend, &mut self.services, &mut self.ports, &mut self.relays).unwrap();
                    Ok(())
                }),
                "Failed to discover services via Tracker"
            );
        }

        Ok(())
    }

    pub fn discover_over_backend(&mut self) -> Result<(), DiscoveryError> {
        fail!(
            from "Tunnel::discover_over_backend",
            when self.backend.discovery().discover(&mut |static_config| {
                on_discovery(static_config, &self.node, &self.backend, &mut self.services, &mut self.ports, &mut self.relays).unwrap();
                Ok(())
            }),
            with DiscoveryError::FailedToDiscoverOverBackend,
            "Failed to discover services via Backend"
        );
        Ok(())
    }

    /// TODO: Consider the ordering ...
    pub fn relay(&mut self) -> Result<(), RelayError> {
        for (id, ports) in &self.ports {
            match ports {
                Ports::PublishSubscribe(port) => {
                    let relay = match self.relays.publish_subscribe.get(id) {
                        Some(relay) => relay,
                        None => {
                            warn!(
                                from "Tunnel::relay",
                                "No relay available for id {:?}. Skipping.", id);
                            return Ok(());
                        }
                    };

                    fail!(
                        from "Tunnel::relay",
                        when port.receive(self.node.id(), |sample| {
                            // TODO: Handle error properly
                            relay.propagate(sample).unwrap();
                        }),
                        "Failed to receive and propagate samples"
                    );

                    fail!(
                        from "Tunnel::relay",

                        when port.send(|loan: &mut LoanFn<_>| {
                            // TODO: Handle error properly
                            relay.ingest(&mut |size| loan(size)).unwrap()
                        }),
                        "Failed to send ingested samples"
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

fn on_discovery<S: Service, B: Backend<S>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    services: &mut HashSet<ServiceId>,
    ports: &mut HashMap<ServiceId, Ports<S>>,
    relays: &mut Relays<S, B>,
) -> Result<(), CreationError> {
    match static_config.messaging_pattern() {
        MessagingPattern::PublishSubscribe(_) => {
            debug!(
                from "Tunnel::on_discovery",
                "Discovered PublishSubscribe({})",
                static_config.name()
            );
            setup_publish_subscribe(static_config, node, backend, services, ports, relays)
        }
        MessagingPattern::Event(_) => {
            debug!(
                from "Tunnel::on_discovery",
                "Discovered Event({})", static_config.name());
            setup_event(static_config, node, backend, services, ports, relays)
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
    services: &mut HashSet<ServiceId>,
    ports: &mut HashMap<ServiceId, Ports<S>>,
    relays: &mut Relays<S, B>,
) -> Result<(), CreationError> {
    let service_id = static_config.service_id();
    if services.contains(service_id) {
        return Ok(());
    }

    // TODO: Use fail!
    let port = ports::publish_subscribe::Ports::new(static_config, &node).unwrap();

    // TODO: Use fail!
    let relay = backend
        .relay_builder()
        .publish_subscribe(static_config)
        .create()
        .unwrap();

    services.insert(service_id.clone());
    ports.insert(service_id.clone(), Ports::PublishSubscribe(port));
    relays.publish_subscribe.insert(service_id.clone(), relay);

    Ok(())
}

fn setup_event<S: Service, B: Backend<S>>(
    static_config: &StaticConfig,
    node: &Node<S>,
    backend: &B,
    services: &mut HashSet<ServiceId>,
    ports: &mut HashMap<ServiceId, Ports<S>>,
    relays: &mut Relays<S, B>,
) -> Result<(), CreationError> {
    let service_id = static_config.service_id();
    if services.contains(service_id) {
        return Ok(());
    }

    // TODO: Use fail!
    let port = ports::event::Ports::new(static_config, &node).unwrap();

    services.insert(service_id.clone());
    ports.insert(service_id.clone(), Ports::Event(port));

    Ok(())
}
