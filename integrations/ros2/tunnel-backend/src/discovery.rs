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

use std::cell::RefCell;
use std::collections::HashMap;

use core::error::Error;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_services_common::{DiscoveryEvent, DiscoveryEventRef};

use crate::backend::TopicConfig;
use crate::ros_header::RosHeader;
use crate::{keys, rcl};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    Graph(rcl::node::GraphError),
    InvalidServiceName,
    InvalidTypeName,
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

/// Reports presence transitions of the configured topics in the ROS graph:
/// the configuration is the allowlist, the graph decides timing.
#[derive(Debug)]
pub struct Discovery<S: Service> {
    node: rcl::Node,
    /// The configured allowlist: ROS 2 topic → type name.
    topics: HashMap<String, String>,
    /// Configured topics currently present in the graph, with the service
    /// hash they were reported under.
    reported: RefCell<HashMap<String, ServiceHash>>,
    _phantom: core::marker::PhantomData<S>,
}

impl<S: Service> Discovery<S> {
    pub(crate) fn new(node: rcl::Node, topics: &[TopicConfig]) -> Self {
        Self {
            node,
            topics: topics
                .iter()
                .map(|topic| (topic.topic.clone(), topic.type_name.clone()))
                .collect(),
            reported: RefCell::new(HashMap::new()),
            _phantom: core::marker::PhantomData,
        }
    }

    /// The [`StaticConfig`] under which a discovered topic is reported:
    /// service name and payload type per the bridge contract, [`RosHeader`]
    /// user header, everything else from the global iceoryx2 defaults.
    fn static_config(&self, topic: &str, type_name: &str) -> Result<StaticConfig, DiscoveryError> {
        let service_name: ServiceName = keys::service_name(topic)
            .as_str()
            .try_into()
            .map_err(|_| DiscoveryError::InvalidServiceName)?;
        let payload = TypeDetail::__internal_new_from_parts(TypeVariant::Dynamic, type_name, 1, 1)
            .map_err(|_| DiscoveryError::InvalidTypeName)?;
        let user_header = TypeDetail::new::<RosHeader>(TypeVariant::FixedSize);

        Ok(
            StaticConfig::__internal_new_publish_subscribe_with_details::<S::ServiceNameHasher>(
                &service_name,
                IceoryxConfig::global_config(),
                payload,
                user_header,
            ),
        )
    }
}

impl<S: Service> iceoryx2_services_tunnel_backend::traits::Discovery for Discovery<S> {
    type DiscoveryError = DiscoveryError;
    type AnnouncementError = AnnouncementError;

    fn announce(&self, _event: DiscoveryEventRef<'_>) -> Result<(), Self::AnnouncementError> {
        // Nothing to announce explicitly. The tunnel creates a relay for
        // every service it discovers on iceoryx2, and relay creation
        // registers the ROS 2 endpoints, which DDS discovery (SEDP) broadcasts
        // to all participants.
        Ok(())
    }

    fn discover<E: Error, F: FnMut(DiscoveryEvent) -> Result<(), E>>(
        &self,
        mut process_discovery: F,
    ) -> Result<(), Self::DiscoveryError> {
        let graph = self
            .node
            .topic_names_and_types()
            .map_err(DiscoveryError::Graph)?;

        for (topic, type_name) in &self.topics {
            let present = graph.iter().any(|(name, _)| name == topic);
            let reported = self.reported.borrow().contains_key(topic);

            if present && !reported {
                let static_config = self.static_config(topic, type_name)?;
                let service_hash = *static_config.service_hash();
                if process_discovery(DiscoveryEvent::Added(static_config)).is_err() {
                    return Err(DiscoveryError::Processing);
                }
                self.reported
                    .borrow_mut()
                    .insert(topic.clone(), service_hash);
            } else if !present && reported {
                let service_hash = self.reported.borrow()[topic];
                if process_discovery(DiscoveryEvent::Removed(service_hash)).is_err() {
                    return Err(DiscoveryError::Processing);
                }
                self.reported.borrow_mut().remove(topic);
            }
        }

        Ok(())
    }
}
