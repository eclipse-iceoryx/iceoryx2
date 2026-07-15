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

//! Configuration schema of a [`StaticMapping`](super::StaticMapping).
//!
//! The configuration types are serde-compatible and can be loaded from
//! any format serde supports. In TOML, each entry is a `[[mapping]]`
//! table:
//!
//! ```toml
//! [[mapping]]
//! iceoryx2.service_name = "CmdVel"
//! iceoryx2.payload_type = "geometry_msgs/msg/Twist"
//! ros2.topic = "/cmd_vel"
//! ros2.type = "geometry_msgs/msg/Twist"
//! ...
//! ```
//!
//! Names and types are required. The `settings` and `qos` tables are optional
//! and fall back to the iceoryx2 and ROS 2 defaults when unset.
//!
//! ## `iceoryx2` — local service
//!
//! | Field          | Value                       |
//! |----------------|-----------------------------|
//! | `service_name` | string (required)           |
//! | `payload_type` | string (required)           |
//! | `settings`     | table, see below (optional) |
//!
//! ## `iceoryx2.settings` — local service settings
//!
//! | Field                             | Value   |
//! |-----------------------------------|---------|
//! | `max_subscribers`                 | integer |
//! | `max_publishers`                  | integer |
//! | `max_nodes`                       | integer |
//! | `history_size`                    | integer |
//! | `subscriber_max_buffer_size`      | integer |
//! | `subscriber_max_borrowed_samples` | integer |
//! | `safe_overflow`                   | boolean |
//!
//! ## `ros2` — remote topic
//!
//! | Field   | Value                       |
//! |---------|-----------------------------|
//! | `topic` | string (required)           |
//! | `type`  | string (required)           |
//! | `qos`   | table, see below (optional) |
//!
//! ## `ros2.qos` — endpoint QoS
//!
//! | Policy                      | Value                                                                  |
//! |-----------------------------|------------------------------------------------------------------------|
//! | `history`                   | `"SystemDefault"`, `"KeepAll"`, `{ KeepLast = <n> }`                   |
//! | `reliability`               | `"SystemDefault"`, `"Reliable"`, `"BestEffort"`, `"BestAvailable"`     |
//! | `durability`                | `"SystemDefault"`, `"Volatile"`, `"TransientLocal"`, `"BestAvailable"` |
//! | `liveliness`                | `"SystemDefault"`, `"Automatic"`, `"ManualByTopic"`, `"BestAvailable"` |
//! | `deadline`                  | duration                                                               |
//! | `lifespan`                  | duration                                                               |
//! | `liveliness_lease_duration` | duration                                                               |
//!
//! Durations are strings of the form `"<value><unit>"` (units `ns`, `us`,
//! `ms`, `s`), e.g. `"500ms"`; unset means no bound.
//!
//! See `examples/mapping.toml` for a complete entry.

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_services_tunnel_backend::types::service_description::{
    PortSettings, PublishSubscribeSettings,
};
use serde::{Deserialize, Serialize};

use crate::config::{TopicName, TypeName};
use crate::qos::QosProfile;

/// The iceoryx2 half of an [`Entry`]: the local service and its
/// settings. Omitted settings let the tunnel apply its local defaults; a
/// partial `settings` table fills the rest from the iceoryx2 defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceoryxSettings {
    pub service_name: ServiceName,
    pub payload_type: String,
    // Custom serialization to make the PortSettings enum transparent.
    #[serde(
        default = "port_settings_inline::local_defaults",
        skip_serializing_if = "port_settings_inline::is_local_defaults",
        with = "port_settings_inline"
    )]
    pub settings: PortSettings<PublishSubscribeSettings>,
}

/// The ROS 2 half of an [`Entry`]: the topic, its message type and
/// the QoS of the tunnel's endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosSettings {
    pub topic: TopicName,
    #[serde(rename = "type")]
    pub type_name: TypeName,
    #[serde(default)]
    pub qos: QosProfile,
}

/// One iceoryx2 service ↔ ROS 2 topic pairing. Both sides are applied
/// verbatim; nothing is derived and no cross-side compatibility is checked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub iceoryx2: IceoryxSettings,
    pub ros2: RosSettings,
}

/// Serializable list of [`Entry`]s, one per tunneled service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Serialized as `mapping`: entries appear as `[[mapping]]` in TOML.
    #[serde(default, rename = "mapping")]
    pub entries: Vec<Entry>,
}

mod port_settings_inline {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::PortSettings;

    pub fn serialize<S: Serializer, T: Serialize>(
        settings: &PortSettings<T>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match settings {
            PortSettings::Value(value) => value.serialize(serializer),
            PortSettings::LocalDefaults => serializer.serialize_unit(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
        deserializer: D,
    ) -> Result<PortSettings<T>, D::Error> {
        T::deserialize(deserializer).map(PortSettings::Value)
    }

    pub fn local_defaults<T>() -> PortSettings<T> {
        PortSettings::LocalDefaults
    }

    pub fn is_local_defaults<T>(settings: &PortSettings<T>) -> bool {
        matches!(settings, PortSettings::LocalDefaults)
    }
}
