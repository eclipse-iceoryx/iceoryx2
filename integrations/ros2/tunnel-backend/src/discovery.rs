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
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_bb_concurrency::cell::RefCell;
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::types::discovery::{DiscoveryUpdate, DiscoveryUpdateRef};
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, PublishSubscribeDescription, ServiceDescription, TypeDescription,
};

use crate::config::TopicConfig;
use crate::mapping;
use crate::rcl::{RclNode, TopicName, TypeName};
use crate::ros_header::RosHeader;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    Graph,
    InvalidServiceName,
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
pub struct Discovery<S: Service> {
    node: Rc<RclNode>,
    /// The configured allowlist.
    allowlist: HashMap<TopicName, TypeName>,
    /// Configured topics detected as live in the ROS graph, with the service
    /// hash they were reported under.
    discovered: RefCell<HashMap<TopicName, ServiceHash>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Discovery<S> {
    pub(crate) fn new(node: Rc<RclNode>, topics: &[TopicConfig]) -> Self {
        Self {
            node,
            allowlist: topics
                .iter()
                .map(|topic| {
                    (
                        TopicName::from(&topic.topic),
                        TypeName::from(&topic.type_name),
                    )
                })
                .collect(),
            discovered: RefCell::new(HashMap::new()),
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<S: Service> iceoryx2_services_tunnel_backend::traits::Discovery for Discovery<S> {
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
            let live = graph.iter().any(|(name, _)| name == topic);
            let discovered = self.discovered.borrow().contains_key(topic);

            if live && !discovered {
                let description = fail!(from origin,
                    when service_description::<S>(topic, type_name),
                    "Failed to describe the service for topic '{}'",
                    topic.as_str()
                );

                let service_hash = description.service_hash;
                fail!(from origin,
                    when process_discovery(DiscoveryUpdate::Added(description)),
                    with DiscoveryError::Processing,
                    "Failed to process discovery 'Added' event for topic '{}'",
                    topic.as_str()
                );

                self.discovered
                    .borrow_mut()
                    .insert(topic.clone(), service_hash);
            } else if !live && discovered {
                let service_hash = self.discovered.borrow()[topic];
                fail!(from origin,
                    when process_discovery(DiscoveryUpdate::Removed(service_hash)),
                    with DiscoveryError::Processing,
                    "Failed to process discovery 'Removed' event for topic '{}'",
                    topic.as_str()
                );

                self.discovered.borrow_mut().remove(topic);
            }
        }

        Ok(())
    }
}

/// The [`ServiceDescription`] of the service bridging a discovered ROS
/// topic. Settings are left unset; the tunnel applies local defaults.
fn service_description<S: Service>(
    topic: &TopicName,
    type_name: &TypeName,
) -> Result<ServiceDescription, DiscoveryError> {
    let origin = "discovery::service_description";

    let service_name: ServiceName = fail!(from origin,
        when mapping::service_name(topic.as_str()).as_str().try_into(),
        with DiscoveryError::InvalidServiceName,
        "Invalid service name derived from topic '{}'",
        topic.as_str()
    );

    // TODO: Define configuration per-topic in configuration file; apply here.

    // The payload is a dynamically-sized CDR stream; its type name carries
    // the ROS 2 type name.
    let payload = TypeDescription {
        variant: TypeVariant::Dynamic,
        type_name: type_name.as_str().to_string(),
        size: 1,
        alignment: 1,
    };
    let user_header = TypeDescription::from(&RosHeader::type_detail());

    Ok(ServiceDescription {
        service_hash: ServiceHash::new::<S::ServiceNameHasher>(
            &service_name,
            MessagingPattern::PublishSubscribe,
        ),
        name: service_name,
        pattern: PatternDescription::PublishSubscribe(PublishSubscribeDescription {
            payload,
            user_header,
            settings: None,
        }),
    })
}
