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
use crate::discovery::state::DeltaUpdate;
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
    discovery_strategy: LocalDiscoveryStrategy<S>,
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
        discovery_strategy: LocalDiscoveryStrategy<S>,
        services_filter: Option<BTreeSet<String>>,
    ) -> Self {
        Self {
            node,
            backend,
            discovery_state: DiscoveryState::default(),
            bridges: BTreeMap::new(),
            discovery_strategy,
            services_filter,
        }
    }

    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        self.iceoryx_discovery()?;
        self.backend_discovery()?;
        self.reconcile()
    }

    pub fn discover_over_iceoryx(&mut self) -> Result<(), DiscoveryError> {
        self.iceoryx_discovery()?;
        self.reconcile()
    }

    pub fn discover_over_backend(&mut self) -> Result<(), DiscoveryError> {
        self.backend_discovery()?;
        self.reconcile()
    }

    pub fn propagate(&mut self) -> Result<(), PropagateError> {
        self.debug_assert_synchronized();

        // Propagate publish-subscribe payloads before events
        // TODO(#1103): Retain ordering across the wire
        for bridge in self.bridges.values() {
            if matches!(bridge, Bridge::PublishSubscribe { .. }) {
                bridge.propagate(self.node.id())?;
            }
        }
        for bridge in self.bridges.values() {
            if matches!(bridge, Bridge::Event { .. }) {
                bridge.propagate(self.node.id())?;
            }
        }

        Ok(())
    }

    pub fn tunneled_services(&self) -> BTreeSet<ServiceHash> {
        self.debug_assert_synchronized();
        self.bridges.keys().cloned().collect()
    }

    /// Updates the locally offerred services.
    fn iceoryx_discovery(&mut self) -> Result<(), DiscoveryError> {
        match &self.discovery_strategy {
            LocalDiscoveryStrategy::Subscriber(_) => self.subscriber_discovery(),
            LocalDiscoveryStrategy::Tracker(_) => self.tracker_discovery(),
        }
    }

    /// Updates the remotely offerred services.
    fn backend_discovery(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Tunnel({})::backend_discovery", self.node.id());

        let node = &self.node;
        let backend = &self.backend;
        let services_filter = &self.services_filter;
        let mut update = self.discovery_state.delta_update(Origin::Remote);

        fail!(
            from origin,
            when backend.discovery().discover(|event| {
                on_discovery_event::<S, B>(node, backend, &mut update, services_filter, event)
            }),
            with DiscoveryError::DiscoveryOverBackend,
            "Failed to discover services via Backend"
        );
        Ok(())
    }

    /// Subscriber-mode local discovery: events from the discovery service
    /// describe local additions and removals.
    fn subscriber_discovery(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Tunnel({})::subscriber_discovery", self.node.id());

        let LocalDiscoveryStrategy::Subscriber(subscriber) = &self.discovery_strategy else {
            panic!("Should never happen. Discovery strategy enforced in discover().");
        };

        let node = &self.node;
        let backend = &self.backend;
        let services_filter = &self.services_filter;
        let mut update = self.discovery_state.delta_update(Origin::Local);

        fail!(
            from origin,
            when subscriber.discover(|event| {
                on_discovery_event::<S, B>(node, backend, &mut update, services_filter, event)
            }),
            with DiscoveryError::DiscoveryOverService,
            "Failed to discover services via subscriber to discovery service"
        );
        Ok(())
    }

    /// Tracker-mode local discovery: refresh the local registry snapshot, then
    /// bring the locally-offered set in line with it. A service is considered
    /// locally offered when at least one non-tunnel, non-dead node offers it.
    fn tracker_discovery(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Tunnel({})::tracker_discovery", self.node.id());

        let LocalDiscoveryStrategy::Tracker(tracker) = &mut self.discovery_strategy else {
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

        // Force the discovery local state to match the tracker snapshot.
        discovery_state.force_update(
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

    /// Reconciles the opened bridges with a snapshot of the discovery state.
    fn reconcile(&mut self) -> Result<(), DiscoveryError> {
        let log_origin = format!("Tunnel({})::reconcile", self.node.id());

        let snapshot = self.discovery_state.snapshot();

        // Close bridges no longer offered by any side.
        self.bridges.retain(|hash, _| {
            let keep = snapshot.contains(hash);
            if !keep {
                info!(from log_origin, "Closing bridge: {}", hash.as_str());
            }
            keep
        });

        // Open bridges for newly-offered services.
        for (hash, static_config) in snapshot.iter() {
            if self.bridges.contains_key(hash) {
                continue;
            }
            info!(
                from log_origin,
                "Opening bridge: {}({})",
                static_config.messaging_pattern(),
                static_config.name()
            );
            let bridge = Bridge::open(&self.node, &self.backend, static_config)?;
            self.bridges.insert(*hash, bridge);
        }

        Ok(())
    }

    /// Sanity check that the open bridges match the discovery
    /// state exactly. No-op in release builds.
    fn debug_assert_synchronized(&self) {
        #[cfg(debug_assertions)]
        {
            let snapshot = self.discovery_state.snapshot();
            let same_count = self.bridges.len() == snapshot.iter().count();
            let all_services_bridged = snapshot
                .iter()
                .all(|(hash, _)| self.bridges.contains_key(hash));

            debug_assert!(
                same_count && all_services_bridged,
                "bridges out of sync with discovery state"
            );
        }
    }
}

/// Updates the discovery state and announces local-side 0 → 1 and 1 → 0
/// transitions.
fn on_discovery_event<S: Service, B: Backend<S>>(
    node: &Node<S>,
    backend: &B,
    update: &mut DeltaUpdate<'_>,
    services_filter: &Option<BTreeSet<String>>,
    event: DiscoveryEvent,
) -> Result<(), DiscoveryError> {
    match event {
        DiscoveryEvent::Added(static_config) => {
            if !allowed(&static_config, services_filter) {
                return Ok(());
            }

            // Announce on local 0 → 1 transitions, then record as offered.
            let hash = *static_config.service_hash();
            let newly_offered_locally =
                update.origin() == Origin::Local && !update.is_offered(&hash);
            if newly_offered_locally {
                announce_added::<S, B>(node, backend, &static_config)?;
            }
            update.set_offered(static_config);
        }
        DiscoveryEvent::Removed(hash) => {
            // Announce on local 1 → 0 transitions.
            let removed_config = update.set_not_offered(&hash);
            if update.origin() == Origin::Local {
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
