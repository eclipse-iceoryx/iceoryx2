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

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::{Node, NodeState, NodeView};
use iceoryx2::service::Service;
use iceoryx2::service::ServiceDetails;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2_log::{fail, info};
use iceoryx2_services_common::{DiscoveryEvent, DiscoveryEventRef};
use iceoryx2_services_tunnel_backend::traits::{Backend, Discovery};

use crate::bridge::Bridge;
use crate::discovery::LocalDiscoveryStrategy;
use crate::discovery::state::DiscoveryState;
use crate::discovery::state::Origin;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Node,
    ServiceName,
    Backend,
    DiscoverySubscriber,
    ReactiveMode,
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
    UnsupportedMessagingPattern,
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

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub discovery_service: Option<String>,
    pub services: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct Tunnel<S: Service, B: for<'a> Backend<S> + Debug> {
    node: Node<S>,
    backend: B,
    discovery_state: DiscoveryState,
    bridges: BTreeMap<ServiceHash, Bridge<S, B>>,
    local_discovery: LocalDiscoveryStrategy<S>,
    services_filter: Option<BTreeSet<String>>,
}

impl<S: Service, B: for<'a> Backend<S> + Debug> Tunnel<S, B> {
    /// Returns a builder for configuring and constructing a [`Tunnel`].
    #[allow(clippy::new_ret_no_self)] // entry point to the type-state builder
    pub fn new() -> crate::builder::TunnelBuilder<S, B, crate::builder::Unconfigured> {
        crate::builder::TunnelBuilder::new()
    }

    /// Returns the iceoryx2 [`Node`] hosting the tunnel's services.
    pub fn node(&self) -> &Node<S> {
        &self.node
    }

    /// Wires pre-built parts into a [`Tunnel`]. All creation logic lives in
    /// [`crate::builder::TunnelBuilder`].
    pub(crate) fn create(
        node: Node<S>,
        backend: B,
        local_discovery: LocalDiscoveryStrategy<S>,
        services_filter: Option<BTreeSet<String>>,
    ) -> Self {
        Self {
            node,
            backend,
            discovery_state: DiscoveryState::default(),
            bridges: BTreeMap::new(),
            local_discovery,
            services_filter,
        }
    }

    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        self.discover_local()?;
        self.discover_remote()?;
        self.reconcile()
    }

    pub fn discover_over_iceoryx(&mut self) -> Result<(), DiscoveryError> {
        self.discover_local()?;
        self.reconcile()
    }

    pub fn discover_over_backend(&mut self) -> Result<(), DiscoveryError> {
        self.discover_remote()?;
        self.reconcile()
    }

    pub fn propagate(&mut self) -> Result<(), PropagateError> {
        // Ensure open bridges are in sync with discovery state. Debug only.
        debug_assert!(
            self.bridges.len() == self.discovery_state.snapshot().iter().count()
                && self
                    .discovery_state
                    .snapshot()
                    .iter()
                    .all(|(hash, _)| self.bridges.contains_key(hash)),
            "bridges out of sync with desired services — a discovery path skipped reconcile"
        );

        for bridge in self.bridges.values() {
            bridge.propagate(self.node.id())?;
        }

        Ok(())
    }

    pub fn tunneled_services(&self) -> BTreeSet<ServiceHash> {
        // Ensure open bridges are in sync with discovery state. Debug only.
        debug_assert!(
            self.bridges.len() == self.discovery_state.snapshot().iter().count()
                && self
                    .discovery_state
                    .snapshot()
                    .iter()
                    .all(|(hash, _)| self.bridges.contains_key(hash)),
            "bridges out of sync with desired services — a discovery path skipped reconcile"
        );

        self.bridges.keys().cloned().collect()
    }

    /// Updates the locally offerred services..
    fn discover_local(&mut self) -> Result<(), DiscoveryError> {
        match &self.local_discovery {
            LocalDiscoveryStrategy::Subscriber(_) => self.discover_over_subscriber(),
            LocalDiscoveryStrategy::Tracker(_) => self.discover_over_tracker(),
        }
    }

    /// Updates the remotely offerred services.
    fn discover_remote(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Tunnel({})::discover_remote", self.node.id());

        let node = &self.node;
        let backend = &self.backend;
        let services_filter = &self.services_filter;
        let discovery_state = &mut self.discovery_state;

        fail!(
            from origin,
            when backend.discovery().discover(|event| {
                apply_discovery::<S, B>(
                    node,
                    backend,
                    discovery_state,
                    services_filter,
                    event,
                    Origin::Remote,
                )
            }),
            with DiscoveryError::DiscoveryOverBackend,
            "Failed to discover services via Backend"
        );
        Ok(())
    }

    /// Subscriber-mode local discovery: events from the discovery service
    /// describe local additions and removals.
    fn discover_over_subscriber(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Tunnel({})::discover_over_subscriber", self.node.id());

        let LocalDiscoveryStrategy::Subscriber(subscriber) = &self.local_discovery else {
            panic!("Should never happen. Discovery strategy enforced in discover().");
        };

        let node = &self.node;
        let backend = &self.backend;
        let services_filter = &self.services_filter;
        let discovery_state = &mut self.discovery_state;

        fail!(
            from origin,
            when subscriber.discover(|event| {
                apply_discovery::<S, B>(
                    node,
                    backend,
                    discovery_state,
                    services_filter,
                    event,
                    Origin::Local,
                )
            }),
            with DiscoveryError::DiscoveryOverService,
            "Failed to discover services via subscriber to discovery service"
        );
        Ok(())
    }

    /// Tracker-mode local discovery: refresh the local registry snapshot, then
    /// bring the locally-offered set in line with it. A service is considered
    /// locally offered when at least one non-tunnel, non-dead node offers it.
    fn discover_over_tracker(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Tunnel({})::discover_over_tracker", self.node.id());

        let LocalDiscoveryStrategy::Tracker(tracker) = &mut self.local_discovery else {
            panic!("Should never happen. Discovery strategy enforced in discover().");
        };

        // Refresh the tracker's view of the system.
        fail!(
            from origin,
            when tracker.sync(),
            with DiscoveryError::DiscoveryOverTracker,
            "Failed to refresh discovery tracker"
        );

        let node = &self.node;
        let backend = &self.backend;
        let services_filter = &self.services_filter;
        let discovery_state = &mut self.discovery_state;

        // Reconcile the locally-offered set against the tracker snapshot:
        // services offered by some non-tunnel node and admitted by the filters.
        discovery_state.reconcile_update(
            Origin::Local,
            tracker
                .iter()
                .filter(|details| {
                    is_locally_offered(details, node.id())
                        && allowed(&details.static_details, services_filter)
                })
                .map(|details| &details.static_details),
            |config| announce_added::<S, B>(node, backend, config),
            |config| announce_removed::<S, B>(node, backend, config),
        )
    }

    /// Reconciles the opened bridges with the discovery snapshot.
    fn reconcile(&mut self) -> Result<(), DiscoveryError> {
        let log_origin = format!("Tunnel({})::reconcile", self.node.id());

        // Close bridges no longer offered by any side.
        let snapshot = self.discovery_state.snapshot();
        let stale: Vec<ServiceHash> = self
            .bridges
            .keys()
            .filter(|hash| !snapshot.contains(hash))
            .cloned()
            .collect();
        for hash in stale {
            info!(from log_origin, "Closing bridge: {}", hash.as_str());
            self.bridges.remove(&hash);
        }

        // Open bridges for newly-offered services.
        let missing: Vec<StaticConfig> = self
            .discovery_state
            .snapshot()
            .iter()
            .filter(|&(hash, _)| !self.bridges.contains_key(hash))
            .map(|(_, static_config)| static_config.clone())
            .collect();
        for static_config in missing {
            let hash = *static_config.service_hash();
            info!(
                from log_origin,
                "Opening bridge: {}({})",
                static_config.messaging_pattern(),
                static_config.name()
            );
            let bridge = Bridge::open(&self.node, &self.backend, &static_config)?;
            self.bridges.insert(hash, bridge);
        }

        Ok(())
    }
}

/// Applies a discovery event from `origin` to the offering snapshots and
/// announces local-side 0 → 1 and 1 → 0 transitions.
fn apply_discovery<S: Service, B: Backend<S>>(
    node: &Node<S>,
    backend: &B,
    discovery_state: &mut DiscoveryState,
    services_filter: &Option<BTreeSet<String>>,
    event: DiscoveryEvent,
    origin: Origin,
) -> Result<(), DiscoveryError> {
    match event {
        DiscoveryEvent::Added(static_config) => {
            if !allowed(&static_config, services_filter) {
                return Ok(());
            }

            // Announce on local 0 → 1 transitions, then record this side as
            // offering. Announcing first keeps `static_config` available for the
            // announcement before it is moved into the snapshot.
            let hash = *static_config.service_hash();
            let offered_locally =
                origin == Origin::Local && discovery_state.is_offered_by(origin, &hash);
            if !offered_locally {
                announce_added::<S, B>(node, backend, &static_config)?;
            }
            discovery_state
                .delta_update(origin)
                .set_offered(static_config);
        }
        DiscoveryEvent::Removed(hash) => {
            // Mark this side as not offered; announce on local 1 → 0 transitions.
            let removed_config = discovery_state.delta_update(origin).set_not_offered(&hash);
            if origin == Origin::Local {
                if let Some(static_config) = &removed_config {
                    announce_removed::<S, B>(node, backend, static_config)?;
                }
            }
        }
    }

    Ok(())
}

/// Whether the tunnel should offer `static_config`: a supported messaging
/// pattern (publish-subscribe or event) that passes the optional services
/// allowlist.
fn allowed(static_config: &StaticConfig, services_filter: &Option<BTreeSet<String>>) -> bool {
    let supported_pattern = matches!(
        static_config.messaging_pattern(),
        MessagingPattern::PublishSubscribe(_) | MessagingPattern::Event(_)
    );
    let in_allowlist = match services_filter {
        Some(allowlist) => allowlist.contains(static_config.name().as_str()),
        None => true,
    };
    supported_pattern && in_allowlist
}

/// Broadcasts a service's availability to remote peers over the backend.
fn announce_added<S: Service, B: Backend<S>>(
    node: &Node<S>,
    backend: &B,
    static_config: &StaticConfig,
) -> Result<(), DiscoveryError> {
    let origin = format!("Tunnel({})::announce_added", node.id());

    info!(
        from origin,
        "Announcing addition: {}({})",
        static_config.messaging_pattern(),
        static_config.name()
    );
    fail!(
        from origin,
        when backend.discovery().announce(DiscoveryEventRef::Added(static_config)),
        with DiscoveryError::DiscoveryAnnouncement,
        "Failed to announce service over backend"
    );
    Ok(())
}

/// Withdraws a previously-announced service from remote peers over the backend.
fn announce_removed<S: Service, B: Backend<S>>(
    node: &Node<S>,
    backend: &B,
    static_config: &StaticConfig,
) -> Result<(), DiscoveryError> {
    let origin = format!("Tunnel({})::announce_removed", node.id());
    info!(
        from origin,
        "Announcing removal: {}({})",
        static_config.messaging_pattern(),
        static_config.name()
    );
    fail!(
        from origin,
        when backend.discovery().announce(DiscoveryEventRef::Removed(static_config.service_hash())),
        with DiscoveryError::DiscoveryAnnouncement,
        "Failed to announce service removal over backend"
    );
    Ok(())
}

/// Whether `details` is offered by at least one live node other than the tunnel
/// itself (`tunnel_node`). The tunnel's own mirror ports keep a service alive in
/// the registry, so they must be excluded when deciding if a service is still
/// locally offered.
fn is_locally_offered<S: Service>(details: &ServiceDetails<S>, tunnel_node: &UniqueNodeId) -> bool {
    details.dynamic_details.as_ref().is_some_and(|d| {
        d.nodes.iter().any(|node| match node {
            NodeState::Alive(view) => view.id() != tunnel_node,
            NodeState::Inaccessible(id) | NodeState::Undefined(id) => id != tunnel_node,
            NodeState::Dead(_) => false,
        })
    })
}
