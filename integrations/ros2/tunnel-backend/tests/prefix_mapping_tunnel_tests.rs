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

use common::{DISCOVERY_RETRY_ATTEMPTS, DISCOVERY_RETRY_PERIOD, RosString, service_name};

use iceoryx2::prelude::*;
use iceoryx2::service::Service as _;
use iceoryx2::service::local::Service;
use iceoryx2::service::static_config::message_type_details::{TypeDetail, TypeVariant};
use iceoryx2::testing::generate_isolated_config;
use iceoryx2_integrations_ros2_tunnel_backend::ros_header::RosHeader;
use iceoryx2_integrations_ros2_tunnel_backend::testing::{TestPeer, Testing};
use iceoryx2_integrations_ros2_tunnel_backend::{Config, PrefixMapping, Ros2Backend, TopicConfig};
use iceoryx2_services_tunnel::Tunnel;
use iceoryx2_services_tunnel_backend::traits::Passthrough;
use iceoryx2_services_tunnel_backend::traits::testing::Testing as _;

#[test]
fn maps_iceoryx_services_onto_ros_topics() {
    let topic = format!(
        "/prefix_mapping_tunnel_tests/outbound_{}",
        std::process::id()
    );

    let config = generate_isolated_config();
    let node = NodeBuilder::new()
        .config(&config)
        .create::<Service>()
        .unwrap();
    let _service = node
        .service_builder(&service_name(&format!("ros2://topics{topic}")))
        .publish_subscribe::<[RosString]>()
        .create()
        .unwrap();

    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, PrefixMapping, Passthrough>>::new()
        .iceoryx_config(config)
        .backend_config(Config::default())
        .polled()
        .create()
        .unwrap();

    let peer = TestPeer::create();
    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            peer.topic_types(&topic)
                .iter()
                .any(|type_name| type_name == "std_msgs/msg/String")
                .then_some(())
                .ok_or("mapped topic not yet on the ROS 2 graph")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("mapped topic did not appear on the ROS 2 graph");
}

#[test]
fn does_not_map_unprefixed_services() {
    let pid = std::process::id();
    let publish_subscribe_topic =
        format!("/prefix_mapping_tunnel_tests/publish_subscribe_topic_{pid}");

    let config = generate_isolated_config();
    let node = NodeBuilder::new()
        .config(&config)
        .create::<Service>()
        .unwrap();
    let _publish_subscribe = node
        .service_builder(&service_name(&format!(
            "ros2://topics{publish_subscribe_topic}"
        )))
        .publish_subscribe::<[RosString]>()
        .create()
        .unwrap();
    let _unprefixed = node
        .service_builder(&service_name(&format!("unprefixed_{pid}")))
        .publish_subscribe::<u64>()
        .create()
        .unwrap();

    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, PrefixMapping, Passthrough>>::new()
        .iceoryx_config(config)
        .backend_config(Config::default())
        .polled()
        .create()
        .unwrap();

    let peer = TestPeer::create();

    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            (!peer.topic_types(&publish_subscribe_topic).is_empty())
                .then_some(())
                .ok_or("publish_subscribe topic not yet on the ROS 2 graph")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("publish_subscribe topic did not appear");

    assert_eq!(tunnel.tunneled_services().len(), 1);
}

#[test]
fn does_not_map_event_services() {
    let pid = std::process::id();
    let publish_subscribe_topic =
        format!("/prefix_mapping_tunnel_tests/publish_subscribe_topic_{pid}");
    let event_topic = format!("/prefix_mapping_tunnel_tests/event_topic_{pid}");

    let config = generate_isolated_config();
    let node = NodeBuilder::new()
        .config(&config)
        .create::<Service>()
        .unwrap();
    let _publish_subscribe = node
        .service_builder(&service_name(&format!(
            "ros2://topics{publish_subscribe_topic}"
        )))
        .publish_subscribe::<[RosString]>()
        .create()
        .unwrap();
    let _event = node
        .service_builder(&service_name(&format!("ros2://topics{event_topic}")))
        .event()
        .create()
        .unwrap();

    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, PrefixMapping, Passthrough>>::new()
        .iceoryx_config(config)
        .backend_config(Config::default())
        .polled()
        .create()
        .unwrap();

    let peer = TestPeer::create();

    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            (!peer.topic_types(&publish_subscribe_topic).is_empty())
                .then_some(())
                .ok_or("publish_subscribe topic not yet on the ROS 2 graph")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("publish_subscribe topic did not appear");

    assert!(peer.topic_types(&event_topic).is_empty());
    assert_eq!(tunnel.tunneled_services().len(), 1);
}

#[test]
fn maps_ros_topics_onto_iceoryx2_services() {
    let topic = format!(
        "/prefix_mapping_tunnel_tests/inbound_{}",
        std::process::id()
    );

    let config = generate_isolated_config();
    let mut tunnel = Tunnel::<Service, Ros2Backend<Service, PrefixMapping, Passthrough>>::new()
        .iceoryx_config(config.clone())
        .backend_config(Config {
            topics: vec![
                TopicConfig::new(&topic, "std_msgs/msg/String").expect("valid topic config"),
            ],
        })
        .polled()
        .create()
        .unwrap();

    let peer = TestPeer::create();
    let _publisher = peer.create_publisher(&topic, "std_msgs/msg/String");

    let name = service_name(&format!("ros2://topics{topic}"));
    Testing::retry(
        || {
            tunnel.discover().expect("tunnel discovery failed");
            Service::details(&name, &config, MessagingPattern::PublishSubscribe)
                .expect("failed to query service details")
                .is_some()
                .then_some(())
                .ok_or("local service not yet created")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("local service for the ROS 2 topic did not appear");

    let details = Service::details(&name, &config, MessagingPattern::PublishSubscribe)
        .expect("failed to query service details")
        .expect("service exists");
    let message_types = details
        .static_details
        .publish_subscribe()
        .message_type_details();
    assert_eq!(*message_types.payload.type_name(), "std_msgs/msg/String");
    assert_eq!(message_types.payload.variant(), TypeVariant::Dynamic);
    assert_eq!(
        message_types.user_header,
        TypeDetail::new::<RosHeader>(TypeVariant::FixedSize)
    );
}
