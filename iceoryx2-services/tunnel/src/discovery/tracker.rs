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

use alloc::vec::Vec;

use iceoryx2::config::Config;
use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::{NodeState, NodeView};
use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::{fail, fatal_panic};
use iceoryx2_services_common::DiscoveryEvent;
use iceoryx2_services_discovery::service_discovery::Tracker;
use iceoryx2_services_tunnel_backend::traits::Discovery;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    TrackerSynchronization,
    DiscoveryProcessing,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnnouncementError {}

impl core::fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "AnnouncementError::{self:?}")
    }
}

impl core::error::Error for AnnouncementError {}

#[derive(Debug)]
pub struct DiscoveryTracker<S: Service> {
    tracker: RefCell<Tracker<S>>,
    node_id: UniqueNodeId,
}

impl<S: Service> DiscoveryTracker<S> {
    /// Creates a [`DiscoveryTracker`].
    ///
    /// `node_id` must be the [`UniqueNodeId`] of the tunnel's own
    /// [`Node`](iceoryx2::node::Node). It is used during `discover` to
    /// recognise services that remain in the system listing only because the
    /// tunnel itself opened a port on them.
    pub fn create(iceoryx_config: &Config, node_id: UniqueNodeId) -> Self {
        let tracker = Tracker::new(iceoryx_config);
        DiscoveryTracker {
            tracker: RefCell::new(tracker),
            node_id,
        }
    }
}

impl<S: Service> Discovery for DiscoveryTracker<S> {
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&self, _discovery: DiscoveryEvent) -> Result<(), Self::AnnouncementError> {
        // NOOP - iceoryx2 handles discovery internally
        Ok(())
    }

    fn discover<E: core::error::Error, F: FnMut(&DiscoveryEvent) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), Self::DiscoveryError> {
        let tracker = &mut self.tracker.borrow_mut();

        let (added, removed) = fail!(
            from self,
            when tracker.sync(),
            with DiscoveryError::TrackerSynchronization,
            "Failed to synchronize tracker"
        );

        // Newly-appeared services.
        for id in added {
            match tracker.get(&id) {
                Some(service_details) => {
                    fail!(
                        from self,
                        when process_discovery(&DiscoveryEvent::Added(service_details.static_details.clone())),
                        with DiscoveryError::DiscoveryProcessing,
                        "Failed to process discovery event"
                    );
                }
                None => {
                    fatal_panic!(
                        from "DiscoveryTracker::discover",
                        "This should never happen. Service discovered by tracker is not retrievable."
                    )
                }
            }
        }

        // Services that disappeared from the listing entirely.
        for service_details in removed {
            fail!(
                from self,
                when process_discovery(&DiscoveryEvent::Removed(*service_details.static_details.service_hash())),
                with DiscoveryError::DiscoveryProcessing,
                "Failed to process discovery event"
            );
        }

        // Services still listed but only held by this tunnel's node: the user-side owner is
        // gone, so the service is logically removed from the tunnel's perspective.
        let abandoned: Vec<ServiceHash> = tracker
            .get_all()
            .iter()
            .filter(|service_details| {
                service_details.dynamic_details.as_ref().is_some_and(|d| {
                    d.nodes
                        .iter()
                        .filter_map(node_id)
                        .all(|id| id == &self.node_id)
                })
            })
            .map(|details| *details.static_details.service_hash())
            .collect();

        for hash in abandoned {
            tracker.forget(&hash);
            fail!(
                from self,
                when process_discovery(&DiscoveryEvent::Removed(hash)),
                with DiscoveryError::DiscoveryProcessing,
                "Failed to process discovery event"
            );
        }

        Ok(())
    }
}

fn node_id<S: Service>(node: &NodeState<S>) -> Option<&UniqueNodeId> {
    match node {
        NodeState::Alive(view) => Some(view.id()),
        NodeState::Inaccessible(id) | NodeState::Undefined(id) => Some(id),
        // Dead nodes are not active holders.
        NodeState::Dead(_) => None,
    }
}
