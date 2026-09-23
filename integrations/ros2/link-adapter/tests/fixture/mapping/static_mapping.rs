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
use iceoryx2_integrations_ros2_link_adapter::StaticMapping;
use iceoryx2_integrations_ros2_link_adapter::mapping::static_mapping;
use iceoryx2_link_backend::service_description::PublishSubscribeSettings;
use iceoryx2_link_conformance_tests::parameters::PublishSubscribeName;

use super::MappingUnderTest;
use crate::fixture::PUBLISH_SUBSCRIBE_SETTINGS;

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
        let PublishSubscribeSettings {
            max_subscribers,
            max_publishers,
            max_nodes,
            history_size,
            subscriber_max_buffer_size,
            subscriber_max_borrowed_samples,
            safe_overflow,
        } = PUBLISH_SUBSCRIBE_SETTINGS;

        let service = Self::SERVICE;
        let topic = Self::TOPIC;
        let config = format!(
            r#"
            [[mapping]]
            iceoryx2.service_name = "{service}"
            iceoryx2.payload_type = "{payload_type}"
            iceoryx2.settings.max_publishers = {max_publishers}
            iceoryx2.settings.max_subscribers = {max_subscribers}
            iceoryx2.settings.max_nodes = {max_nodes}
            iceoryx2.settings.history_size = {history_size}
            iceoryx2.settings.subscriber_max_buffer_size = {subscriber_max_buffer_size}
            iceoryx2.settings.subscriber_max_borrowed_samples = {subscriber_max_borrowed_samples}
            iceoryx2.settings.safe_overflow = {safe_overflow}
            ros2.topic = "{topic}"
            ros2.type = "{payload_type}"
            ros2.qos.durability = "TransientLocal"
            "#
        );

        let config: static_mapping::Config =
            toml::from_str(&config).expect("a valid static mapping config");
        StaticMapping::new(config).expect("a valid static mapping")
    }
}

impl PublishSubscribeName for StaticMapped {
    fn service_name() -> ServiceName {
        ServiceName::new(Self::SERVICE).expect("a valid service name")
    }
}
