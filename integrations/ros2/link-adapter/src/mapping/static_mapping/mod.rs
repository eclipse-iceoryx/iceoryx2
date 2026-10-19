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
pub use config::{
    Config, Entry, IceoryxPublishSubscribeSettings, IceoryxRequestResponseSettings,
    RosServiceSettings, RosTopicSettings, ServiceEntry, TopicEntry,
};

use std::collections::HashMap;

use iceoryx2::service::service_name::ServiceName;
use iceoryx2_link_adapter::Mapping;
use iceoryx2_link_backend::service_description::{Identified, PatternSettings, ServiceSettings};
use iceoryx2_log::{fail, origin};

use crate::config::TypeName;
use crate::endpoint_description::{EndpointName, EndpointSettings};
use crate::qos::QosProfile;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    DuplicateServiceName,
    DuplicateTopic,
    DuplicateService,
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
    /// A local service has another pattern or other settings than its
    /// entry defines.
    Settings {
        entry: PatternSettings,
        local: PatternSettings,
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

impl Entry {
    fn service_name(&self) -> &ServiceName {
        match self {
            Self::Topic(entry) => &entry.iceoryx2.service_name,
            Self::Service(entry) => &entry.iceoryx2.service_name,
        }
    }

    fn pattern_settings(&self) -> PatternSettings {
        match self {
            Self::Topic(entry) => {
                PatternSettings::PublishSubscribe(entry.iceoryx2.settings.clone())
            }
            Self::Service(entry) => {
                PatternSettings::RequestResponse(entry.iceoryx2.settings.clone())
            }
        }
    }

    fn endpoint_name(&self) -> EndpointName {
        match self {
            Self::Topic(entry) => EndpointName::Topic(entry.ros2.topic.clone()),
            Self::Service(entry) => EndpointName::Service(entry.ros2.service.clone()),
        }
    }

    fn type_name(&self) -> &TypeName {
        match self {
            Self::Topic(entry) => &entry.ros2.type_name,
            Self::Service(entry) => &entry.ros2.type_name,
        }
    }

    fn qos(&self) -> &QosProfile {
        match self {
            Self::Topic(entry) => &entry.ros2.qos,
            Self::Service(entry) => &entry.ros2.qos,
        }
    }
}

/// Maps services to topics and ROS 2 services according to a static
/// configuration.
///
/// Each entry pairs a service with a ROS 2 topic or service and defines
/// the settings of the one and the QoS of the other. See [`Config`] for the
/// schema. A service, topic or ROS 2 service that no entry names is
/// considered out-of-scope.
#[derive(Debug, Default)]
pub struct StaticMapping {
    entries: Vec<Entry>,
    by_service: HashMap<String, usize>,
    by_name: HashMap<EndpointName, usize>,
}

impl StaticMapping {
    /// Builds the mapping, rejecting configs where a service name, a topic
    /// or a ROS 2 service appears in more than one entry.
    pub fn new(config: Config) -> Result<Self, CreationError> {
        let origin = origin!("StaticMapping::new");

        let mut by_service = HashMap::new();
        let mut by_name = HashMap::new();

        for (index, entry) in config.entries.iter().enumerate() {
            let service_name = entry.service_name().as_str().to_string();
            if by_service.insert(service_name, index).is_some() {
                fail!(from origin,
                    with CreationError::DuplicateServiceName,
                    "Multiple mapping entries for service '{}'",
                    entry.service_name().as_str()
                );
            }
            let name = entry.endpoint_name();
            if by_name.insert(name.clone(), index).is_some() {
                match name {
                    EndpointName::Topic(topic) => {
                        fail!(from origin,
                            with CreationError::DuplicateTopic,
                            "Multiple mapping entries for topic '{}'",
                            topic
                        );
                    }
                    EndpointName::Service(service) => {
                        fail!(from origin,
                            with CreationError::DuplicateService,
                            "Multiple mapping entries for ROS 2 service '{}'",
                            service
                        );
                    }
                }
            }
        }

        Ok(Self {
            entries: config.entries,
            by_service,
            by_name,
        })
    }

    /// The message and service types of the entries.
    pub fn type_names(&self) -> Vec<TypeName> {
        self.entries
            .iter()
            .map(|entry| entry.type_name().clone())
            .collect()
    }
}

impl Mapping for StaticMapping {
    type EndpointSettings = EndpointSettings;
    type Error = Mismatch;

    fn local(&self, remote: &EndpointSettings) -> Result<Option<ServiceSettings>, Mismatch> {
        let origin = origin!("StaticMapping::local");

        let Some(&index) = self.by_name.get(&remote.name) else {
            return Ok(None);
        };

        let entry = &self.entries[index];
        if !entry.qos().admits(&remote.qos) {
            fail!(
                from origin,
                with Mismatch::Qos { entry: entry.qos().clone(), listed: remote.qos.clone() },
                "'{}' has the QoS {:?}, its entry defines {:?}",
                remote.name, remote.qos, entry.qos()
            );
        }

        Ok(Some(ServiceSettings::new(
            *entry.service_name(),
            entry.pattern_settings(),
        )))
    }

    fn remote(&self, local: &ServiceSettings) -> Result<Option<EndpointSettings>, Mismatch> {
        let origin = origin!("StaticMapping::remote");

        let Some(&index) = self.by_service.get(local.id().as_str()) else {
            return Ok(None);
        };

        let entry = &self.entries[index];
        let settings = entry.pattern_settings();
        if local.pattern != settings {
            fail!(
                from origin,
                with Mismatch::Settings { entry: settings, local: local.pattern.clone() },
                "Service '{}' has the settings {:?}, its entry defines {:?}",
                local.id(), local.pattern, entry.pattern_settings()
            );
        }

        Ok(Some(EndpointSettings {
            name: entry.endpoint_name(),
            qos: entry.qos().clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::service_description::{
        PublishSubscribeSettings, RequestResponseSettings,
    };

    use crate::config::{self, TopicName};
    use crate::qos::{History, Liveliness, Reliability};

    const SERVICE: &str = "static/chatter";
    const TOPIC: &str = "/chatter";
    const TYPE: &str = "std_msgs/msg/String";
    const REQUEST_RESPONSE_SERVICE: &str = "static/add_two_ints";
    const ROS_SERVICE: &str = "/add_two_ints";
    const SERVICE_TYPE: &str = "example_interfaces/srv/AddTwoInts";

    fn publish_subscribe_settings() -> PublishSubscribeSettings {
        PublishSubscribeSettings::from_config(&iceoryx2::config::Config::default())
    }

    fn other_publish_subscribe_settings() -> PublishSubscribeSettings {
        let mut settings = publish_subscribe_settings();
        settings.max_publishers += 1;
        settings
    }

    fn request_response_settings() -> RequestResponseSettings {
        RequestResponseSettings::from_config(&iceoryx2::config::Config::default())
    }

    fn other_request_response_settings() -> RequestResponseSettings {
        let mut settings = request_response_settings();
        settings.max_servers += 1;
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

    fn topic_settings(qos: QosProfile) -> EndpointSettings {
        EndpointSettings {
            name: EndpointName::Topic(TopicName::new(TOPIC).expect("a valid topic name")),
            qos,
        }
    }

    fn request_response_service_settings(settings: RequestResponseSettings) -> ServiceSettings {
        ServiceSettings::new(
            ServiceName::new(REQUEST_RESPONSE_SERVICE).expect("a valid service name"),
            PatternSettings::RequestResponse(settings),
        )
    }

    fn ros_service_settings(qos: QosProfile) -> EndpointSettings {
        EndpointSettings {
            name: EndpointName::Service(
                config::ServiceName::new(ROS_SERVICE).expect("a valid service name"),
            ),
            qos,
        }
    }

    fn mapping() -> StaticMapping {
        StaticMapping::new(Config {
            entries: vec![
                Entry::Topic(TopicEntry {
                    iceoryx2: IceoryxPublishSubscribeSettings {
                        service_name: ServiceName::new(SERVICE).expect("a valid service name"),
                        payload_type: TYPE.to_string(),
                        settings: publish_subscribe_settings(),
                    },
                    ros2: RosTopicSettings {
                        topic: TopicName::new(TOPIC).expect("a valid topic name"),
                        type_name: TypeName::new(TYPE).expect("a valid type name"),
                        qos: QosProfile::default(),
                    },
                }),
                Entry::Service(ServiceEntry {
                    iceoryx2: IceoryxRequestResponseSettings {
                        service_name: ServiceName::new(REQUEST_RESPONSE_SERVICE)
                            .expect("a valid service name"),
                        request_type: format!("{SERVICE_TYPE}_Request"),
                        response_type: format!("{SERVICE_TYPE}_Response"),
                        settings: request_response_settings(),
                    },
                    ros2: RosServiceSettings {
                        service: config::ServiceName::new(ROS_SERVICE)
                            .expect("a valid service name"),
                        type_name: TypeName::new(SERVICE_TYPE).expect("a valid type name"),
                        qos: QosProfile::default(),
                    },
                }),
            ],
        })
        .expect("a valid config")
    }

    #[test]
    fn a_topic_with_the_entrys_qos_maps_to_its_service() {
        let sut = mapping();

        let local = sut.local(&topic_settings(listed(QosProfile::default())));

        assert_that!(local, eq Ok(Some(service_settings(publish_subscribe_settings()))));
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

        let remote = sut.remote(&service_settings(publish_subscribe_settings()));

        assert_that!(remote, eq Ok(Some(topic_settings(QosProfile::default()))));
    }

    #[test]
    fn a_service_with_other_settings_is_refused() {
        let sut = mapping();

        let remote = sut.remote(&service_settings(other_publish_subscribe_settings()));

        assert_that!(
            remote,
            eq Err(Mismatch::Settings {
                entry: PatternSettings::PublishSubscribe(publish_subscribe_settings()),
                local: PatternSettings::PublishSubscribe(other_publish_subscribe_settings()),
            })
        );
    }

    #[test]
    fn a_ros_service_with_the_entrys_qos_maps_to_its_service() {
        let sut = mapping();

        let local = sut.local(&ros_service_settings(QosProfile::default()));

        assert_that!(
            local,
            eq Ok(Some(request_response_service_settings(request_response_settings())))
        );
    }

    #[test]
    fn a_request_response_service_with_the_entrys_settings_maps_to_its_ros_service() {
        let sut = mapping();

        let remote = sut.remote(&request_response_service_settings(
            request_response_settings(),
        ));

        assert_that!(remote, eq Ok(Some(ros_service_settings(QosProfile::default()))));
    }

    #[test]
    fn a_request_response_service_with_other_settings_is_refused() {
        let sut = mapping();

        let remote = sut.remote(&request_response_service_settings(
            other_request_response_settings(),
        ));

        assert_that!(
            remote,
            eq Err(Mismatch::Settings {
                entry: PatternSettings::RequestResponse(request_response_settings()),
                local: PatternSettings::RequestResponse(other_request_response_settings()),
            })
        );
    }

    #[test]
    fn a_service_with_another_pattern_than_its_entry_is_refused() {
        let sut = mapping();

        let remote = sut.remote(&ServiceSettings::new(
            ServiceName::new(SERVICE).expect("a valid service name"),
            PatternSettings::RequestResponse(request_response_settings()),
        ));

        assert_that!(
            remote,
            eq Err(Mismatch::Settings {
                entry: PatternSettings::PublishSubscribe(publish_subscribe_settings()),
                local: PatternSettings::RequestResponse(request_response_settings()),
            })
        );
    }

    #[test]
    fn what_no_entry_covers_is_out_of_scope() {
        let sut = mapping();

        let local = sut.local(&EndpointSettings {
            name: EndpointName::Topic(TopicName::new("/elsewhere").expect("a valid topic name")),
            qos: QosProfile::default(),
        });
        let remote = sut.remote(&ServiceSettings::new(
            ServiceName::new("static/elsewhere").expect("a valid service name"),
            PatternSettings::PublishSubscribe(publish_subscribe_settings()),
        ));

        assert_that!(local, eq Ok(None));
        assert_that!(remote, eq Ok(None));
    }
}
