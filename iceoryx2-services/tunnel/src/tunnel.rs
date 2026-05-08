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

use core::convert::Infallible;
use core::fmt::Debug;

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::{Node, NodeBuilder, NodeState, NodeView};
use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2_log::{debug, fail, info, trace, warn};
use iceoryx2_services_common::DiscoveryEvent;
use iceoryx2_services_tunnel_backend::traits::{
    Backend, Discovery, EventRelay, PublishSubscribeRelay, RelayBuilder, RelayFactory,
};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::LoanFn;

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
    pub(crate) publish_subscribe: BTreeMap<ServiceHash, PublishSubscribePorts<S>>,
    pub(crate) event: BTreeMap<ServiceHash, EventPorts<S>>,
}

impl<S: Service> Ports<S> {
    pub fn new() -> Self {
        Self {
            publish_subscribe: BTreeMap::new(),
            event: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Relays<S: Service, B: Backend<S>> {
    publish_subscribe: BTreeMap<ServiceHash, B::PublishSubscribeRelay>,
    event: BTreeMap<ServiceHash, B::EventRelay>,
}

impl<S: Service, B: Backend<S>> Relays<S, B> {
    pub fn new() -> Self {
        Self {
            publish_subscribe: BTreeMap::new(),
            event: BTreeMap::new(),
        }
    }
}

/// Side of the system that a discovery event refers to.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Origin {
    /// Service was discovered (or its absence detected) on the local iceoryx system.
    Local,
    /// Service was discovered (or its absence detected) over the backend.
    Remote,
}

/// Outcome of marking a side as offering.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AddOutcome {
    /// Side just started offering.
    NewlyOffering,
    /// Side was already offering.
    AlreadyOffering,
}

/// Outcome of marking a side as no longer offering.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum RemoveOutcome {
    /// Side just stopped offering.
    NoLongerOffering,
    /// Side was already not offering.
    AlreadyNotOffering,
}

/// Per-service lifecycle state tracked by the tunnel.
///
/// Required to determine that services that only exist because the tunnel
/// is mirroring it rather than an active node offering the service.
#[derive(Debug)]
pub(crate) struct TrackedService {
    static_config: StaticConfig,
    locally_offered: bool,
    remotely_offered: bool,
}

impl TrackedService {
    fn is_offered(&self) -> bool {
        self.locally_offered || self.remotely_offered
    }

    /// Sets the service as offered from `origin`.
    fn set_offered(&mut self, origin: Origin) -> AddOutcome {
        let flag = match origin {
            Origin::Local => &mut self.locally_offered,
            Origin::Remote => &mut self.remotely_offered,
        };
        let was = *flag;
        *flag = true;
        if was {
            AddOutcome::AlreadyOffering
        } else {
            AddOutcome::NewlyOffering
        }
    }

    /// Sets the service as no longer offered from `origin`.
    fn set_not_offered(&mut self, origin: Origin) -> RemoveOutcome {
        let flag = match origin {
            Origin::Local => &mut self.locally_offered,
            Origin::Remote => &mut self.remotely_offered,
        };
        let was = *flag;
        *flag = false;
        if was {
            RemoveOutcome::NoLongerOffering
        } else {
            RemoveOutcome::AlreadyNotOffering
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
    services: BTreeMap<ServiceHash, TrackedService>,
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
            services: BTreeMap::new(),
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
        if self.subscriber.is_some() {
            self.discover_over_subscriber()?;
        }
        if self.tracker.is_some() {
            self.discover_over_tracker()?;
        }
        Ok(())
    }

    pub fn discover_over_backend(&mut self) -> Result<(), DiscoveryError> {
        let mut events: Vec<DiscoveryEvent> = Vec::new();
        fail!(
            from "Tunnel::discover_over_backend",
            when self.backend.discovery().discover(|event| -> Result<(), Infallible> {
                events.push(event.clone());
                Ok(())
            }),
            with DiscoveryError::DiscoveryOverBackend,
            "Failed to discover services via Backend"
        );

        for event in events {
            self.process_discovery(event, Origin::Remote)?;
        }
        Ok(())
    }

    pub fn propagate(&mut self) -> Result<(), PropagateError> {
        for (service_hash, port) in &self.ports.publish_subscribe {
            match self.relays.publish_subscribe.get(service_hash) {
                Some(relay) => {
                    propagate_publish_subscribe_payloads::<S, B>(self.node.id(), port, relay)?;
                }
                None => {
                    warn!(from "Tunnel::propagate", "No relay available for {:?}", service_hash);
                    continue;
                }
            };
        }

        for (service_hash, port) in &self.ports.event {
            match self.relays.event.get(service_hash) {
                Some(relay) => {
                    propagate_events::<S, B>(port, relay)?;
                }
                None => {
                    warn!(from "Tunnel::propagate", "No relay available for {:?}", service_hash);
                    continue;
                }
            };
        }

        Ok(())
    }

    pub fn tunneled_services(&self) -> BTreeSet<ServiceHash> {
        self.services.keys().cloned().collect()
    }

    /// Subscriber-mode local discovery: events from the discovery service
    /// describe local additions and removals.
    fn discover_over_subscriber(&mut self) -> Result<(), DiscoveryError> {
        let mut events: Vec<DiscoveryEvent> = Vec::new();
        let subscriber = self
            .subscriber
            .as_mut()
            .expect("Should never happen. Subscriber created in constructor.");
        fail!(
            from "Tunnel::discover_over_subscriber",
            when subscriber.discover(|event| -> Result<(), Infallible> {
                events.push(event.clone());
                Ok(())
            }),
            with DiscoveryError::DiscoveryOverService,
            "Failed to discover services via subscriber to discovery service"
        );

        for event in events {
            self.process_discovery(event, Origin::Local)?;
        }
        Ok(())
    }

    /// Tracker-mode local discovery: sync against the local registry, then
    /// derive `locally_offered` transitions from the snapshot. A service is
    /// considered locally offered when at least one non-tunnel, non-dead node
    /// is offering it.
    fn discover_over_tracker(&mut self) -> Result<(), DiscoveryError> {
        let mut events: Vec<DiscoveryEvent> = Vec::new();
        {
            let tracker = self
                .tracker
                .as_ref()
                .expect("Should never happen. Tracker created in constructor.");

            let sync = fail!(
                from "Tunnel::discover_over_tracker",
                when tracker.sync(),
                with DiscoveryError::DiscoveryOverTracker,
                "Failed to synchronize discovery tracker"
            );

            // A service is considered locally offered when at least one
            // non-tunnel, non-dead node is currently registered for it.
            // Inaccessible/Undefined nodes are counted to give them the
            // benefit of the doubt.
            let tunnel_node_id = *self.node.id();
            let is_offered_by_other = |hash: &ServiceHash| -> bool {
                tracker.get(hash, |service_details| {
                    service_details
                        .and_then(|d| d.dynamic_details.as_ref())
                        .map(|d| {
                            d.nodes.iter().any(|node| match node {
                                NodeState::Alive(view) => view.id() != &tunnel_node_id,
                                NodeState::Inaccessible(id) | NodeState::Undefined(id) => {
                                    id != &tunnel_node_id
                                }
                                NodeState::Dead(_) => false,
                            })
                        })
                        .unwrap_or(false)
                })
            };

            // Newly-listed services with at least one non-tunnel offerer.
            for static_config in sync.added {
                if is_offered_by_other(static_config.service_hash()) {
                    events.push(DiscoveryEvent::Added(static_config));
                }
            }

            // Services that disappeared from the registry entirely.
            for hash in sync.removed {
                events.push(DiscoveryEvent::Removed(hash));
            }

            // Currently-tracked services whose local offerer has gone away.
            for (hash, state) in &self.services {
                if state.locally_offered && !is_offered_by_other(hash) {
                    events.push(DiscoveryEvent::Removed(*hash));
                }
            }
        }

        for event in events {
            self.process_discovery(event, Origin::Local)?;
        }
        Ok(())
    }

    /// Processes a discovery event from `origin`, advancing per-service state.
    ///
    /// Opens the mirror on first observation, announces over the backend on
    /// local-side 0 → 1 and 1 → 0 transitions, and tears the mirror down once
    /// neither side is offering.
    fn process_discovery(
        &mut self,
        event: DiscoveryEvent,
        origin: Origin,
    ) -> Result<(), DiscoveryError> {
        match event {
            DiscoveryEvent::Added(static_config) => {
                if !matches!(
                    static_config.messaging_pattern(),
                    MessagingPattern::PublishSubscribe(_) | MessagingPattern::Event(_)
                ) {
                    debug!(
                        from "Tunnel::process_discovery",
                        "Skipping unsupported messaging pattern: {}({})",
                        static_config.messaging_pattern(),
                        static_config.name()
                    );
                    return Ok(());
                }

                let hash = *static_config.service_hash();

                // If the service was not already tracked, mirror the service
                if !self.services.contains_key(&hash) {
                    info!(
                        from "Tunnel::process_discovery",
                        "Opening mirror ({:?}): {}({})",
                        origin,
                        static_config.messaging_pattern(),
                        static_config.name()
                    );

                    self.open_ports(&static_config)?;
                    self.open_relay(&static_config)?;
                    self.services.insert(
                        hash,
                        TrackedService {
                            static_config,
                            locally_offered: false,
                            remotely_offered: false,
                        },
                    );
                }

                // Mark this side as offered; announce on local 0 → 1 transitions.
                let outcome = self
                    .services
                    .get_mut(&hash)
                    .expect("Should never happen. Entry was just inserted.")
                    .set_offered(origin);

                if origin == Origin::Local && outcome == AddOutcome::NewlyOffering {
                    let tracked_service = self
                        .services
                        .get(&hash)
                        .expect("Should never happen. Entry was confirmed above.");
                    self.announce_added(&tracked_service.static_config)?;
                }
            }
            DiscoveryEvent::Removed(hash) => {
                let Some(tracked_service) = self.services.get_mut(&hash) else {
                    debug!(
                        from "Tunnel::process_discovery",
                        "Ignoring Removed for untracked service: {}",
                        hash.as_str()
                    );
                    return Ok(());
                };

                // Mark this side as not offered; announce on local 1 → 0 transitions.
                let outcome = tracked_service.set_not_offered(origin);
                let last_offerer_gone = !tracked_service.is_offered();

                if origin == Origin::Local && outcome == RemoveOutcome::NoLongerOffering {
                    let tracked_service = self
                        .services
                        .get(&hash)
                        .expect("Should never happen. Entry was confirmed above.");
                    self.announce_removed(&tracked_service.static_config)?;
                }

                // If no side is offering anymore, close the mirror.
                if last_offerer_gone {
                    let removed_service = self
                        .services
                        .remove(&hash)
                        .expect("Should never happen. Entry was confirmed above.");
                    info!(
                        from "Tunnel::process_discovery",
                        "Closing mirror ({:?}): {}({})",
                        origin,
                        removed_service.static_config.messaging_pattern(),
                        removed_service.static_config.name()
                    );
                    self.close_ports(&hash);
                    self.close_relay(&hash);

                    // Drop the tracker's cached snapshot so a same-hash service
                    // recreated by a another node reappears as a fresh `added` on the
                    // next sync.
                    if let Some(tracker) = &self.tracker {
                        tracker.forget(&hash);
                    }
                }
            }
        }

        Ok(())
    }

    /// Creates the mirror ports for a service.
    fn open_ports(&mut self, static_config: &StaticConfig) -> Result<(), DiscoveryError> {
        let hash = *static_config.service_hash();
        match static_config.messaging_pattern() {
            MessagingPattern::PublishSubscribe(_) => {
                let port = fail!(
                    from "Tunnel::open_ports",
                    when PublishSubscribePorts::new(static_config, &self.node),
                    with DiscoveryError::PublishSubscribePortCreation,
                    "Failed to create publish-subscribe ports"
                );
                self.ports.publish_subscribe.insert(hash, port);
            }
            MessagingPattern::Event(_) => {
                let port = fail!(
                    from "Tunnel::open_ports",
                    when EventPorts::new(static_config, &self.node),
                    with DiscoveryError::EventPortsCreation,
                    "Failed to create event ports"
                );
                self.ports.event.insert(hash, port);
            }
            _ => {}
        }
        Ok(())
    }

    /// Creates the backend relay for a service.
    fn open_relay(&mut self, static_config: &StaticConfig) -> Result<(), DiscoveryError> {
        let hash = *static_config.service_hash();
        match static_config.messaging_pattern() {
            MessagingPattern::PublishSubscribe(_) => {
                let relay = fail!(
                    from "Tunnel::open_relay",
                    when self.backend
                        .relay_builder()
                        .publish_subscribe(static_config)
                        .create(),
                    with DiscoveryError::PublishSubscribeRelayCreation,
                    "Failed to create publish-subscribe relay"
                );
                self.relays.publish_subscribe.insert(hash, relay);
            }
            MessagingPattern::Event(_) => {
                let relay = fail!(
                    from "Tunnel::open_relay",
                    when self.backend
                        .relay_builder()
                        .event(static_config)
                        .create(),
                    with DiscoveryError::EventRelayCreation,
                    "Failed to create event relay"
                );
                self.relays.event.insert(hash, relay);
            }
            _ => {}
        }
        Ok(())
    }

    /// Drops a service's mirrored ports.
    fn close_ports(&mut self, hash: &ServiceHash) {
        self.ports.publish_subscribe.remove(hash);
        self.ports.event.remove(hash);
    }

    /// Drops a service's backend relay.
    fn close_relay(&mut self, hash: &ServiceHash) {
        self.relays.publish_subscribe.remove(hash);
        self.relays.event.remove(hash);
    }

    /// Broadcasts a service's availability to remote peers over the backend.
    fn announce_added(&self, static_config: &StaticConfig) -> Result<(), DiscoveryError> {
        info!(
            from "Tunnel::announce_added",
            "Announcing addition: {}({})",
            static_config.messaging_pattern(),
            static_config.name()
        );
        fail!(
            from "Tunnel::announce_added",
            when self.backend.discovery().announce(&DiscoveryEvent::Added(static_config.clone())),
            with DiscoveryError::DiscoveryAnnouncement,
            "Failed to announce service over backend"
        );
        Ok(())
    }

    /// Withdraws a previously-announced service from remote peers over the backend.
    fn announce_removed(&self, static_config: &StaticConfig) -> Result<(), DiscoveryError> {
        info!(
            from "Tunnel::announce_removed",
            "Announcing removal: {}({})",
            static_config.messaging_pattern(),
            static_config.name()
        );
        fail!(
            from "Tunnel::announce_removed",
            when self.backend.discovery().announce(&DiscoveryEvent::Removed(*static_config.service_hash())),
            with DiscoveryError::DiscoveryAnnouncement,
            "Failed to announce service removal over backend"
        );
        Ok(())
    }
}

fn propagate_publish_subscribe_payloads<S: Service, B: Backend<S> + Debug>(
    node_id: &UniqueNodeId,
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
