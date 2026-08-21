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
use iceoryx2_gateway_backend::traits::{Mapping, PayloadLayout, Translation, Translator};
use iceoryx2_gateway_backend::types::discovery::{Announcement, DiscoveryUpdate};
use iceoryx2_gateway_backend::types::identity::GatewayId;
use iceoryx2_gateway_backend::types::service_description::PatternDescription;
use iceoryx2_log::{fail, warn};

use crate::config::{TopicName, TypeName};
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

/// Outcome of bridging a topic found live in the ROS graph.
#[derive(Debug, Clone, Copy)]
enum TopicState {
    /// Bridged under the given service hash.
    Bridged(ServiceHash),
    /// Out of the mapping's scope.
    Unmapped,
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
    mapping: Rc<M>,
    translator: Rc<T>,
    state: HashMap<TopicName, TopicState>,
    _phantom: core::marker::PhantomData<S>,
}

impl<
    S: Service,
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
> Discovery<S, M, T>
{
    /// Creates a `Discovery` instance tracking the ROS graph via the provided
    /// `node`. Only topics mappable by the provided mapper are considered.
    pub(crate) fn new(node: Rc<RclNode>, mapping: Rc<M>, translator: Rc<T>) -> Self {
        Self {
            node,
            mapping,
            translator,
            state: HashMap::new(),
            _phantom: core::marker::PhantomData,
        }
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

    /// Handles a topic that has become live.
    ///
    /// Returns the state to track the topic under, or `Err` if querying its
    /// QoS or running `process_discovery` failed.
    fn on_discovered<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &self,
        own_id: GatewayId,
        topic: &TopicName,
        type_name: &TypeName,
        process_discovery: &mut F,
    ) -> Result<TopicState, DiscoveryError> {
        let origin = "Discovery::on_discovered";

        // The mapping is the sole authority on what is in scope. Topics it
        // does not resolve to a local iceoryx2 service are not considered.
        let topic_description = self.topic_description(topic, type_name)?;
        let Some(mut service_description) = self.mapping.local::<S>(&topic_description) else {
            return Ok(TopicState::Unmapped);
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
            when process_discovery(DiscoveryUpdate::Added(own_id, service_description)),
            with DiscoveryError::Processing,
            "Failed to process discovery 'Added' event for topic '{}'",
            topic.as_str()
        );

        Ok(TopicState::Bridged(service_hash))
    }

    /// Handles a previously bridged topic that is no longer live.
    ///
    /// Returns `Ok` when the removal was processed, or `Err` if
    /// `process_discovery` failed.
    fn on_removed<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &mut self,
        own_id: GatewayId,
        topic: &TopicName,
        service_hash: ServiceHash,
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError> {
        let origin = "Discovery::on_removed";

        // Run discovery logic provided by the caller for the service discovered
        // as removed.
        fail!(from origin,
            when process_discovery(DiscoveryUpdate::Removed(own_id, service_hash)),
            with DiscoveryError::Processing,
            "Failed to process discovery 'Removed' event for topic '{}'",
            topic.as_str()
        );

        // Stop tracking the service as discovered.
        self.state.remove(topic);

        Ok(())
    }

    /// Bridges the topics that have appeared on the graph since the last run.
    ///
    /// Topics that do not get processed successfully are remembered and not
    /// retried in later runs until removed from the graph. After removal,
    /// discovery logic is retried for the new instance of the topic.
    fn discover_additions<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &mut self,
        own_id: GatewayId,
        live: &[(TopicName, TypeName)],
        process_discovery: &mut F,
    ) {
        for (topic, type_name) in live {
            if self.state.contains_key(topic) {
                continue;
            }

            let state = match self.on_discovered(own_id, topic, type_name, process_discovery) {
                Ok(state) => state,
                Err(error) => {
                    warn!("Topic '{}' will not be bridged: {}", topic.as_str(), error);
                    TopicState::Failed
                }
            };

            self.state.insert(topic.clone(), state);
        }
    }

    /// Withdraws the topics that have left the graph since the last run.
    ///
    /// The departed set is taken before the loop so that no borrow of the
    /// tracked state is held while `process_discovery` runs.
    fn discover_removals<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &mut self,
        own_id: GatewayId,
        live: &[(TopicName, TypeName)],
        process_discovery: &mut F,
    ) -> Result<(), DiscoveryError> {
        let is_live = |topic: &TopicName| live.iter().any(|(name, _)| name == topic);

        let departed: Vec<(TopicName, TopicState)> = self
            .state
            .iter()
            .filter(|(topic, _)| !is_live(topic))
            .map(|(topic, state)| (topic.clone(), *state))
            .collect();

        for (topic, state) in departed {
            match state {
                TopicState::Bridged(service_hash) => {
                    self.on_removed(own_id, &topic, service_hash, process_discovery)?
                }
                // Forget the verdict so the topic is judged again if it
                // is discovered again.
                TopicState::Unmapped | TopicState::Failed => {
                    self.state.remove(&topic);
                }
            }
        }

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

    fn announce(
        &mut self,
        _own_id: GatewayId,
        _announcement: Announcement<'_>,
    ) -> Result<(), Self::AnnouncementError> {
        // Nothing to announce explicitly. The gateway creates a relay for
        // every service it discovers on iceoryx2, and relay creation
        // registers the ROS 2 endpoints, which DDS discovery (SEDP) broadcasts
        // to all participants.
        Ok(())
    }

    fn discover<E: Error, F: FnMut(DiscoveryUpdate) -> Result<(), E>>(
        &mut self,
        own_id: GatewayId,
        mut process_discovery: F,
    ) -> Result<(), Self::DiscoveryError> {
        let origin = "Discovery::discover";

        let graph = fail!(from origin,
            when self.node.topic_names_and_types(),
            with DiscoveryError::Graph,
            "Failed to query the ROS 2 graph"
        );

        // Topics with multiple types cannot be bridged safely and are
        // skipped.
        let live: Vec<(TopicName, TypeName)> = graph
            .into_iter()
            .filter_map(|(name, mut types)| {
                if types.len() > 1 {
                    warn!(
                        "Topic '{}' will not be bridged: multiple types found on the topic {:?}",
                        name.as_str(),
                        types
                    );
                    return None;
                }
                let type_name = types.pop()?;

                Some((TopicName::from(name), TypeName::from(type_name)))
            })
            .collect();

        self.discover_additions(own_id, &live, &mut process_discovery);
        self.discover_removals(own_id, &live, &mut process_discovery)
    }
}
