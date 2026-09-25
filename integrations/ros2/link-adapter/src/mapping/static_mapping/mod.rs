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

pub mod config;
pub use config::{Config, Entry, IceoryxSettings, RosSettings};

use std::collections::HashMap;

use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_backend::service_description::PublishSubscribeSettings;
use iceoryx2_link_backend::service_description::{Identified, PatternSettings, ServiceSettings};
use iceoryx2_log::{fail, origin};

use crate::config::{TopicName, TypeName};
use crate::endpoint_description::TopicSettings;
use crate::qos::QosProfile;

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

/// Mismatch between settings or QoS profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mismatch {
    /// A local service has other settings than its entry defines.
    Settings {
        entry: PublishSubscribeSettings,
        local: PublishSubscribeSettings,
    },
    /// A topic's endpoint has another QoS than its entry defines, as per
    /// what DDS provides.
    Qos {
        entry: QosProfile,
        listed: QosProfile,
    },
}

impl core::fmt::Display for Mismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Mismatch::Settings { entry, local } => {
                write!(f, "has the settings {local:?}, its entry defines {entry:?}")
            }
            Mismatch::Qos { entry, listed } => {
                write!(f, "has the QoS {listed:?}, its entry defines {entry:?}")
            }
        }
    }
}

impl core::error::Error for Mismatch {}

/// Maps services to topics according to a static configuration.
///
/// Each entry pairs a service with a topic and defines the settings of the
/// one and the QoS of the other, see [`Config`] for the schema. A service
/// or topic that no entry names is considered out-of-scope.
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
        let origin = origin!("StaticMapping::new");

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

    /// The message types of the topics covered by configuration.
    pub fn type_names(&self) -> Vec<TypeName> {
        self.entries
            .iter()
            .map(|entry| entry.ros2.type_name.clone())
            .collect()
    }
}

impl Mapping for StaticMapping {
    type EndpointSettings = TopicSettings;
    type Error = Mismatch;

    fn local(&self, remote: &TopicSettings) -> Result<Option<ServiceSettings>, Mismatch> {
        let origin = origin!("StaticMapping::local");

        let Some(&index) = self.by_topic.get(&remote.topic) else {
            return Ok(None);
        };

        let entry = &self.entries[index];
        if !entry.ros2.qos.admits(&remote.qos) {
            fail!(
                from origin,
                with Mismatch::Qos { entry: entry.ros2.qos.clone(), listed: remote.qos.clone() },
                "Topic '{}' has the QoS {:?}, its entry defines {:?}",
                remote.topic, remote.qos, entry.ros2.qos
            );
        }

        Ok(Some(ServiceSettings::new(
            entry.iceoryx2.service_name,
            PatternSettings::PublishSubscribe(entry.iceoryx2.settings.clone()),
        )))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<TopicSettings>, Mismatch> {
        let origin = origin!("StaticMapping::remote");

        let PatternSettings::PublishSubscribe(settings) = &local.pattern else {
            return Ok(None);
        };
        let Some(&index) = self.by_service.get(local.id().as_str()) else {
            return Ok(None);
        };

        let entry = &self.entries[index];
        if *settings != entry.iceoryx2.settings {
            fail!(
                from origin,
                with Mismatch::Settings { entry: entry.iceoryx2.settings.clone(), local: settings.clone() },
                "Service '{}' has the settings {:?}, its entry defines {:?}",
                local.id(), settings, entry.iceoryx2.settings
            );
        }

        Ok(Some(TopicSettings {
            topic: entry.ros2.topic.clone(),
            qos: entry.ros2.qos.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2_bb_testing::assert_that;

    use crate::qos::{History, Liveliness, Reliability};

    const SERVICE: &str = "static/chatter";
    const TOPIC: &str = "/chatter";
    const TYPE: &str = "std_msgs/msg/String";

    fn settings() -> PublishSubscribeSettings {
        PublishSubscribeSettings::from_config(&iceoryx2::config::Config::default())
    }

    fn other_settings() -> PublishSubscribeSettings {
        let mut settings = settings();
        settings.max_publishers += 1;
        settings
    }

    fn other_qos() -> QosProfile {
        QosProfile {
            reliability: Reliability::BestEffort,
            ..QosProfile::default()
        }
    }

    /// A profile as the graph reports it, no history and the liveliness
    /// resolved.
    fn listed(qos: QosProfile) -> QosProfile {
        QosProfile {
            history: History::SystemDefault,
            liveliness: Liveliness::Automatic,
            ..qos
        }
    }

    fn service_settings(settings: PublishSubscribeSettings) -> ServiceSettings {
        ServiceSettings::new(
            ServiceName::new(SERVICE).expect("a valid service name"),
            PatternSettings::PublishSubscribe(settings),
        )
    }

    fn topic_settings(qos: QosProfile) -> TopicSettings {
        TopicSettings {
            topic: TopicName::new(TOPIC).expect("a valid topic name"),
            qos,
        }
    }

    fn mapping() -> StaticMapping {
        StaticMapping::new(Config {
            entries: vec![Entry {
                iceoryx2: IceoryxSettings {
                    service_name: ServiceName::new(SERVICE).expect("a valid service name"),
                    payload_type: TYPE.to_string(),
                    settings: settings(),
                },
                ros2: RosSettings {
                    topic: TopicName::new(TOPIC).expect("a valid topic name"),
                    type_name: TypeName::new(TYPE).expect("a valid type name"),
                    qos: QosProfile::default(),
                },
            }],
        })
        .expect("a valid config")
    }

    #[test]
    fn a_topic_with_the_entrys_qos_maps_to_its_service() {
        let sut = mapping();

        let local = sut.local(&topic_settings(listed(QosProfile::default())));

        assert_that!(local, eq Ok(Some(service_settings(settings()))));
    }

    #[test]
    fn a_topic_with_another_qos_is_refused() {
        let sut = mapping();

        let local = sut.local(&topic_settings(listed(other_qos())));

        assert_that!(
            local,
            eq Err(Mismatch::Qos {
                entry: QosProfile::default(),
                listed: listed(other_qos()),
            })
        );
    }

    #[test]
    fn a_service_with_the_entrys_settings_maps_to_its_topic() {
        let sut = mapping();

        let remote = sut.remote(&service_settings(settings()));

        assert_that!(remote, eq Ok(Some(topic_settings(QosProfile::default()))));
    }

    #[test]
    fn a_service_with_other_settings_is_refused() {
        let sut = mapping();

        let remote = sut.remote(&service_settings(other_settings()));

        assert_that!(remote, eq Err(Mismatch::Settings { entry: settings(), local: other_settings() }));
    }

    #[test]
    fn what_no_entry_covers_is_out_of_scope() {
        let sut = mapping();

        let local = sut.local(&TopicSettings {
            topic: TopicName::new("/elsewhere").expect("a valid topic name"),
            qos: QosProfile::default(),
        });
        let remote = sut.remote(&ServiceSettings::new(
            ServiceName::new("static/elsewhere").expect("a valid service name"),
            PatternSettings::PublishSubscribe(settings()),
        ));

        assert_that!(local, eq Ok(None));
        assert_that!(remote, eq Ok(None));
    }
}
