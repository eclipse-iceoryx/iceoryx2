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

mod common;

use core::time::Duration;

use common::{DISCOVERY_RETRY_ATTEMPTS, DISCOVERY_RETRY_PERIOD, RosString, service_name};

use iceoryx2::prelude::*;
use iceoryx2::service::Service as _;
use iceoryx2::service::local::Service;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2::testing::generate_isolated_config;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_integrations_ros2_tunnel_backend::Config as BackendConfig;
use iceoryx2_integrations_ros2_tunnel_backend::mapping::static_mapping::{
    Config, Entry, IceoryxSettings, RosSettings,
};
use iceoryx2_integrations_ros2_tunnel_backend::ros_header::RosHeader;
use iceoryx2_integrations_ros2_tunnel_backend::testing::{TestPeer, Testing};
use iceoryx2_integrations_ros2_tunnel_backend::{
    Durability, QosProfile, Reliability, Ros2Backend, StaticMapping, TopicName, TypeName,
};
use iceoryx2_services_tunnel::Tunnel;
use iceoryx2_services_tunnel_backend::traits::Passthrough;
use iceoryx2_services_tunnel_backend::traits::testing::Testing as _;
use iceoryx2_services_tunnel_backend::types::service_description::{
    PortSettings, PublishSubscribeSettings,
};

#[test]
fn maps_iceoryx_services_onto_ros_topics() {
    let pid = std::process::id();
    let chatter_topic = format!("/static_mapping_tunnel_tests/chatter_{pid}");
    let chatter_service = format!("Chatter_{pid}");
    let payload_type = "std_msgs/msg/String";

    let iceoryx_config = generate_isolated_config();
    let node = NodeBuilder::new()
        .config(&iceoryx_config)
        .create::<Service>()
        .expect("failed to create node");
    let _service = node
        .service_builder(&service_name(&chatter_service))
        .publish_subscribe::<[RosString]>()
        .create()
        .expect("failed to create service");

    let mapping = StaticMapping::new(Config {
        entries: vec![Entry {
            iceoryx2: IceoryxSettings {
                service_name: service_name(&chatter_service),
                payload_type: payload_type.to_string(),
                settings: PortSettings::LocalDefaults,
            },
            ros2: RosSettings {
                topic: TopicName::new(&chatter_topic).expect("valid topic name"),
                type_name: TypeName::new(payload_type).expect("valid type name"),
                qos: QosProfile::default(),
            },
        }],
    })
    .expect("valid mapping config");

    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, StaticMapping, Passthrough>>::new()
        .iceoryx_config(iceoryx_config)
        .backend_config(BackendConfig::default())
        .mapping(mapping)
        .polled()
        .create()
        .expect("failed to create tunnel");

    let peer = TestPeer::create();
    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            peer.topic_types(&chatter_topic)
                .iter()
                .any(|type_name| type_name == payload_type)
                .then_some(())
                .ok_or("mapped topic not yet on the ROS 2 graph")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("mapped topic did not appear on the ROS 2 graph");
}

#[test]
fn does_not_map_iceoryx_services_without_an_entry() {
    let pid = std::process::id();
    let chatter_topic = format!("/static_mapping_tunnel_tests/chatter_{pid}");
    let chatter_service = format!("Chatter_{pid}");
    let status_topic = format!("/static_mapping_tunnel_tests/status_{pid}");
    let payload_type = "std_msgs/msg/String";

    let iceoryx_config = generate_isolated_config();
    let node = NodeBuilder::new()
        .config(&iceoryx_config)
        .create::<Service>()
        .expect("failed to create node");
    let _chatter = node
        .service_builder(&service_name(&chatter_service))
        .publish_subscribe::<[RosString]>()
        .create()
        .expect("failed to create service");
    let _status = node
        .service_builder(&service_name(&format!("ros2://topics{status_topic}")))
        .publish_subscribe::<[RosString]>()
        .create()
        .expect("failed to create service");

    let mapping = StaticMapping::new(Config {
        entries: vec![Entry {
            iceoryx2: IceoryxSettings {
                service_name: service_name(&chatter_service),
                payload_type: payload_type.to_string(),
                settings: PortSettings::LocalDefaults,
            },
            ros2: RosSettings {
                topic: TopicName::new(&chatter_topic).expect("valid topic name"),
                type_name: TypeName::new(payload_type).expect("valid type name"),
                qos: QosProfile::default(),
            },
        }],
    })
    .expect("valid mapping config");

    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, StaticMapping, Passthrough>>::new()
        .iceoryx_config(iceoryx_config)
        .backend_config(BackendConfig::default())
        .mapping(mapping)
        .polled()
        .create()
        .expect("failed to create tunnel");

    let peer = TestPeer::create();
    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            (!peer.topic_types(&chatter_topic).is_empty())
                .then_some(())
                .ok_or("chatter topic not yet on the ROS 2 graph")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("chatter topic did not appear");

    assert_that!(peer.topic_types(&status_topic), is_empty);
    assert_that!(tunnel.tunneled_services(), len 1);
}

#[test]
fn maps_ros_topics_onto_iceoryx_services() {
    let pid = std::process::id();
    let chatter_topic = format!("/static_mapping_tunnel_tests/chatter_{pid}");
    let chatter_service = format!("Chatter_{pid}");
    let payload_type = "std_msgs/msg/String";

    let mapping = StaticMapping::new(Config {
        entries: vec![Entry {
            iceoryx2: IceoryxSettings {
                service_name: service_name(&chatter_service),
                payload_type: payload_type.to_string(),
                settings: PortSettings::LocalDefaults,
            },
            ros2: RosSettings {
                topic: TopicName::new(&chatter_topic).expect("valid topic name"),
                type_name: TypeName::new(payload_type).expect("valid type name"),
                qos: QosProfile::default(),
            },
        }],
    })
    .expect("valid mapping config");

    let iceoryx_config = generate_isolated_config();
    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, StaticMapping, Passthrough>>::new()
        .iceoryx_config(iceoryx_config.clone())
        .backend_config(BackendConfig {
            topics: mapping.topics(),
        })
        .mapping(mapping)
        .polled()
        .create()
        .expect("failed to create tunnel");

    let peer = TestPeer::create();
    let _publisher = peer.create_publisher(&chatter_topic, payload_type);

    let name = service_name(&chatter_service);
    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            Service::details(&name, &iceoryx_config, MessagingPattern::PublishSubscribe)
                .expect("failed to query service details")
                .is_some()
                .then_some(())
                .ok_or("local service not yet created")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("local service for the ROS 2 topic did not appear");

    let details = Service::details(&name, &iceoryx_config, MessagingPattern::PublishSubscribe)
        .expect("failed to query service details")
        .expect("service exists");
    let message_types = details
        .static_details
        .publish_subscribe()
        .message_type_details();

    assert_that!(*message_types.payload.type_name(), eq payload_type);
    assert_that!(message_types.payload.variant(), eq TypeVariant::Dynamic);
    assert_that!(
        message_types.user_header,
        eq TypeDetail::new::<RosHeader>(TypeVariant::FixedSize)
    );
}

#[test]
fn applies_specified_qos_to_ros_endpoints() {
    const QOS_DEADLINE: Duration = Duration::from_millis(500);

    let pid = std::process::id();
    let chatter_topic = format!("/static_mapping_tunnel_tests/chatter_{pid}");
    let chatter_service = format!("Chatter_{pid}");
    let payload_type = "std_msgs/msg/String";
    let qos = QosProfile {
        reliability: Reliability::BestEffort,
        durability: Durability::TransientLocal,
        deadline: Some(QOS_DEADLINE),
        ..QosProfile::default()
    };

    let iceoryx_config = generate_isolated_config();
    let node = NodeBuilder::new()
        .config(&iceoryx_config)
        .create::<Service>()
        .expect("failed to create node");
    let _service = node
        .service_builder(&service_name(&chatter_service))
        .publish_subscribe::<[RosString]>()
        .create()
        .expect("failed to create service");

    let mapping = StaticMapping::new(Config {
        entries: vec![Entry {
            iceoryx2: IceoryxSettings {
                service_name: service_name(&chatter_service),
                payload_type: payload_type.to_string(),
                settings: PortSettings::LocalDefaults,
            },
            ros2: RosSettings {
                topic: TopicName::new(&chatter_topic).expect("valid topic name"),
                type_name: TypeName::new(payload_type).expect("valid type name"),
                qos,
            },
        }],
    })
    .expect("valid mapping config");

    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, StaticMapping, Passthrough>>::new()
        .iceoryx_config(iceoryx_config)
        .backend_config(BackendConfig::default())
        .mapping(mapping)
        .polled()
        .create()
        .expect("failed to create tunnel");

    let peer = TestPeer::create();
    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            (!peer.publisher_qos(&chatter_topic).is_empty()
                && !peer.subscription_qos(&chatter_topic).is_empty())
            .then_some(())
            .ok_or("tunnel endpoints not yet on the ROS 2 graph")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("tunnel endpoints did not appear on the ROS 2 graph");

    // History/depth is not exchanged over DDS discovery; assert the
    // policies that are.
    let publishers = peer.publisher_qos(&chatter_topic);
    let subscriptions = peer.subscription_qos(&chatter_topic);
    for profile in publishers.iter().chain(subscriptions.iter()) {
        assert_that!(profile.reliability, eq Reliability::BestEffort);
        assert_that!(profile.durability, eq Durability::TransientLocal);
        assert_that!(profile.deadline, eq Some(QOS_DEADLINE));
    }
}

#[test]
fn applies_specified_settings_to_iceoryx_services() {
    const MAX_SUBSCRIBERS: usize = 3;
    const SUBSCRIBER_MAX_BUFFER_SIZE: usize = 5;

    let pid = std::process::id();
    let chatter_topic = format!("/static_mapping_tunnel_tests/chatter_{pid}");
    let chatter_service = format!("Chatter_{pid}");
    let iox_payload_type = "static_mapping_tunnel_tests::LocalString";
    let ros_payload_type = "std_msgs/msg/String";

    // Local settings different from the defaults.
    let mapping = StaticMapping::new(Config {
        entries: vec![Entry {
            iceoryx2: IceoryxSettings {
                service_name: service_name(&chatter_service),
                payload_type: iox_payload_type.to_string(),
                settings: PortSettings::Value(PublishSubscribeSettings {
                    max_subscribers: MAX_SUBSCRIBERS,
                    subscriber_max_buffer_size: SUBSCRIBER_MAX_BUFFER_SIZE,
                    safe_overflow: false,
                    ..PublishSubscribeSettings::default()
                }),
            },
            ros2: RosSettings {
                topic: TopicName::new(&chatter_topic).expect("valid topic name"),
                type_name: TypeName::new(ros_payload_type).expect("valid type name"),
                qos: QosProfile::default(),
            },
        }],
    })
    .expect("valid mapping config");

    let iceoryx_config = generate_isolated_config();
    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, StaticMapping, Passthrough>>::new()
        .iceoryx_config(iceoryx_config.clone())
        .backend_config(BackendConfig {
            topics: mapping.topics(),
        })
        .mapping(mapping)
        .polled()
        .create()
        .expect("failed to create tunnel");

    let peer = TestPeer::create();
    let _publisher = peer.create_publisher(&chatter_topic, ros_payload_type);

    let name = service_name(&chatter_service);
    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            Service::details(&name, &iceoryx_config, MessagingPattern::PublishSubscribe)
                .expect("failed to query service details")
                .is_some()
                .then_some(())
                .ok_or("local service not yet created")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("local service for the ROS 2 topic did not appear");

    let details = Service::details(&name, &iceoryx_config, MessagingPattern::PublishSubscribe)
        .expect("failed to query service details")
        .expect("service exists");
    let static_config = details.static_details.publish_subscribe();

    assert_that!(static_config.max_subscribers(), eq MAX_SUBSCRIBERS);
    assert_that!(
        static_config.subscriber_max_buffer_size(),
        eq SUBSCRIBER_MAX_BUFFER_SIZE
    );
    assert_that!(static_config.has_safe_overflow(), eq false);
    assert_that!(
        *static_config.message_type_details().payload.type_name(),
        eq iox_payload_type
    );
}
