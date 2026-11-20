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
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_adapter::config;
use iceoryx2_integrations_ros2_link_adapter::mapping::static_mapping::{
    Config, Entry, IceoryxPublishSubscribeSettings, IceoryxRequestResponseSettings,
    RosServiceSettings, RosTopicSettings, ServiceEntry, TopicEntry,
};
use iceoryx2_integrations_ros2_link_adapter::{QosProfile, TopicName, TypeName};
use iceoryx2_integrations_ros2_link_translator_flatbuffer::{TypeMismatch, type_mismatches};
use iceoryx2_link_backend::service_description::{
    PublishSubscribeSettings, RequestResponseSettings,
};

const SERVICE_NAME: &str = "JointReadings";
const JOINT_STATE: &str = "iox2::Flatbuffer<sensor_msgs.msg.JointState>";
const JOINT_STATE_TYPE: &str = "sensor_msgs/msg/JointState";
const POSE_STAMPED_TYPE: &str = "geometry_msgs/msg/PoseStamped";
const BARE_MARKER: &str = "iox2::Flatbuffer";
const ADD_TWO_INTS_REQUEST: &str = "iox2::Flatbuffer<example_interfaces.srv.AddTwoInts_Request>";
const ADD_TWO_INTS_RESPONSE: &str = "iox2::Flatbuffer<example_interfaces.srv.AddTwoInts_Response>";
const ADD_TWO_INTS_TYPE: &str = "example_interfaces/srv/AddTwoInts";

fn service_name() -> ServiceName {
    ServiceName::new(SERVICE_NAME).expect("is a service name")
}

fn topic_entry(payload_type: &str, ros2_type: &str) -> Entry {
    Entry::Topic(TopicEntry {
        iceoryx2: IceoryxPublishSubscribeSettings {
            service_name: service_name(),
            payload_type: payload_type.to_string(),
            settings: PublishSubscribeSettings::from_config(&iceoryx2::config::Config::default()),
        },
        ros2: RosTopicSettings {
            topic: TopicName::new("/joint_states").expect("is a topic name"),
            type_name: TypeName::new(ros2_type).expect("is a type name"),
            qos: QosProfile::default(),
        },
    })
}

fn service_entry(request_type: &str, response_type: &str, ros2_type: &str) -> Entry {
    Entry::Service(ServiceEntry {
        iceoryx2: IceoryxRequestResponseSettings {
            service_name: service_name(),
            request_type: request_type.to_string(),
            response_type: response_type.to_string(),
            settings: RequestResponseSettings::from_config(&iceoryx2::config::Config::default()),
        },
        ros2: RosServiceSettings {
            service: config::ServiceName::new("/add_two_ints").expect("is a service name"),
            type_name: TypeName::new(ros2_type).expect("is a type name"),
            qos: QosProfile::default(),
        },
    })
}

fn config(entries: Vec<Entry>) -> Config {
    Config { entries }
}

#[test]
fn an_entry_whose_root_type_names_its_ros2_type_passes() {
    let config = config(vec![topic_entry(JOINT_STATE, JOINT_STATE_TYPE)]);

    let mismatches = type_mismatches(&config);

    assert_that!(mismatches, len 0);
}

#[test]
fn an_entry_whose_root_type_names_another_ros2_type_is_reported() {
    let config = config(vec![topic_entry(JOINT_STATE, POSE_STAMPED_TYPE)]);

    let mismatches = type_mismatches(&config);

    assert_that!(
        mismatches,
        eq vec![TypeMismatch::Disagrees {
            service_name: service_name(),
            payload_type: JOINT_STATE.to_string(),
            ros2_type: POSE_STAMPED_TYPE.to_string(),
        }]
    );
}

#[test]
fn an_entry_whose_payload_type_names_no_root_type_is_reported() {
    let config = config(vec![topic_entry(BARE_MARKER, JOINT_STATE_TYPE)]);

    let mismatches = type_mismatches(&config);

    assert_that!(
        mismatches,
        eq vec![TypeMismatch::RootTypeMissing {
            service_name: service_name(),
        }]
    );
}

#[test]
fn an_entry_whose_payload_is_no_flatbuffer_is_not_checked() {
    let config = config(vec![topic_entry(POSE_STAMPED_TYPE, JOINT_STATE_TYPE)]);

    let mismatches = type_mismatches(&config);

    assert_that!(mismatches, len 0);
}

#[test]
fn a_service_entry_whose_root_types_name_its_ros2_type_passes() {
    let config = config(vec![service_entry(
        ADD_TWO_INTS_REQUEST,
        ADD_TWO_INTS_RESPONSE,
        ADD_TWO_INTS_TYPE,
    )]);

    let mismatches = type_mismatches(&config);

    assert_that!(mismatches, len 0);
}

#[test]
fn a_service_entry_is_reported_per_disagreeing_side() {
    let config = config(vec![service_entry(
        ADD_TWO_INTS_RESPONSE,
        ADD_TWO_INTS_REQUEST,
        ADD_TWO_INTS_TYPE,
    )]);

    let mismatches = type_mismatches(&config);

    assert_that!(
        mismatches,
        eq vec![
            TypeMismatch::Disagrees {
                service_name: service_name(),
                payload_type: ADD_TWO_INTS_RESPONSE.to_string(),
                ros2_type: format!("{ADD_TWO_INTS_TYPE}_Request"),
            },
            TypeMismatch::Disagrees {
                service_name: service_name(),
                payload_type: ADD_TWO_INTS_REQUEST.to_string(),
                ros2_type: format!("{ADD_TWO_INTS_TYPE}_Response"),
            },
        ]
    );
}

#[test]
fn every_entry_is_checked() {
    let config = config(vec![
        topic_entry(JOINT_STATE, JOINT_STATE_TYPE),
        topic_entry(BARE_MARKER, JOINT_STATE_TYPE),
        topic_entry(JOINT_STATE, POSE_STAMPED_TYPE),
    ]);

    let mismatches = type_mismatches(&config);

    assert_that!(mismatches, len 2);
}
