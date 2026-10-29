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

use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_backend::AllowList;
use iceoryx2_link_backend::service_description::{
    Identified, PatternSettings, RequestResponseSettings, ServiceSettings,
};
use iceoryx2_log::warn;

use crate::adapter::Ros2Adapter;
use crate::config::{self, TopicName};
use crate::endpoint_description::{EndpointName, EndpointSettings};
use crate::qos::QosProfile;

const TOPIC_PREFIX: &str = "ros2://topics";
const SERVICE_PREFIX: &str = "ros2://services";

/// Maps services named after a ROS 2 topic or service to that endpoint.
///
/// A service named `ros2://topics/<topic>` maps to the fully qualified
/// topic `/<topic>`, namespace included, and back. A service named
/// `ros2://services/<service>` maps to the ROS 2 service `/<service>` the
/// same way. These are filtered by the provided [`AllowList`]:
///
/// ```text
/// ros2://topics/robot1/camera/front    ↔  /robot1/camera/front
/// ros2://services/robot1/add_two_ints  ↔  /robot1/add_two_ints
/// ```
///
/// Service settings and QoS profiles overlap only partially, so each side
/// is approximated from the other. Settings the QoS does not provide
/// take the defaults of the iceoryx2 configuration the mapping is created
/// with. QoS the settings do not provide takes the ROS 2 system defaults.
///
/// This mapping is suitable for prototyping. A
/// [`StaticMapping`](crate::mapping::StaticMapping) is recommended for
/// production deployments and gives full control.
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
///
/// A ROS 2 service has no QoS to approximate from. A request-response
/// service takes the defaults of the iceoryx2 configuration and its ROS 2
/// service the system default QoS.
#[derive(Debug, Default, Clone)]
pub struct PrefixMapping {
    allowlist: AllowList,
    config: iceoryx2::config::Config,
}

impl PrefixMapping {
    /// Creates a mapping covering the topics and services `allowlist`
    /// admits. The settings a QoS profile does not provide take the
    /// defaults of `config`.
    pub fn new(allowlist: AllowList, config: iceoryx2::config::Config) -> Self {
        Self { allowlist, config }
    }
}

impl Mapping for PrefixMapping {
    type EndpointSettings = EndpointSettings;
    type Error = Infallible;

    fn local(&self, remote: &EndpointSettings) -> Result<Option<ServiceSettings>, Self::Error> {
        let (prefix, name, pattern) = match &remote.name {
            EndpointName::Topic(topic) => (
                TOPIC_PREFIX,
                topic.as_str(),
                PatternSettings::PublishSubscribe(
                    remote.qos.publish_subscribe_settings(&self.config),
                ),
            ),
            EndpointName::Service(service) => (
                SERVICE_PREFIX,
                service.as_str(),
                PatternSettings::RequestResponse(RequestResponseSettings::from_config(
                    &self.config,
                )),
            ),
        };
        if !self.allowlist.admits(name) {
            return Ok(None);
        }

        let service_name = match service_name(prefix, name) {
            Ok(service_name) => service_name,
            Err(error) => {
                warn!(
                    "'{}' cannot be mapped to a valid iceoryx2 service name and is not covered: {}",
                    name, error
                );
                return Ok(None);
            }
        };

        Ok(Some(ServiceSettings::new(service_name, pattern)))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<EndpointSettings>, Self::Error> {
        if let PatternSettings::RequestResponse(_) = local.pattern
            && !Ros2Adapter::supports(MessagingPattern::RequestResponse)
        {
            return Ok(None);
        }

        let service_name = local.id();
        let prefix = match &local.pattern {
            PatternSettings::PublishSubscribe(_) => TOPIC_PREFIX,
            PatternSettings::RequestResponse(_) => SERVICE_PREFIX,
            PatternSettings::Event(_) => return Ok(None),
        };
        let Some(name) = strip_prefix(prefix, service_name.as_str()) else {
            return Ok(None);
        };
        if !self.allowlist.admits(name) {
            return Ok(None);
        }

        let settings = match &local.pattern {
            PatternSettings::PublishSubscribe(settings) => {
                let Ok(topic) = TopicName::new(name) else {
                    return Ok(None);
                };
                EndpointSettings {
                    name: EndpointName::Topic(topic),
                    qos: QosProfile::from(settings),
                }
            }
            PatternSettings::RequestResponse(_) => {
                let Ok(service) = config::ServiceName::new(name) else {
                    return Ok(None);
                };
                EndpointSettings {
                    name: EndpointName::Service(service),
                    qos: QosProfile::default(),
                }
            }
            PatternSettings::Event(_) => return Ok(None),
        };

        Ok(Some(settings))
    }
}

/// The ROS 2 topic or service name after `prefix` in `service_name`, if
/// the service name carries that prefix.
fn strip_prefix<'a>(prefix: &str, service_name: &'a str) -> Option<&'a str> {
    let name = service_name.strip_prefix(prefix)?;
    name.starts_with('/').then_some(name)
}

/// The iceoryx2 service name of the ROS 2 topic or service `name`, with
/// `prefix`.
fn service_name(
    prefix: &str,
    name: &str,
) -> Result<ServiceName, iceoryx2::service::service_name::ServiceNameError> {
    format!("{prefix}{name}").as_str().try_into()
}
