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
use iceoryx2_integrations_ros2_link_adapter::mapping::static_mapping::{
    Config as StaticConfig, Entry, IceoryxSettings, RosSettings,
};
use iceoryx2_integrations_ros2_link_adapter::qos::Durability;
use iceoryx2_integrations_ros2_link_adapter::{QosProfile, StaticMapping, TopicName, TypeName};
use iceoryx2_link_backend::service_description::PublishSubscribeSettings;
use iceoryx2_link_conformance_tests::parameters::PublishSubscribeName;

use super::MappingUnderTest;

/// Tests the static mapping with a single entry. Every service it generates
/// is the one that entry defines.
pub struct StaticMapped;

impl StaticMapped {
    const SERVICE: &str = "static/chatter";
    const TOPIC: &str = "/static_chatter";
}

impl MappingUnderTest for StaticMapped {
    type Mapping = StaticMapping;

    fn mapping(payload_type: &str) -> StaticMapping {
        StaticMapping::new(StaticConfig {
            entries: vec![Entry {
                iceoryx2: IceoryxSettings {
                    service_name: Self::service_name(),
                    payload_type: payload_type.to_string(),
                    settings: PublishSubscribeSettings::from_config(
                        &iceoryx2::config::Config::default(),
                    ),
                },
                ros2: RosSettings {
                    topic: TopicName::new(Self::TOPIC).expect("a valid topic name"),
                    type_name: TypeName::new(payload_type).expect("a valid type name"),
                    // Transient local durability so messsages sent before matching completes
                    // are kept.
                    qos: QosProfile {
                        durability: Durability::TransientLocal,
                        ..QosProfile::default()
                    },
                },
            }],
        })
        .expect("a valid static mapping")
    }
}

impl PublishSubscribeName for StaticMapped {
    fn service_name() -> ServiceName {
        ServiceName::new(Self::SERVICE).expect("a valid service name")
    }
}
