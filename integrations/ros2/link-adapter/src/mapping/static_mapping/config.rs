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

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_backend::service_description::PublishSubscribeSettings;
use serde::{Deserialize, Serialize};

use crate::config::{TopicName, TypeName};
use crate::qos::QosProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceoryxSettings {
    pub service_name: ServiceName,
    pub payload_type: String,
    /// Defaults to `iceoryx2` defaults if omitted.
    #[serde(default = "default_settings")]
    pub settings: PublishSubscribeSettings,
}

fn default_settings() -> PublishSubscribeSettings {
    PublishSubscribeSettings::from_config(&iceoryx2::config::Config::default())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosSettings {
    pub topic: TopicName,
    #[serde(rename = "type")]
    pub type_name: TypeName,
    /// The QoS of the topic either to be used by local endpoints or compared
    /// against remote endpoints for consistency.
    /// DDS does not exchange history, so it is not checked against
    /// remote endpoints.
    #[serde(default)]
    pub qos: QosProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub iceoryx2: IceoryxSettings,
    pub ros2: RosSettings,
}

/// Configuration schema of a [`StaticMapping`](super::StaticMapping).
///
/// One entry per service and topic pair, loadable from any format serde
/// supports. e.g. In TOML (showing every field):
///
/// ```toml
/// [[mapping]]
/// iceoryx2.service_name = "CmdVel"
/// iceoryx2.payload_type = "geometry_msgs/msg/Twist"
/// iceoryx2.settings.max_publishers = 1
/// iceoryx2.settings.max_subscribers = 4
/// iceoryx2.settings.max_nodes = 8
/// iceoryx2.settings.history_size = 0
/// iceoryx2.settings.subscriber_max_buffer_size = 2
/// iceoryx2.settings.subscriber_max_borrowed_samples = 2
/// iceoryx2.settings.safe_overflow = true
/// ros2.topic = "/cmd_vel"
/// ros2.type = "geometry_msgs/msg/Twist"
/// ros2.qos.history = { KeepLast = 10 }
/// ros2.qos.reliability = "Reliable"
/// ros2.qos.durability = "Volatile"
/// ros2.qos.liveliness = "Automatic"
/// ros2.qos.deadline = "100ms"
/// ros2.qos.lifespan = "1s"
/// ros2.qos.liveliness_lease_duration = "500ms"
/// ```
///
/// Names and types are required. The `settings` table is optional as a
/// whole and takes the `iceoryx2` configuration's defaults when omitted.
/// Every `qos` field is optional on its own and takes the ROS 2 system
/// default when omitted.
///
/// Possible QoS values are:
///
/// * `history`: `"SystemDefault"`, `"KeepAll"` or `{ KeepLast = <n> }`
/// * `reliability`: `"SystemDefault"`, `"Reliable"` or `"BestEffort"`
/// * `durability`: `"SystemDefault"`, `"Volatile"` or `"TransientLocal"`
/// * `liveliness`: `"SystemDefault"`, `"Automatic"` or `"ManualByTopic"`
/// * `deadline`, `lifespan`, `liveliness_lease_duration`: a duration such
///   as `"500ms"`, in `ns`, `us`, `ms` or `s`. If omitted, the policy is
///   unbound.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Serialized as `mapping`: entries appear as `[[mapping]]` in TOML.
    #[serde(default, rename = "mapping")]
    pub entries: Vec<Entry>,
}
