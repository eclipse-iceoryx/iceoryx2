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
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_gateway_backend::traits::{Mapping, PayloadLayout, Translation, Translator};
use iceoryx2_gateway_backend::types::discovery::{DiscoveryUpdate, DiscoveryUpdateRef};
use iceoryx2_gateway_backend::types::service_description::PatternDescription;
use iceoryx2_log::{fail, warn};

use crate::config::{TopicConfig, TopicName, TypeName};
use crate::mapping::TopicDescription;
use crate::rcl::RclNode;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    Graph,
    Processing,
    Translator,
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

/// Outcome of bridging a configured topic found live in the ROS graph.
#[derive(Debug, Clone, Copy)]
enum TopicState {
    /// Bridged under the given service hash.
    Bridged(ServiceHash),
    /// Could not be bridged. Not retried while the topic stays on the graph.
    Failed,
}

/// Reports liveness status of the configured topics in the ROS graph.
#[derive(Debug)]
pub struct Discovery<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> {
    node: Rc<RclNode>,
    allowlist: HashMap<TopicName, TypeName>,
    mapping: Rc<M>,
    translator: Rc<T>,
    /// Outcome for each configured topic seen live in the ROS graph.
    state: RefCell<HashMap<TopicName, TopicState>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> Discovery<S, M, T>
{
    /// Creates a `Discovery` instance to track `topics` on the ROS graph
    /// via the provided `node`.
    pub(crate) fn new(
        node: Rc<RclNode>,
        topics: &[TopicConfig],
        mapping: Rc<M>,
        translator: Rc<T>,
    ) -> Self {
        Self {
            node,
            allowlist: topics
                .iter()
                .map(|topic| (topic.topic.clone(), topic.type_name.clone()))
                .collect(),
            mapping,
            translator,
            state: RefCell::new(HashMap::new()),
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
        let Some(mut service_description) = self.mapping.local::<S>(&topic_description) else {
            return Ok(());
        };

        // Translated topics carry the payload layout, which only the
        // translator knows; the local service is created from it.
        let translation = fail!(from origin,
            when self.translator.create(&service_description, &topic_description),
            with DiscoveryError::Translator,
            "Translator failed to create translation for topic '{}'",
            topic.as_str()
        );
        if let Translation::Transcode { payload_layout, .. } = translation
            && let PatternDescription::PublishSubscribe(pattern_description) =
                &mut service_description.pattern
        {
            match payload_layout {
                PayloadLayout::FixedSize(layout) => {
                    pattern_description.payload.variant = TypeVariant::FixedSize;
                    pattern_description.payload.size = layout.size();
                    pattern_description.payload.alignment = layout.align();
                }
                PayloadLayout::Dynamic { element } => {
                    pattern_description.payload.variant = TypeVariant::Dynamic;
                    pattern_description.payload.size = element.size();
                    pattern_description.payload.alignment = element.align();
                }
            }
        }

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
        self.state
            .borrow_mut()
            .insert(topic.clone(), TopicState::Bridged(service_hash));

        Ok(())
    }

    /// Bridges a configured topic that has become live.
    ///
    /// A topic that cannot be bridged is recorded as failed, so the reason is
    /// reported once instead of on every discovery run, and the rest of the
    /// configured topics are still processed.
    fn try_discover<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &self,
        topic: &TopicName,
        type_name: &TypeName,
        process_discovery: &mut F,
    ) {
        let Err(error) = self.on_discovered(topic, type_name, process_discovery) else {
            return;
        };

        warn!("Topic '{}' will not be bridged: {}", topic.as_str(), error);
        self.state
            .borrow_mut()
            .insert(topic.clone(), TopicState::Failed);
    }

    /// Handles a previously bridged topic that is no longer live.
    ///
    /// Returns `Ok` when the removal was processed, or `Err` if
    /// `process_discovery` failed.
    fn on_removed<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &self,
        topic: &TopicName,
        service_hash: ServiceHash,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError> {
        let origin = "Discovery::discover_removed";

        // Run discovery logic provided by the caller for the service discovered
        // as removed.
        fail!(from origin,
            when process_discovery(DiscoveryUpdate::Removed(service_hash)),
            with DiscoveryError::Processing,
            "Failed to process discovery 'Removed' event for topic '{}'",
            topic.as_str()
        );

        // Stop tracking the service as discovered.
        self.state.borrow_mut().remove(topic);

        Ok(())
    }
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> iceoryx2_gateway_backend::traits::Discovery for Discovery<S, M, T>
{
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&self, _update: DiscoveryUpdateRef<'_>) -> Result<(), Self::AnnouncementError> {
        // Nothing to announce explicitly. The gateway creates a relay for
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
            let state = self.state.borrow().get(topic).copied();

            match (live, state) {
                (true, None) => self.try_discover(topic, type_name, &mut process_discovery),
                (false, Some(TopicState::Bridged(service_hash))) => {
                    self.on_removed(topic, service_hash, &mut process_discovery)?
                }
                // Forget the failure so the topic is retried should it return.
                (false, Some(TopicState::Failed)) => {
                    self.state.borrow_mut().remove(topic);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
