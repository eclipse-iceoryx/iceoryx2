// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use alloc::boxed::Box;

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::{Node, NodeState, NodeView};
use iceoryx2::port::listener::Listener;
use iceoryx2::prelude::CallbackProgression;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::{Service, ServiceDetails};
use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::resolver::{Resolution, Resolver};
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_link_backend::{Backend, Reactive, Refusal, RemoteDescription, RemoteId, WakeService};
use iceoryx2_log::{fail, origin};

use crate::announcement_state::AnnouncementState;
use crate::bridge::{Bridges, PropagateError};
use crate::discovery_state::DiscoveryState;
use crate::wake::{Wake, WakeCreationError};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    LocalDiscovery,
    BackendListing,
    Announcement,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

/// Extends the local `iceoryx2` system across a boundary through one
/// backend.
pub struct Link<S: Service, B: Backend<S>> {
    backend: B,
    state: DiscoveryState<RemoteId<S, B>, RemoteDescription<S, B>, Refusal<S, B>>,
    announcements: AnnouncementState,
    bridges: Bridges<S, B>,
    /// Whether the link bridges a service, by its name.
    filter: Box<dyn Fn(&ServiceName) -> bool>,
    /// Whether propagation reports counts of propagated data.
    monitoring: bool,
    /// A reactive link's wake service.
    wake: Option<Wake>,
    // Placed last on purpose, required for clean-up order.
    node: Node<S>,
}

impl<S: Service, B: Backend<S>> Link<S, B> {
    /// Creates a link running `backend` from `node`.
    pub fn new(node: Node<S>, backend: B) -> Self {
        Self {
            backend,
            state: DiscoveryState::default(),
            announcements: AnnouncementState::default(),
            bridges: Bridges::default(),
            filter: Box::new(|_| true),
            monitoring: false,
            wake: None,
            node,
        }
    }

    /// Restricts the link to the services whose name `filter` admits, both
    /// the local services it exports and the mirrors it creates.
    pub fn with_filter(mut self, filter: impl Fn(&ServiceName) -> bool + 'static) -> Self {
        self.filter = Box::new(filter);
        self
    }

    /// Reports what every bridge moved each propagation at trace log level.
    pub fn with_monitoring(mut self) -> Self {
        self.monitoring = true;
        self
    }

    /// The node the link participates in the local system as.
    pub fn node(&self) -> &Node<S> {
        &self.node
    }

    /// Runs one discovery cycle. Updates the discovery state from the local
    /// and the opposing side's listings, resolves what changed, opens and
    /// closes bridges to match, and announces what is exported.
    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        let origin = origin!("Link::discover");
        let Self {
            node,
            backend,
            state,
            announcements,
            bridges,
            filter,
            ..
        } = self;

        // Update the state from the local services other nodes hold a port on.
        if let Some(mut update) = state.update_locals(Generation::Untracked) {
            let listed = S::list(node.config(), |details| {
                if is_offered_by_others(&details, node.id()) {
                    let config = &details.static_details;
                    let creation_id = config.unique_service_id().value();
                    if !update.seen(config.service_hash(), creation_id)
                        && let Ok(description) = ServiceDescription::try_from(config)
                    {
                        update.insert(creation_id, description);
                    }
                }
                CallbackProgression::Continue
            });
            fail!(
                from origin,
                when listed,
                with DiscoveryError::LocalDiscovery,
                "Failed to list the local services"
            );
            update.finalize();
        }

        // The resolver borrows the backend until the remote services are resolved.
        {
            // Update the state from the opposing side's listing, each remote under its service.
            let resolver = backend.resolver();
            let generation = backend.generation();
            if let Some(mut update) = state.update_remotes(generation) {
                let mut on_remote = |id: &RemoteId<S, B>, remote: &RemoteDescription<S, B>| {
                    if update.seen(id, remote) {
                        return;
                    }
                    match resolver.service_hash(remote) {
                        Ok(Some(hash)) => update.insert(hash, id.clone(), remote.clone()),
                        Ok(None) => {}
                        Err(refusal) => update.refuse(id.clone(), refusal),
                    }
                };
                fail!(
                    from origin,
                    when backend.list(&mut on_remote),
                    with DiscoveryError::BackendListing,
                    "Failed to list the opposing side"
                );
                update.finalize();
            }

            // Resolve each service a side of which changed.
            for change in state.changes() {
                change.resolve(|local, remotes| {
                    let resolution = resolver.resolve(local, remotes);

                    // Leave the service out when the filter rejects the name it bridges.
                    let bridged = match &resolution {
                        Resolution::Exported(_) => local.map(ServiceDescription::name),
                        Resolution::Imported(mirror, _) => Some(mirror.name()),
                        Resolution::Refused(_) | Resolution::OutOfScope => None,
                    };
                    match bridged {
                        Some(name) if !filter(&name) => Resolution::OutOfScope,
                        _ => resolution,
                    }
                });
            }
        }

        // Open and close bridges to match what resolved.
        bridges.reconcile(node, backend, state.bridgeable());

        // Announce the exported services to the opposing side.
        let mut update = announcements.update(|announcement| backend.announce(announcement));
        let exported = state
            .exported()
            .filter(|(description, _)| bridges.contains(&description.hash()));
        for (description, creation_id) in exported {
            fail!(
                from origin,
                when update.set_exported(description, creation_id),
                with DiscoveryError::Announcement,
                "Failed to announce {}", description.name()
            );
        }
        fail!(
            from origin,
            when update.finalize(),
            with DiscoveryError::Announcement,
            "Failed to withdraw a service no longer exported"
        );

        Ok(())
    }

    /// Moves the samples and notifications pending on every bridge in both
    /// directions.
    pub fn propagate(&mut self) -> Result<(), PropagateError> {
        self.bridges.propagate(self.node.id(), self.monitoring)
    }

    /// The bridges the link currently holds.
    pub fn bridges(&self) -> &Bridges<S, B> {
        &self.bridges
    }
}

impl<S: Service, B: Backend<S> + Reactive> Link<S, B> {
    /// A listener on the link's wake service, signalled whenever a cycle
    /// may have something to do.
    pub fn listener(&mut self) -> Result<Listener<WakeService>, WakeCreationError> {
        let origin = origin!("Link::listener");

        // Create the service.
        if self.wake.is_none() {
            let wake = fail!(
                from origin,
                when Wake::create(self.node.config(), self.node.id()),
                "Failed to create the wake service"
            );
            let handle = fail!(
                from origin,
                when wake.handle(),
                "Failed to create the wake handle"
            );
            self.backend.attach(handle);
            self.wake = Some(wake);
        }

        // Hand the listener to the caller.
        self.wake.as_ref().expect("created above").listener()
    }
}

/// Whether a node other than `own_node` holds a port on the service.
fn is_offered_by_others<S: Service>(details: &ServiceDetails<S>, own_node: &UniqueNodeId) -> bool {
    details.dynamic_details.as_ref().is_some_and(|dynamic| {
        dynamic.nodes.iter().any(|node| match node {
            NodeState::Alive(view) => view.id() != own_node,
            NodeState::Inaccessible(id) | NodeState::Undefined(id) => id != own_node,
            NodeState::Dead(_) => false,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::config::Config;
    use iceoryx2::node::NodeBuilder;
    use iceoryx2::service::local;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2::testing::{generate_isolated_config, generate_service_name};
    use iceoryx2_bb_testing::assert_that;

    /// The names of the services a node other than `own_node`
    /// holds a port on.
    fn offered(config: &Config, own_node: &UniqueNodeId) -> alloc::vec::Vec<ServiceName> {
        let mut offered = alloc::vec::Vec::new();
        local::Service::list(config, |details| {
            if is_offered_by_others(&details, own_node) {
                offered.push(*details.static_details.name());
            }
            CallbackProgression::Continue
        })
        .expect("listing succeeds");
        offered
    }

    #[test]
    fn services_held_by_other_nodes_are_offered() {
        let config = generate_isolated_config();
        let link_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let app_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let _service = app_node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .expect("service is created");

        assert_that!(offered(&config, link_node.id()), eq alloc::vec![service_name]);
    }

    #[test]
    fn services_held_only_by_the_link_are_not_offered() {
        let config = generate_isolated_config();
        let link_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let _service = link_node
            .service_builder(&generate_service_name())
            .publish_subscribe::<u64>()
            .create()
            .expect("service is created");

        assert_that!(offered(&config, link_node.id()), len 0);
    }

    #[test]
    fn services_others_let_go_of_stop_being_offered() {
        let config = generate_isolated_config();
        let link_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let app_node = NodeBuilder::new()
            .config(&config)
            .create::<local::Service>()
            .expect("node is created");
        let service_name = generate_service_name();
        let _link_service = link_node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .create()
            .expect("service is created");
        let app_service = app_node
            .service_builder(&service_name)
            .publish_subscribe::<u64>()
            .open()
            .expect("service is opened");

        assert_that!(offered(&config, link_node.id()), len 1);

        drop(app_service);

        assert_that!(offered(&config, link_node.id()), len 0);
    }
}
