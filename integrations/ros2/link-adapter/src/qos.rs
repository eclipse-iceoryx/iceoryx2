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

use iceoryx2_link_backend::service_description::PublishSubscribeSettings;
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
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Durability {
    SystemDefault,
    TransientLocal,
    Volatile,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum Liveliness {
    SystemDefault,
    Automatic,
    ManualByTopic,
}

/// QoS profile for a ROS 2 endpoint pair created by the gateway.
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

impl QosProfile {
    /// Whether the QoS the graph reports for an endpoint matches this
    /// profile.
    ///
    /// The graph reports concrete policies, so this profile's system
    /// defaults are resolved before comparing. History is left out as the
    /// graph does not report it.
    pub fn admits(&self, listed: &QosProfile) -> bool {
        let resolved = self.resolved();
        resolved.reliability == listed.reliability
            && resolved.durability == listed.durability
            && resolved.liveliness == listed.liveliness
            && resolved.deadline == listed.deadline
            && resolved.lifespan == listed.lifespan
            && resolved.liveliness_lease_duration == listed.liveliness_lease_duration
    }

    /// This profile with every system default replaced by the policy of
    /// the ROS 2 default profile it stands for.
    fn resolved(&self) -> QosProfile {
        let default = QosProfile::default();
        QosProfile {
            history: match self.history {
                History::SystemDefault => default.history,
                history => history,
            },
            reliability: match self.reliability {
                Reliability::SystemDefault => default.reliability,
                reliability => reliability,
            },
            durability: match self.durability {
                Durability::SystemDefault => default.durability,
                durability => durability,
            },
            liveliness: match self.liveliness {
                Liveliness::SystemDefault => Liveliness::Automatic,
                liveliness => liveliness,
            },
            ..self.clone()
        }
    }
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
                Durability::SystemDefault | Durability::Volatile => defaults.publisher_history_size,
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

    use iceoryx2_log::{fail, origin};
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
        let origin = origin!("deserialize");

        let string = fail!(
            from origin,
            when Option::<String>::deserialize(deserializer),
            "Failed to deserialize a duration"
        );
        match string {
            Some(string) => {
                let duration = fail!(
                    from origin,
                    when parse(&string).map_err(serde::de::Error::custom),
                    "Failed to parse duration '{}'", string
                );
                Ok(Some(duration))
            }
            None => Ok(None),
        }
    }

    fn parse(string: &str) -> Result<Duration, String> {
        let origin = origin!("parse");
        let Some(unit_start) = string.find(|c: char| !c.is_ascii_digit()) else {
            fail!(
                from origin,
                with format!("missing unit in duration '{string}'"),
                "Missing unit in duration '{}'", string
            );
        };
        let (value, unit) = string.split_at(unit_start);
        let Ok(value) = value.parse::<u64>() else {
            fail!(
                from origin,
                with format!("invalid value in duration '{string}'"),
                "Invalid value in duration '{}'", string
            );
        };
        match unit {
            "ns" => Ok(Duration::from_nanos(value)),
            "us" => Ok(Duration::from_micros(value)),
            "ms" => Ok(Duration::from_millis(value)),
            "s" => Ok(Duration::from_secs(value)),
            _ => {
                fail!(
                    from origin,
                    with format!("unsupported unit '{unit}' in duration '{string}' (use ns, us, ms or s)"),
                    "Unsupported unit '{}' in duration '{}'", unit, string
                );
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A profile as the graph reports it, no history and the liveliness
    /// resolved.
    fn listed(qos: QosProfile) -> QosProfile {
        QosProfile {
            history: History::SystemDefault,
            liveliness: Liveliness::Automatic,
            ..qos
        }
    }

    #[test]
    fn a_profile_matches_its_own_listing() {
        let profile = QosProfile {
            history: History::KeepLast(3),
            durability: Durability::TransientLocal,
            ..QosProfile::default()
        };

        assert!(profile.admits(&listed(profile.clone())));
    }

    #[test]
    fn a_profile_does_not_match_a_listing_of_another_exchanged_policy() {
        let profile = QosProfile::default();
        let best_effort = QosProfile {
            reliability: Reliability::BestEffort,
            ..QosProfile::default()
        };

        assert!(!profile.admits(&listed(best_effort)));
    }
}
