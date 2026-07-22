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

use std::collections::HashMap;
use std::rc::Rc;

use core::error::Error;

use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::Mapping;
use iceoryx2_services_tunnel_backend::types::discovery::{DiscoveryUpdate, DiscoveryUpdateRef};

use crate::config::{TopicConfig, TopicName, TypeName};
use crate::mapping::TopicDescription;
use crate::rcl::RclNode;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    Graph,
    Processing,
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

/// Reports liveness status of the configured topics in the ROS graph.
#[derive(Debug)]
pub struct Discovery<S: Service, M: Mapping<EndpointDescription = TopicDescription>> {
    node: Rc<RclNode>,
    allowlist: HashMap<TopicName, TypeName>,
    mapping: Rc<M>,
    /// Configured topics detected as live in the ROS graph, with the service
    /// hash they were reported under.
    discovered: RefCell<HashMap<TopicName, ServiceHash>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service, M: Mapping<EndpointDescription = TopicDescription>> Discovery<S, M> {
    /// Creates a `Discovery` instance to track `topics` on the ROS graph
    /// via the provided `node`.
    pub(crate) fn new(node: Rc<RclNode>, topics: &[TopicConfig], mapping: Rc<M>) -> Self {
        Self {
            node,
            allowlist: topics
                .iter()
                .map(|topic| (topic.topic.clone(), topic.type_name.clone()))
                .collect(),
            mapping,
            discovered: RefCell::new(HashMap::new()),
            _phantom: core::marker::PhantomData,
        }
    }

    /// Returns true when `topic` currently appears in a ROS graph snapshot.
    fn is_present_on_graph(
        graph: &[(crate::rcl::TopicName, Vec<crate::rcl::TypeName>)],
        topic: &TopicName,
    ) -> bool {
        graph
            .iter()
            .any(|(name, _)| name.as_str() == topic.as_str())
    }

    /// Builds the topic description for a live `topic`.
    fn topic_description(
        &self,
        topic: &TopicName,
        type_name: &TypeName,
    ) -> Result<TopicDescription, DiscoveryError> {
        let origin = "Discovery::describe_remote";

        let profiles = fail!(from origin,
            when self.node.publisher_qos_profiles(&topic.into()),
            with DiscoveryError::Graph,
            "Failed to query publisher QoS for topic '{}'",
            topic.as_str()
        );

        // Assume a single publisher: take its QoS, defaulting when only
        // subscribers exist. Any further publishers' QoS is ignored.
        // TODO: reconcile QoS across multiple publishers on the same topic.
        let qos = profiles.into_iter().next().unwrap_or_default();

        Ok(TopicDescription {
            topic: topic.clone(),
            type_name: type_name.clone(),
            qos,
        })
    }

    /// Handles a configured topic that has become live.
    ///
    /// Topics the active mapping does not resolve to a local service are
    /// skipped.
    ///
    /// Returns `Ok` when the topic was processed or skipped, or `Err` if
    /// querying its QoS or running `process_discovery` failed.
    fn on_discovered<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &self,
        topic: &TopicName,
        type_name: &TypeName,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError> {
        let origin = "Discovery::discover_added";

        // Skip topic descriptions that the mapping is unable to map
        // to a local iceoryx2 service.
        // These could be topics not following the conventions of the mapping (e.g. prefix)
        // or those explicitly not configured (e.g. static).
        let topic_description = self.topic_description(topic, type_name)?;
        let Some(service_description) = self.mapping.local::<S>(&topic_description) else {
            return Ok(());
        };

        // Run discovery logic provided by the caller for the service discovered
        // as added.
        let service_hash = service_description.service_hash;
        fail!(from origin,
            when process_discovery(DiscoveryUpdate::Added(service_description)),
            with DiscoveryError::Processing,
            "Failed to process discovery 'Added' event for topic '{}'",
            topic.as_str()
        );

        // Keep track of the discovered service for later discovery iterations.
        self.discovered
            .borrow_mut()
            .insert(topic.clone(), service_hash);

        Ok(())
    }

    /// Handles a previously discovered topic that is no longer live.
    ///
    /// Returns `Ok` when the removal was processed, or `Err` if
    /// `process_discovery` failed.
    fn on_removed<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &self,
        topic: &TopicName,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError> {
        let origin = "Discovery::discover_removed";

        // Run discovery logic provided by the caller for the service discovered
        // as removed.
        let service_hash = self.discovered.borrow()[topic];
        fail!(from origin,
            when process_discovery(DiscoveryUpdate::Removed(service_hash)),
            with DiscoveryError::Processing,
            "Failed to process discovery 'Removed' event for topic '{}'",
            topic.as_str()
        );

        // Stop tracking the service as discovered.
        self.discovered.borrow_mut().remove(topic);

        Ok(())
    }
}

impl<S: Service, M: Mapping<EndpointDescription = TopicDescription>>
    iceoryx2_services_tunnel_backend::traits::Discovery for Discovery<S, M>
{
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&self, _update: DiscoveryUpdateRef<'_>) -> Result<(), Self::AnnouncementError> {
        // Nothing to announce explicitly. The tunnel creates a relay for
        // every service it discovers on iceoryx2, and relay creation
        // registers the ROS 2 endpoints, which DDS discovery (SEDP) broadcasts
        // to all participants.
        Ok(())
    }

    fn discover<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), Self::DiscoveryError> {
        let origin = "Discovery::discover";

        let graph = fail!(from origin,
            when self.node.topic_names_and_types(),
            with DiscoveryError::Graph,
            "Failed to query the ROS 2 graph"
        );

        for (topic, type_name) in &self.allowlist {
            let live = Self::is_present_on_graph(&graph, topic);
            let discovered = self.discovered.borrow().contains_key(topic);

            if live && !discovered {
                self.on_discovered(topic, type_name, &mut process_discovery)?;
            } else if !live && discovered {
                self.on_removed(topic, &mut process_discovery)?;
            }
        }

        Ok(())
    }
}
