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

//! Maps service names following a defined prefix convention to ROS 2
//! topics.
//!
//! This mapping strategy is intended to get communication
//! up-and-running quickly with minimal configuration. Since many QoS
//! can only be approximated with iceoryx2 service settings, this mapping
//! should only used during development / prototyping.
//!
//! For finer control over the settings and QoS, the the
//! [`StaticMapping`](crate::mapping::StaticMapping) strategy.
//!
//! # Name mapping
//!
//! A service is bridgeable when its name carries the fully-qualified
//! topic under the `ros2://topics` prefix and its payload type name is
//! the ROS 2 type name:
//!
//! ```text
//! ros2://topics/Camera/FrontRight  ↔  /Camera/FrontRight
//! ```
//!
//! Any other form is not tunneled.
//!
//! # QoS mapping
//!
//! Service settings and QoS profiles overlap only partially. Settings that
//! can be mapped to ROS 2 QoS are applied. The rest are left default.
//!
//! Outbound, service settings → topic QoS:
//!
//! | Settings               | QoS                                         |
//! |------------------------|---------------------------------------------|
//! | `safe_overflow: true`  | `KeepLast(depth)`                           |
//! | `safe_overflow: false` | `KeepAll`                                   |
//! | `history_size > 0`     | `TransientLocal`                            |
//! | `history_size == 0`    | `Volatile`                                  |
//! | -                      | `Reliable` (iceoryx2 transport is lossless) |
//!
//! ```text
//! depth = max(subscriber_max_buffer_size, history_size)
//! ```
//!
//! Inbound, topic QoS → service settings:
//!
//! | QoS                | Settings                                      |
//! |--------------------|-----------------------------------------------|
//! | `KeepLast(depth)`  | `subscriber_max_buffer_size = depth`          |
//! | `KeepAll`          | `safe_overflow = false`, default buffer size  |
//! | `TransientLocal`   | `history_size = depth`                        |
//! | other durability   | `history_size = 0`                            |
//!

use iceoryx2::service::Service;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_services_tunnel_backend::traits::Mapping;
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, PatternSettings, PublishSubscribeDescription, ServiceDescription,
    TypeDescription,
};

use crate::config::{TopicName, TypeName};
use crate::mapping::TopicDescription;
use crate::qos::QosProfile;
use crate::ros_header::RosHeader;

const TOPIC_PREFIX: &str = "ros2://topics";

/// Accepts publish-subscribe services named `ros2://topics{topic}` whose
/// payload type name is the ROS 2 type name, and attempts to derive each
/// side's QoS/settings.
#[derive(Debug, Default, Clone, Copy)]
pub struct PrefixMapping;

impl Mapping for PrefixMapping {
    type EndpointDescription = TopicDescription;

    fn remote(&self, description: &ServiceDescription) -> Option<TopicDescription> {
        let PatternDescription::PublishSubscribe(pattern) = &description.pattern else {
            return None;
        };
        let payload = &pattern.payload;

        let topic = TopicName::new(topic(description.name.as_str())?).ok()?;
        let type_name = TypeName::new(&payload.type_name).ok()?;

        Some(TopicDescription {
            topic,
            type_name,
            qos: match &pattern.settings {
                PatternSettings::Value(settings) => QosProfile::from(settings),
                PatternSettings::UnknownApplyDefaults => QosProfile::default(),
            },
        })
    }

    fn local<S: Service>(&self, remote: &TopicDescription) -> Option<ServiceDescription> {
        let name: ServiceName = service_name(remote.topic.as_str())
            .as_str()
            .try_into()
            .expect("prefixed topic names are valid service names");

        // The payload is a dynamically-sized CDR stream; its type name
        // carries the ROS 2 type name.
        let payload = TypeDescription {
            variant: TypeVariant::Dynamic,
            type_name: remote.type_name.as_str().to_string(),
            size: 1,
            alignment: 1,
        };
        let user_header = TypeDescription::from(&RosHeader::type_detail());

        Some(ServiceDescription {
            service_hash: ServiceHash::new::<S::ServiceNameHasher>(
                &name,
                MessagingPattern::PublishSubscribe,
            ),
            name,
            pattern: PatternDescription::PublishSubscribe(PublishSubscribeDescription {
                payload,
                user_header,
                settings: PatternSettings::Value((&remote.qos).into()),
            }),
        })
    }
}

/// Convert a conventional iceoryx2 service name into a ROS 2 topic name.
fn topic(service_name: &str) -> Option<&str> {
    let topic = service_name.strip_prefix(TOPIC_PREFIX)?;
    if !topic.starts_with('/') || topic.len() == 1 {
        return None;
    }
    Some(topic)
}

/// Convert a ROS 2 topic name into a conventional iceoryx2 service name.
fn service_name(topic: &str) -> String {
    format!("{TOPIC_PREFIX}{topic}")
}
