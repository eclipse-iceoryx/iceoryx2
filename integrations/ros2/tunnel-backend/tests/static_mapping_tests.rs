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

use iceoryx2::service::local::Service;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_integrations_ros2_tunnel_backend::mapping::static_mapping::{
    Config, CreationError, Entry, IceoryxSettings, RosSettings,
};
use iceoryx2_integrations_ros2_tunnel_backend::{
    QosProfile, Reliability, StaticMapping, TopicDescription, TopicName, TypeName,
};
use iceoryx2_services_tunnel_backend::traits::Mapping;
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, PortSettings, PublishSubscribeDescription, PublishSubscribeSettings,
    ServiceDescription,
};

fn entry(service_name: &str, topic: &str) -> Entry {
    Entry {
        iceoryx2: IceoryxSettings {
            service_name: service_name.try_into().unwrap(),
            payload_type: "foxglove::PointCloud".to_string(),
            settings: PortSettings::LocalDefaults,
        },
        ros2: RosSettings {
            topic: TopicName::new(topic).unwrap(),
            type_name: TypeName::new("sensor_msgs/msg/PointCloud2").unwrap(),
            qos: QosProfile {
                reliability: Reliability::BestEffort,
                ..QosProfile::default()
            },
        },
    }
}

fn mapping(entries: Vec<Entry>) -> StaticMapping {
    StaticMapping::new(Config { entries }).unwrap()
}

fn topic_description(topic: &str) -> TopicDescription {
    TopicDescription {
        topic: TopicName::new(topic).unwrap(),
        type_name: TypeName::new("sensor_msgs/msg/PointCloud2").unwrap(),
        qos: QosProfile::default(),
    }
}

#[test]
fn validate_config_used_for_examples() {
    let config: Config = toml::from_str(include_str!("../examples/mapping.toml")).unwrap();

    assert_that!(config.entries, is_not_empty);
    assert_that!(StaticMapping::new(config), is_ok);
}

#[test]
fn parses_toml() {
    let config: Config = toml::from_str(
        r#"
        [[mapping]]
        iceoryx2.service_name = "CmdVel"
        iceoryx2.payload_type = "geometry_msgs/msg/Twist"
        ros2.topic = "/cmd_vel"
        ros2.type = "geometry_msgs/msg/Twist"

        [[mapping]]
        iceoryx2.service_name = "LidarFront"
        iceoryx2.payload_type = "foxglove::PointCloud"
        iceoryx2.settings.subscriber_max_buffer_size = 4
        ros2.topic = "/Lidar/Front"
        ros2.type = "sensor_msgs/msg/PointCloud2"
        ros2.qos.reliability = "Reliable"
        ros2.qos.deadline = "500ms"
        "#,
    )
    .unwrap();
    assert_that!(config.entries, len 2);

    let sut = StaticMapping::new(config).unwrap();

    let full = sut
        .local::<Service>(&topic_description("/Lidar/Front"))
        .expect("topic maps to a service");
    assert_that!(full.name.as_str(), eq "LidarFront");
    let details = sut.remote(&full).expect("service maps to a topic");
    assert_that!(details.type_name.as_str(), eq "sensor_msgs/msg/PointCloud2");
    assert_that!(details.qos.reliability, eq Reliability::Reliable);
    assert_that!(
        details.qos.deadline,
        eq Some(core::time::Duration::from_millis(500))
    );

    let minimal = sut
        .local::<Service>(&topic_description("/cmd_vel"))
        .expect("topic maps to a service");
    assert_that!(minimal.name.as_str(), eq "CmdVel");
    assert_that!(
        sut.remote(&minimal).expect("service maps to a topic").qos,
        eq QosProfile::default()
    );
}

#[test]
fn maps_topic_description_to_service_description() {
    let sut = mapping(vec![entry("LidarFront", "/Lidar/Front")]);

    let topic_description = topic_description("/Lidar/Front");
    let service_description = sut
        .local::<Service>(&topic_description)
        .expect("topic maps to a service");

    assert_that!(service_description.name.as_str(), eq "LidarFront");
    assert_that!(sut.remote(&service_description), is_some);

    let round_tripped = sut
        .remote(&service_description)
        .expect("service maps to a topic");
    assert_that!(round_tripped.topic, eq topic_description.topic);
    assert_that!(round_tripped.type_name, eq topic_description.type_name);
}

#[test]
fn rejects_duplicate_service_names_and_topics() {
    let result = StaticMapping::new(Config {
        entries: vec![entry("LidarFront", "/A"), entry("LidarFront", "/B")],
    });
    assert_that!(result.unwrap_err(), eq CreationError::DuplicateServiceName);

    let result = StaticMapping::new(Config {
        entries: vec![entry("A", "/Lidar/Front"), entry("B", "/Lidar/Front")],
    });
    assert_that!(result.unwrap_err(), eq CreationError::DuplicateTopic);
}

#[test]
fn rejects_unmapped_services() {
    let sut = mapping(vec![entry("LidarFront", "/Lidar/Front")]);

    let unmapped = ServiceDescription::new::<Service>(
        "SomethingElse".try_into().unwrap(),
        PatternDescription::PublishSubscribe(PublishSubscribeDescription {
            user_header: (&TypeDetail::new::<()>(TypeVariant::FixedSize)).into(),
            payload: (&TypeDetail::new::<u8>(TypeVariant::FixedSize)).into(),
            settings: PortSettings::LocalDefaults,
        }),
    );

    assert_that!(sut.remote(&unmapped), is_none);
}

#[test]
fn specified_settings_override_defaults() {
    let mut mapping_with_overrides = entry("LidarFront", "/Lidar/Front");
    mapping_with_overrides.iceoryx2.settings = PortSettings::Value(PublishSubscribeSettings {
        subscriber_max_buffer_size: 4,
        safe_overflow: false,
        ..PublishSubscribeSettings::default()
    });
    let sut = mapping(vec![mapping_with_overrides]);

    let PatternDescription::PublishSubscribe(pattern_description) = sut
        .local::<Service>(&topic_description("/Lidar/Front"))
        .expect("topic maps to a service")
        .pattern
    else {
        panic!("expected a publish-subscribe pattern description");
    };

    let defaults = iceoryx2::config::Config::default()
        .defaults
        .publish_subscribe;
    let PortSettings::Value(settings) = pattern_description.settings else {
        panic!("expected explicit settings");
    };

    assert_that!(settings.subscriber_max_buffer_size, eq 4);
    assert_that!(settings.safe_overflow, eq false);
    assert_that!(settings.max_subscribers, eq defaults.max_subscribers);
    assert_that!(settings.history_size, eq defaults.publisher_history_size);
}

#[test]
fn entries_without_settings_apply_defaults() {
    let sut = mapping(vec![entry("LidarFront", "/Lidar/Front")]);

    let PatternDescription::PublishSubscribe(description) = sut
        .local::<Service>(&topic_description("/Lidar/Front"))
        .expect("topic maps to a service")
        .pattern
    else {
        panic!("expected a publish-subscribe pattern description");
    };

    assert_that!(description.settings, eq PortSettings::LocalDefaults);
}

#[test]
fn mapped_topics_can_be_listed() {
    let sut = mapping(vec![entry("A", "/a"), entry("B", "/b")]);

    let topics = sut.topics();

    assert_that!(topics, len 2);
    assert_that!(topics[0].topic.as_str(), eq "/a");
    assert_that!(topics[1].topic.as_str(), eq "/b");
}
