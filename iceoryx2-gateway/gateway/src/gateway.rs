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

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::{Node, NodeState, NodeView};
use iceoryx2::service::Service;
use iceoryx2::service::ServiceDetails;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_gateway_backend::traits::{Backend, Discovery, Mapping};
use iceoryx2_gateway_backend::types::discovery::{Announcement, DiscoveryUpdate};
use iceoryx2_gateway_backend::types::identity::GatewayId;
use iceoryx2_gateway_backend::types::service_description::ServiceDescription;
use iceoryx2_log::{debug, fail, info};
use iceoryx2_services_discovery::service_discovery::DiscoveryEvent;

use crate::bridge::Bridges;
use crate::discovery::LocalDiscoveryStrategy;
use crate::discovery::state::DiscoveryState;

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
}

#[derive(Debug)]
pub struct Gateway<S: Service, B: Backend<S> + Debug> {
    node: Node<S>,
    backend: B,
    gateway_id: GatewayId,
    discovery_state: DiscoveryState,
    bridges: Bridges<S, B>,
    discovery_strategy: LocalDiscoveryStrategy<S>,
    announced: BTreeMap<ServiceHash, ServiceName>,
}

impl<S: Service, B: Backend<S> + Debug> Gateway<S, B> {
    /// Returns a builder for configuring and constructing a [`Gateway`].
    #[allow(clippy::new_ret_no_self)] // entry point to the type-state builder
    pub fn new() -> crate::builder::GatewayBuilder<S, B, crate::builder::Unconfigured> {
        crate::builder::GatewayBuilder::new()
    }

    /// Returns the iceoryx2 [`Node`] hosting the gateway's services.
    pub fn node(&self) -> &Node<S> {
        &self.node
    }

    /// Wires pre-built parts into a [`Gateway`]. All creation logic lives in
    /// [`crate::builder::GatewayBuilder`].
    pub(crate) fn create(
        node: Node<S>,
        backend: B,
        discovery_strategy: LocalDiscoveryStrategy<S>,
    ) -> Self {
        let gateway_id = GatewayId::new(*node.id(), backend.id());
        Self {
            node,
            backend,
            gateway_id,
            discovery_state: DiscoveryState::default(),
            bridges: Bridges::default(),
            discovery_strategy,
            announced: BTreeMap::new(),
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
        self.bridges.propagate(self.node.id())
    }

    pub fn bridged_services(&self) -> BTreeSet<ServiceHash> {
        self.debug_assert_synchronized();
        self.bridges.established()
    }

    /// Updates the locally offered services.
    fn iceoryx_discovery(&mut self) -> Result<(), DiscoveryError> {
        match &self.discovery_strategy {
            LocalDiscoveryStrategy::Subscriber(_) => self.subscriber_discovery(),
            LocalDiscoveryStrategy::Tracker(_) => self.tracker_discovery(),
        }
    }

    /// Updates the remotely offered services.
    fn backend_discovery(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Gateway({})::backend_discovery", self.node.id());

        let backend = &mut self.backend;
        let gateway_id = self.gateway_id;
        let mut update = self.discovery_state.remote_mut().delta_update();

        fail!(
            from origin,
            when backend.discovery().discover(gateway_id, |discovered| {
                match discovered {
                    DiscoveryUpdate::Added(gateway, description) => {
                        update.set_offered(gateway, &description);
                    }
                    DiscoveryUpdate::Removed(gateway, hash) => {
                        update.set_not_offered(&gateway, &hash);
                    }
                }
                Ok::<(), DiscoveryError>(())
            }),
            with DiscoveryError::DiscoveryOverBackend,
            "Failed to discover services via Backend"
        );
        Ok(())
    }

    /// Subscriber-mode local discovery: events from the discovery service
    /// describe local additions and removals.
    fn subscriber_discovery(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Gateway({})::subscriber_discovery", self.node.id());

        let LocalDiscoveryStrategy::Subscriber(subscriber) = &self.discovery_strategy else {
            panic!("Should never happen. Discovery strategy enforced in discover().");
        };

        let backend = &self.backend;
        let mut update = self.discovery_state.local_mut().delta_update();

        fail!(
            from origin,
            when subscriber.discover(|event| {
                match event {
                    DiscoveryEvent::Added(static_config) => {
                        let Ok(description) = ServiceDescription::try_from(static_config) else {
                            debug!(
                                from origin,
                                "Skipping service with unsupported messaging pattern: {}",
                                static_config.name()
                            );
                            return Ok(());
                        };
                        // Skip local services outside the mapping's scope.
                        if backend.mapping().remote(&description).is_some() {
                            update.set_offered(description);
                        }
                    }
                    DiscoveryEvent::Removed(hash) => {
                        update.set_not_offered(hash);
                    }
                }
                Ok::<(), DiscoveryError>(())
            }),
            with DiscoveryError::DiscoveryOverService,
            "Failed to discover services via subscriber to discovery service"
        );

        Ok(())
    }

    /// Tracker-mode local discovery: refresh the local registry snapshot, then
    /// bring the locally-offered set in line with it. A service is considered
    /// locally offered when at least one non-gateway, non-dead node offers it.
    fn tracker_discovery(&mut self) -> Result<(), DiscoveryError> {
        let origin = format!("Gateway({})::tracker_discovery", self.node.id());

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
        let mapping = backend.mapping();
        let discovery_state = &mut self.discovery_state;

        // Force the discovery local state to match the tracker snapshot.
        // Announcements are aligned in reconcile() once bridges are known.
        discovery_state.local_mut().force_update(
            tracker
                .iter()
                .filter(|details| is_locally_offered(details, node.id()))
                .filter_map(|details| ServiceDescription::try_from(&details.static_details).ok())
                .filter(|description| mapping.remote(description).is_some()),
        );

        Ok(())
    }

    /// Reconciles the bridges and announcements with the discovery state.
    fn reconcile(&mut self) -> Result<(), DiscoveryError> {
        // Removals announced first to minimize window for remote services
        // to communicate with a service no longer offered.
        let removals = self.announce_removals();
        self.reconcile_bridges();
        let additions = self.announce_additions();
        removals.and(additions)
    }

    /// Reconciles the opened bridges with a snapshot of the discovery state.
    fn reconcile_bridges(&mut self) {
        let log_origin = format!("Gateway({})::reconcile_bridges", self.node.id());

        let snapshot = self.discovery_state.snapshot();

        // Drop bridges to services no longer offered by any side.
        self.bridges.retain(
            |hash| snapshot.resolves(hash),
            |hash| info!(from log_origin, "Closing bridge: {}", hash.as_str()),
        );

        // Open bridges for newly-offered services.
        for (hash, description) in snapshot.resolved() {
            if self.bridges.contains(hash) {
                continue;
            }
            info!(
                from log_origin,
                "Opening bridge: {}({})",
                description.pattern,
                description.name
            );

            // Bridges that fail to be established are remembered so that subsequent
            // discovery calls do not waste time retrying. If however the service
            // disappears then reappears, a retry will occur.
            self.bridges.open(&self.node, &self.backend, description);
        }
    }

    /// Announces locally offered services whose bridge is established and
    /// that are not yet announced.
    fn announce_additions(&mut self) -> Result<(), DiscoveryError> {
        let node = &self.node;
        let backend = &mut self.backend;
        let gateway_id = self.gateway_id;
        let bridges = &self.bridges;
        let announced = &mut self.announced;

        let mut result = Ok(());
        for (hash, description) in self.discovery_state.local().iter() {
            if !bridges.is_established(hash) || announced.contains_key(hash) {
                continue;
            }
            if let Err(error) = announce_added::<S, B>(node, backend, gateway_id, description) {
                if result.is_ok() {
                    result = Err(error);
                }
                continue;
            }
            announced.insert(*hash, description.name);
        }
        result
    }

    /// Announces removal of services no longer locally offered.
    fn announce_removals(&mut self) -> Result<(), DiscoveryError> {
        let node = &self.node;
        let backend = &mut self.backend;
        let gateway_id = self.gateway_id;
        let discovery_state = &self.discovery_state;

        // Entries whose withdrawal fails are kept and retried on the next
        // discovery call.
        let mut result = Ok(());
        self.announced.retain(|hash, name| {
            if discovery_state.local().contains(hash) {
                return true;
            }
            if let Err(error) = announce_removed::<S, B>(node, backend, gateway_id, hash, name) {
                if result.is_ok() {
                    result = Err(error);
                }
                return true;
            }
            false
        });
        result
    }

    /// Sanity check that the tracked services match the discovery state
    /// exactly. No-op in release builds.
    fn debug_assert_synchronized(&self) {
        #[cfg(debug_assertions)]
        {
            let snapshot = self.discovery_state.snapshot();
            let same_count =
                self.bridges.number_of_tracked_services() == snapshot.resolved().count();
            let all_services_tracked = snapshot
                .resolved()
                .all(|(hash, _)| self.bridges.contains(hash));

            debug_assert!(
                same_count && all_services_tracked,
                "bridges out of sync with discovery state"
            );
        }
    }
}

/// Broadcasts a service's availability to remote peers over the backend.
fn announce_added<S: Service, B: Backend<S>>(
    node: &Node<S>,
    backend: &mut B,
    own_id: GatewayId,
    description: &ServiceDescription,
) -> Result<(), DiscoveryError> {
    let origin = format!("Gateway({})::announce_added", node.id());

    info!(
        from origin,
        "Announcing addition: {}({})",
        description.pattern,
        description.name
    );
    fail!(
        from origin,
        when backend.discovery().announce(own_id, Announcement::Added(description)),
        with DiscoveryError::DiscoveryAnnouncement,
        "Failed to announce service over backend"
    );
    Ok(())
}

/// Announces a service's removal to remote peers over the backend.
fn announce_removed<S: Service, B: Backend<S>>(
    node: &Node<S>,
    backend: &mut B,
    own_id: GatewayId,
    hash: &ServiceHash,
    name: &ServiceName,
) -> Result<(), DiscoveryError> {
    let origin = format!("Gateway({})::announce_removed", node.id());
    info!(from origin, "Announcing removal: {}", name);
    fail!(
        from origin,
        when backend.discovery().announce(own_id, Announcement::Removed(hash)),
        with DiscoveryError::DiscoveryAnnouncement,
        "Failed to announce service removal over backend"
    );
    Ok(())
}

/// Whether `details` is offered by at least one live node other than the gateway
/// itself (`gateway_node`). The gateway's own mirror ports keep a service alive in
/// the registry, so they must be excluded when deciding if a service is still
/// locally offered.
fn is_locally_offered<S: Service>(
    details: &ServiceDetails<S>,
    gateway_node: &UniqueNodeId,
) -> bool {
    details.dynamic_details.as_ref().is_some_and(|d| {
        d.nodes.iter().any(|node| match node {
            NodeState::Alive(view) => view.id() != gateway_node,
            NodeState::Inaccessible(id) | NodeState::Undefined(id) => id != gateway_node,
            NodeState::Dead(_) => false,
        })
    })
}
