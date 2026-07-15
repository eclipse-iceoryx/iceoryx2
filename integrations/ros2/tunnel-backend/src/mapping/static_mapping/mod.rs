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

//! A [`Mapping`] that uses an explicit lookup table that pairs iceoryx2
//! services and settings with ROS 2 topics and QoS. Allows for full control
//! over settings and qos on both sides.
//!
//! For a zero-configuration alternative see
//! [`PrefixMapping`](crate::mapping::PrefixMapping).
//!
//! Each [`Entry`] pairs one service with one topic. Both endpoints are
//! configured exactly as defined. Nothing is derived and no compatibility
//! of settings on both sides is checked.
//!
//! Services or topics without an entry are not tunneled.

pub mod config;

pub use config::{Config, Entry, IceoryxSettings, RosSettings};

use std::collections::HashMap;

use iceoryx2::service::Service;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_log::fail;
use iceoryx2_services_tunnel_backend::traits::Mapping;
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, PublishSubscribeDescription, ServiceDescription, TypeDescription,
};

use crate::config::{TopicConfig, TopicName};
use crate::mapping::TopicDescription;
use crate::ros_header::RosHeader;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    DuplicateServiceName,
    DuplicateTopic,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

/// A [`Mapping`] defined entirely by configuration: each [`Entry`]
/// pairs one iceoryx2 service with one ROS 2 topic.
#[derive(Debug, Default)]
pub struct StaticMapping {
    entries: Vec<Entry>,
    by_service: HashMap<String, usize>,
    by_topic: HashMap<TopicName, usize>,
}

impl StaticMapping {
    /// Builds the mapping, rejecting configs where a service name or topic
    /// appears in more than one entry.
    pub fn new(config: Config) -> Result<Self, CreationError> {
        let origin = "StaticMapping::new";

        let mut by_service = HashMap::new();
        let mut by_topic = HashMap::new();

        for (index, entry) in config.entries.iter().enumerate() {
            let service_name = entry.iceoryx2.service_name.as_str().to_string();
            if by_service.insert(service_name, index).is_some() {
                fail!(from origin,
                    with CreationError::DuplicateServiceName,
                    "Multiple mapping entries for service '{}'",
                    entry.iceoryx2.service_name.as_str()
                );
            }
            if by_topic.insert(entry.ros2.topic.clone(), index).is_some() {
                fail!(from origin,
                    with CreationError::DuplicateTopic,
                    "Multiple mapping entries for topic '{}'",
                    entry.ros2.topic.as_str()
                );
            }
        }

        Ok(Self {
            entries: config.entries,
            by_service,
            by_topic,
        })
    }

    /// The topics mapped by this instance.
    pub fn topics(&self) -> Vec<TopicConfig> {
        self.entries
            .iter()
            .map(|entry| TopicConfig {
                topic: entry.ros2.topic.clone(),
                type_name: entry.ros2.type_name.clone(),
            })
            .collect()
    }

    /// The entry mapping the service specified by `description`, if present.
    fn entry(&self, description: &ServiceDescription) -> Option<&Entry> {
        if !matches!(description.pattern, PatternDescription::PublishSubscribe(_)) {
            return None;
        }
        self.by_service
            .get(description.name.as_str())
            .map(|&index| &self.entries[index])
    }
}

impl Mapping for StaticMapping {
    type EndpointDescription = TopicDescription;

    fn remote(&self, service_description: &ServiceDescription) -> Option<TopicDescription> {
        let entry = self.entry(service_description)?;

        Some(TopicDescription {
            topic: entry.ros2.topic.clone(),
            type_name: entry.ros2.type_name.clone(),
            qos: entry.ros2.qos.clone(),
        })
    }

    fn local<S: Service>(
        &self,
        topic_description: &TopicDescription,
    ) -> Option<ServiceDescription> {
        let entry = self
            .by_topic
            .get(&topic_description.topic)
            .map(|&index| &self.entries[index])?;

        // The payload is a dynamically-sized CDR stream carrying the
        // configured type name.
        let payload = TypeDescription {
            variant: TypeVariant::Dynamic,
            type_name: entry.iceoryx2.payload_type.clone(),
            size: 1,
            alignment: 1,
        };
        let user_header = TypeDescription::from(&RosHeader::type_detail());

        Some(ServiceDescription::new::<S>(
            entry.iceoryx2.service_name,
            PatternDescription::PublishSubscribe(PublishSubscribeDescription {
                payload,
                user_header,
                settings: entry.iceoryx2.settings.clone(),
            }),
        ))
    }
}
