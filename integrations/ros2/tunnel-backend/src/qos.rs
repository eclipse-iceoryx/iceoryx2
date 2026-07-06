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

use core::time::Duration;

use iceoryx2_services_tunnel_backend::types::service_description::PublishSubscribeSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum History {
    SystemDefault,
    KeepLast(usize),
    KeepAll,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Reliability {
    SystemDefault,
    Reliable,
    BestEffort,
    BestAvailable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Durability {
    SystemDefault,
    TransientLocal,
    Volatile,
    BestAvailable,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Liveliness {
    SystemDefault,
    Automatic,
    ManualByTopic,
    BestAvailable,
}

/// QoS profile for a ROS 2 endpoint pair created by the tunnel.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct QosProfile {
    pub history: History,
    pub reliability: Reliability,
    pub durability: Durability,
    #[serde(with = "duration_string", skip_serializing_if = "Option::is_none")]
    pub deadline: Option<Duration>,
    #[serde(with = "duration_string", skip_serializing_if = "Option::is_none")]
    pub lifespan: Option<Duration>,
    pub liveliness: Liveliness,
    #[serde(with = "duration_string", skip_serializing_if = "Option::is_none")]
    pub liveliness_lease_duration: Option<Duration>,
}

impl Default for QosProfile {
    /// Matches the ROS 2 default profile (`rmw_qos_profile_default`).
    fn default() -> Self {
        Self {
            history: History::KeepLast(10),
            reliability: Reliability::Reliable,
            durability: Durability::Volatile,
            deadline: None,
            lifespan: None,
            liveliness: Liveliness::SystemDefault,
            liveliness_lease_duration: None,
        }
    }
}

impl From<&PublishSubscribeSettings> for QosProfile {
    fn from(settings: &PublishSubscribeSettings) -> Self {
        let depth = settings
            .subscriber_max_buffer_size
            .max(settings.history_size);

        Self {
            history: if settings.safe_overflow {
                History::KeepLast(depth)
            } else {
                History::KeepAll
            },
            reliability: Reliability::Reliable,
            durability: if settings.history_size > 0 {
                Durability::TransientLocal
            } else {
                Durability::Volatile
            },
            ..Self::default()
        }
    }
}

impl From<&QosProfile> for PublishSubscribeSettings {
    fn from(profile: &QosProfile) -> Self {
        let defaults = iceoryx2::config::Config::default()
            .defaults
            .publish_subscribe;

        let depth = match profile.history {
            History::KeepLast(depth) => depth,
            History::SystemDefault | History::KeepAll => defaults.subscriber_max_buffer_size,
        };

        Self {
            max_subscribers: defaults.max_subscribers,
            max_publishers: defaults.max_publishers,
            max_nodes: defaults.max_nodes,
            history_size: match profile.durability {
                Durability::TransientLocal => depth,
                Durability::SystemDefault | Durability::Volatile | Durability::BestAvailable => {
                    defaults.publisher_history_size
                }
            },
            subscriber_max_buffer_size: depth,
            subscriber_max_borrowed_samples: defaults.subscriber_max_borrowed_samples,
            safe_overflow: !matches!(profile.history, History::KeepAll),
        }
    }
}

// Custom serializer/deserializer to support unit strings in
// mapping configuration.
mod duration_string {
    use core::time::Duration;

    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        duration: &Option<Duration>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match duration {
            Some(duration) => serializer.serialize_str(&format(duration)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Duration>, D::Error> {
        Option::<String>::deserialize(deserializer)?
            .map(|string| parse(&string).map_err(serde::de::Error::custom))
            .transpose()
    }

    fn parse(string: &str) -> Result<Duration, String> {
        let unit_start = string
            .find(|c: char| !c.is_ascii_digit())
            .ok_or_else(|| format!("missing unit in duration '{string}'"))?;
        let (value, unit) = string.split_at(unit_start);
        let value: u64 = value
            .parse()
            .map_err(|_| format!("invalid value in duration '{string}'"))?;
        match unit {
            "ns" => Ok(Duration::from_nanos(value)),
            "us" => Ok(Duration::from_micros(value)),
            "ms" => Ok(Duration::from_millis(value)),
            "s" => Ok(Duration::from_secs(value)),
            _ => Err(format!(
                "unsupported unit '{unit}' in duration '{string}' (use ns, us, ms or s)"
            )),
        }
    }

    fn format(duration: &Duration) -> String {
        let nanos = duration.as_nanos();
        if nanos.is_multiple_of(1_000_000_000) {
            format!("{}s", nanos / 1_000_000_000)
        } else if nanos.is_multiple_of(1_000_000) {
            format!("{}ms", nanos / 1_000_000)
        } else if nanos.is_multiple_of(1_000) {
            format!("{}us", nanos / 1_000)
        } else {
            format!("{nanos}ns")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn parses_all_units() {
            assert_eq!(parse("500ms"), Ok(Duration::from_millis(500)));
            assert_eq!(parse("2s"), Ok(Duration::from_secs(2)));
            assert_eq!(parse("10us"), Ok(Duration::from_micros(10)));
            assert_eq!(parse("7ns"), Ok(Duration::from_nanos(7)));
        }

        #[test]
        fn rejects_malformed_durations() {
            for input in ["", "500", "ms", "5.5s", "2h", "-1s"] {
                assert!(parse(input).is_err(), "{input}");
            }
        }

        #[test]
        fn formats_with_the_largest_exact_unit() {
            assert_eq!(format(&Duration::from_millis(500)), "500ms");
            assert_eq!(format(&Duration::from_secs(2)), "2s");
            assert_eq!(format(&Duration::from_nanos(1_000_500)), "1000500ns");
        }
    }
}
