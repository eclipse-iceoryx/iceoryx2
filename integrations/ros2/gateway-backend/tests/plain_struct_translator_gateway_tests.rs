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

use common::{DISCOVERY_RETRY_ATTEMPTS, DISCOVERY_RETRY_PERIOD, service_name};

use iceoryx2::prelude::*;
use iceoryx2::service::Service as _;
use iceoryx2::service::local::Service;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2::testing::generate_isolated_config;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_gateway::Gateway;
use iceoryx2_gateway_backend::traits::testing::Testing as _;
use iceoryx2_gateway_backend::types::service_description::PortSettings;
use iceoryx2_integrations_ros2_gateway_backend::Config as BackendConfig;
use iceoryx2_integrations_ros2_gateway_backend::mapping::static_mapping::{
    Config, Entry, IceoryxSettings, RosSettings,
};
use iceoryx2_integrations_ros2_gateway_backend::ros_header::RosHeader;
use iceoryx2_integrations_ros2_gateway_backend::testing::{TestPeer, Testing, take_serialized};
use iceoryx2_integrations_ros2_gateway_backend::{
    PlainStructTranslator, QosProfile, Ros2Backend, StaticMapping, TopicName, TypeName,
};

const TWIST_TYPE_NAME: &str = "geometry_msgs/msg/Twist";

/// Native mirror of `geometry_msgs/msg/Twist`.
#[derive(Debug, Default, Clone, Copy, PartialEq, ZeroCopySend)]
#[type_name("geometry_msgs/msg/Twist")]
#[repr(C)]
struct Twist {
    linear: [f64; 3],
    angular: [f64; 3],
}

impl Twist {
    fn test_value() -> Self {
        Self {
            linear: [1.0, 2.0, 3.0],
            angular: [-1.0, -2.0, -3.0],
        }
    }

    /// The CDR wire encoding of this twist: an encapsulation header followed
    /// by the six little-endian doubles in declaration order.
    #[allow(clippy::wrong_self_convention)]
    fn to_cdr(&self) -> Vec<u8> {
        /// 4-byte CDR encapsulation header (CDR_LE) preceding every wire message.
        const CDR_HEADER: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
        let mut wire = CDR_HEADER.to_vec();
        for value in self.linear.iter().chain(&self.angular) {
            wire.extend_from_slice(&value.to_le_bytes());
        }
        wire
    }
}

fn twist_mapping(topic: &str, service: &str) -> StaticMapping {
    StaticMapping::new(Config {
        entries: vec![Entry {
            iceoryx2: IceoryxSettings {
                service_name: service_name(service),
                payload_type: TWIST_TYPE_NAME.to_string(),
                settings: PortSettings::LocalDefaults,
            },
            ros2: RosSettings {
                topic: TopicName::new(topic).expect("valid topic name"),
                type_name: TypeName::new(TWIST_TYPE_NAME).expect("valid type name"),
                qos: QosProfile::default(),
            },
        }],
    })
    .expect("valid mapping config")
}

#[test]
fn translates_inbound_messages_into_fixed_size_payloads() {
    let pid = std::process::id();
    let topic = format!("/plain_struct_translator_gateway_tests/inbound_{pid}");
    let service = format!("PlainStructInbound_{pid}");

    let iceoryx_config = generate_isolated_config();
    let mapping = twist_mapping(&topic, &service);
    let mut gateway =
        Gateway::<Service, Ros2Backend<Service, StaticMapping, PlainStructTranslator>>::new()
            .iceoryx_config(iceoryx_config.clone())
            .backend_config(BackendConfig {
                topics: mapping.topics(),
            })
            .mapping(mapping)
            .polled()
            .create()
            .expect("failed to create gateway");

    let rcl_peer = TestPeer::create();
    let rcl_publisher = rcl_peer.create_publisher(&topic, TWIST_TYPE_NAME);

    let name = service_name(&service);
    Testing::retry(
        || {
            gateway.discover().expect("gateway discovery failed");
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

    // The local service carries the translator's fixed native layout.
    let details = Service::details(&name, &iceoryx_config, MessagingPattern::PublishSubscribe)
        .expect("failed to query service details")
        .expect("service exists");
    let payload = &details
        .static_details
        .publish_subscribe()
        .message_type_details()
        .payload;
    assert_that!(payload.variant(), eq TypeVariant::FixedSize);
    assert_that!(payload.size(), eq core::mem::size_of::<Twist>());
    assert_that!(payload.alignment(), eq core::mem::align_of::<Twist>());

    let iceoryx_node = NodeBuilder::new()
        .config(&iceoryx_config)
        .create::<Service>()
        .expect("failed to create node");
    let iceoryx_service = iceoryx_node
        .service_builder(&name)
        .publish_subscribe::<Twist>()
        .user_header::<RosHeader>()
        .open()
        .expect("failed to open the bridged service");
    let iceoryx_subscriber = iceoryx_service
        .subscriber_builder()
        .create()
        .expect("failed to create subscriber");

    let cdr_bytes = Twist::test_value().to_cdr();
    Testing::retry(
        || {
            rcl_publisher
                .publish(&cdr_bytes)
                .expect("failed to publish");
            gateway.propagate().expect("gateway propagation failed");
            iceoryx_subscriber
                .receive()
                .expect("failed to receive")
                .is_some_and(|sample| *sample.payload() == Twist::test_value())
                .then_some(())
                .ok_or("translated twist not yet received")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("translated twist did not arrive natively");
}

#[test]
fn translates_outbound_fixed_size_payloads_onto_the_wire() {
    let pid = std::process::id();
    let topic = format!("/plain_struct_translator_gateway_tests/outbound_{pid}");
    let service = format!("PlainStructOutbound_{pid}");

    let iceoryx_config = generate_isolated_config();
    let iceoryx_node = NodeBuilder::new()
        .config(&iceoryx_config)
        .create::<Service>()
        .expect("failed to create node");
    let iceoryx_service = iceoryx_node
        .service_builder(&service_name(&service))
        .publish_subscribe::<Twist>()
        .create()
        .expect("failed to create service");
    let iceoryx_publisher = iceoryx_service
        .publisher_builder()
        .create()
        .expect("failed to create publisher");

    let mut gateway =
        Gateway::<Service, Ros2Backend<Service, StaticMapping, PlainStructTranslator>>::new()
            .iceoryx_config(iceoryx_config)
            .backend_config(BackendConfig::default())
            .mapping(twist_mapping(&topic, &service))
            .polled()
            .create()
            .expect("failed to create gateway");

    let rcl_peer = TestPeer::create();
    let rcl_subscription = rcl_peer.create_subscription(&topic, TWIST_TYPE_NAME);

    let expected = Twist::test_value().to_cdr();
    Testing::retry(
        || {
            iceoryx_publisher
                .send_copy(Twist::test_value())
                .expect("failed to send");
            gateway.discover().expect("gateway discovery failed");
            gateway.propagate().expect("gateway propagation failed");
            take_serialized(&rcl_subscription)
                .is_some_and(|bytes| bytes == expected)
                .then_some(())
                .ok_or("translated twist not yet on the wire")
        },
        DISCOVERY_RETRY_PERIOD,
        Some(DISCOVERY_RETRY_ATTEMPTS),
    )
    .expect("translated twist did not arrive on the wire");
}
