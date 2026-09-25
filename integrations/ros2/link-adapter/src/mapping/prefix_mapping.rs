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

use core::convert::Infallible;

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_backend::AllowList;
use iceoryx2_link_backend::service_description::{Identified, PatternSettings, ServiceSettings};
use iceoryx2_log::warn;

use crate::config::TopicName;
use crate::endpoint_description::TopicSettings;
use crate::qos::QosProfile;

const TOPIC_PREFIX: &str = "ros2://topics";

/// Maps services named after a topic to that topic.
///
/// A service named `ros2://topics/<topic>` maps to the fully qualified
/// topic `/<topic>`, namespace included, and back, for topics the allow
/// list admits:
///
/// ```text
/// ros2://topics/robot1/camera/front  ↔  /robot1/camera/front
/// ```
///
/// Service settings and QoS profiles overlap only partially, so each side
/// is approximated from the other and the rest is left default. Suitable for
/// prototyping. A [`StaticMapping`](crate::mapping::StaticMapping) is
/// recommended for production deployments and gives full control.
///
/// Service settings to topic QoS (with
/// `depth = max(subscriber_max_buffer_size, history_size)`):
///
/// | Settings               | QoS                                         |
/// |------------------------|---------------------------------------------|
/// | `safe_overflow: true`  | `KeepLast(depth)`                           |
/// | `safe_overflow: false` | `KeepAll`                                   |
/// | `history_size > 0`     | `TransientLocal`                            |
/// | `history_size == 0`    | `Volatile`                                  |
/// | -                      | `Reliable` (iceoryx2 transport is lossless) |
///
/// Topic QoS to service settings:
///
/// | QoS                | Settings                                      |
/// |--------------------|-----------------------------------------------|
/// | `KeepLast(depth)`  | `subscriber_max_buffer_size = depth`          |
/// | `KeepAll`          | `safe_overflow = false`, default buffer size  |
/// | `TransientLocal`   | `history_size = depth`                        |
/// | other durability   | `history_size = 0`                            |
#[derive(Debug, Default, Clone)]
pub struct PrefixMapping {
    allowlist: AllowList,
}

impl PrefixMapping {
    /// Creates a mapping covering the topics `allowlist` admits.
    pub fn new(allowlist: AllowList) -> Self {
        Self { allowlist }
    }
}

impl Mapping for PrefixMapping {
    type EndpointSettings = TopicSettings;
    type Error = Infallible;

    fn local(&self, remote: &TopicSettings) -> Result<Option<ServiceSettings>, Self::Error> {
        if !self.allowlist.admits(remote.topic.as_str()) {
            return Ok(None);
        }
        let name = match service_name(remote.topic.as_str()) {
            Ok(name) => name,
            Err(error) => {
                warn!(
                    "Topic '{}' cannot be mapped to a valid iceoryx2 service name and is not covered: {}",
                    remote.topic.as_str(),
                    error
                );
                return Ok(None);
            }
        };
        Ok(Some(ServiceSettings::new(
            name,
            PatternSettings::PublishSubscribe((&remote.qos).into()),
        )))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<TopicSettings>, Self::Error> {
        let PatternSettings::PublishSubscribe(settings) = &local.pattern else {
            return Ok(None);
        };
        let Some(topic) = topic(local.id().as_str()) else {
            return Ok(None);
        };
        if !self.allowlist.admits(topic.as_str()) {
            return Ok(None);
        }
        Ok(Some(TopicSettings {
            topic,
            qos: QosProfile::from(settings),
        }))
    }
}

fn topic(service_name: &str) -> Option<TopicName> {
    let topic = service_name.strip_prefix(TOPIC_PREFIX)?;
    if !topic.starts_with('/') {
        return None;
    }
    TopicName::new(topic).ok()
}

fn service_name(
    topic: &str,
) -> Result<ServiceName, iceoryx2::service::service_name::ServiceNameError> {
    format!("{TOPIC_PREFIX}{topic}").as_str().try_into()
}
