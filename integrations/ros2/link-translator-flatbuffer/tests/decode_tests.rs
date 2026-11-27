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

use flatbuffers::FlatBufferBuilder;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_translator_flatbuffer::{self as translator, DecodeError};
use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::sensor_msgs::msg::rmw::ChannelFloat32;
use ros_env::std_msgs::msg::rmw::{Header, MultiArrayDimension};
use ros_env::test_msgs::msg::rmw::BoundedSequences;

use crate::schemas::arrays::test_msgs::msg as arrays;
use crate::schemas::channel_float32::sensor_msgs::msg as channel_float32;
use crate::schemas::joint_state::sensor_msgs::msg as joint_state;
use crate::schemas::multi_array_dimension::std_msgs::msg as multi_array_dimension;
use crate::schemas::pose_array::geometry_msgs::msg as pose_array;
use crate::schemas::pose_array_struct::geometry_msgs::msg as pose_array_struct;
use crate::schemas::pose_with_covariance::geometry_msgs::msg as pose_with_covariance;
use crate::schemas::pose_with_covariance_stamped::geometry_msgs::msg as pose_with_covariance_stamped;
use crate::schemas::time::builtin_interfaces::msg as time;
use crate::schemas::{self, header, header_struct, point_cloud, pose_stamped};
use crate::testing::messages;
use crate::testing::messages::{
    arrays_message, joint_state_message, point_cloud_message, pose_array_message,
    pose_stamped_message, pose_with_covariance_message, pose_with_covariance_stamped_message,
};
use crate::testing::{compile, decode, encode, serialize};

fn channel_float32_as_big_endian_cdr(message: &ChannelFloat32) -> Vec<u8> {
    const BIG_ENDIAN_HEADER: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
    const LENGTH_ALIGNMENT: usize = 4;

    let name = message.name.to_string();
    let mut cdr = BIG_ENDIAN_HEADER.to_vec();
    cdr.extend_from_slice(&(name.len() as u32 + 1).to_be_bytes());
    cdr.extend_from_slice(name.as_bytes());
    cdr.push(0);
    while !cdr.len().is_multiple_of(LENGTH_ALIGNMENT) {
        cdr.push(0);
    }
    cdr.extend_from_slice(&(message.values.as_slice().len() as u32).to_be_bytes());
    for value in message.values.as_slice() {
        cdr.extend_from_slice(&value.to_be_bytes());
    }
    cdr
}

#[test]
fn reads_the_scalars_of_the_rmw() {
    let compiled = compile(schemas::TIME, messages::TIME).expect("compiles");
    let message = Time {
        sec: 1_700_000_000,
        nanosec: 123_456_789,
    };

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = time::root_as_time(&flatbuffer).expect("is a flatbuffer");
    assert_that!(decoded.sec(), eq message.sec);
    assert_that!(decoded.nanosec(), eq message.nanosec);
}

#[test]
fn writes_fields_declared_in_another_order() {
    const LABEL: &str = "rows";

    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");
    let message = MultiArrayDimension {
        label: LABEL.into(),
        size: 2,
        stride: 6,
    };

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded =
        multi_array_dimension::root_as_multi_array_dimension(&flatbuffer).expect("is a flatbuffer");
    assert_that!(decoded.label(), eq Some(LABEL));
    assert_that!(decoded.size(), eq message.size);
    assert_that!(decoded.stride(), eq message.stride);
}

#[test]
fn omits_a_scalar_at_its_default() {
    const STRIDE_DEFAULT: u32 = 1;
    const ABSENT: u16 = 0;

    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");
    let message = MultiArrayDimension {
        label: "rows".into(),
        size: 2,
        stride: STRIDE_DEFAULT,
    };

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded =
        multi_array_dimension::root_as_multi_array_dimension(&flatbuffer).expect("is a flatbuffer");
    let stride_offset = decoded
        ._tab
        .vtable()
        .get(multi_array_dimension::MultiArrayDimension::VT_STRIDE);
    assert_that!(stride_offset, eq ABSENT);
    assert_that!(decoded.stride(), eq STRIDE_DEFAULT);
}

#[test]
fn encoding_the_decoded_flatbuffer_reproduces_the_rmw_bytes() {
    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");
    let message = MultiArrayDimension {
        label: "rows".into(),
        size: 2,
        stride: 6,
    };
    let rmw_bytes = serialize(&message);

    let mut encoded_bytes = encode(&compiled, &decode(&compiled, &rmw_bytes));

    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn refuses_a_cdr_that_ends_within_a_value() {
    const NUMBER_OF_BYTES_WITHIN_SEC: usize = 6;

    let compiled = compile(schemas::TIME, messages::TIME).expect("compiles");
    let message = Time {
        sec: 1_700_000_000,
        nanosec: 123_456_789,
    };
    let rmw_bytes = serialize(&message);
    let mut builder = FlatBufferBuilder::new();

    let error = translator::decode(
        &compiled,
        &rmw_bytes[..NUMBER_OF_BYTES_WITHIN_SEC],
        &mut builder,
    )
    .expect_err("refuses");

    assert_that!(error, eq DecodeError::Truncated);
}

#[test]
fn writes_a_nested_message_as_a_nested_table() {
    const FRAME_ID: &str = "base_link";

    let compiled = compile(schemas::HEADER, messages::HEADER).expect("compiles");
    let message = Header {
        stamp: Time {
            sec: 1_700_000_000,
            nanosec: 123_456_789,
        },
        frame_id: FRAME_ID.into(),
    };

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = header::std_msgs::msg::root_as_header(&flatbuffer).expect("is a flatbuffer");
    let stamp = decoded.stamp().expect("holds a stamp");
    assert_that!(stamp.sec(), eq message.stamp.sec);
    assert_that!(stamp.nanosec(), eq message.stamp.nanosec);
    assert_that!(decoded.frame_id(), eq Some(FRAME_ID));
}

#[test]
fn writes_a_nested_message_as_a_struct() {
    const FRAME_ID: &str = "base_link";

    let compiled = compile(schemas::HEADER_STRUCT, messages::HEADER).expect("compiles");
    let message = Header {
        stamp: Time {
            sec: 1_700_000_000,
            nanosec: 123_456_789,
        },
        frame_id: FRAME_ID.into(),
    };

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded =
        header_struct::std_msgs::msg::root_as_header(&flatbuffer).expect("is a flatbuffer");
    let stamp = decoded.stamp().expect("holds a stamp");
    assert_that!(stamp.sec(), eq message.stamp.sec);
    assert_that!(stamp.nanosec(), eq message.stamp.nanosec);
    assert_that!(decoded.frame_id(), eq Some(FRAME_ID));
}

#[test]
fn writes_a_struct_inside_a_struct() {
    let compiled = compile(schemas::POSE_STAMPED, messages::POSE_STAMPED).expect("compiles");
    let message = pose_stamped_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = pose_stamped::geometry_msgs::msg::root_as_pose_stamped(&flatbuffer)
        .expect("is a flatbuffer");
    let pose = decoded.pose().expect("holds a pose");
    assert_that!(pose.position().x(), eq message.pose.position.x);
    assert_that!(pose.position().y(), eq message.pose.position.y);
    assert_that!(pose.position().z(), eq message.pose.position.z);
    assert_that!(pose.orientation().x(), eq message.pose.orientation.x);
    assert_that!(pose.orientation().y(), eq message.pose.orientation.y);
    assert_that!(pose.orientation().z(), eq message.pose.orientation.z);
    assert_that!(pose.orientation().w(), eq message.pose.orientation.w);
}

#[test]
fn writes_a_scalar_sequence_as_a_vector() {
    const VALUES: [f32; 3] = [0.5, -1.25, 3.0];

    let compiled = compile(schemas::CHANNEL_FLOAT32, messages::CHANNEL_FLOAT32).expect("compiles");
    let message = ChannelFloat32 {
        name: "intensity".into(),
        values: VALUES.to_vec().into(),
    };

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = channel_float32::root_as_channel_float_32(&flatbuffer).expect("is a flatbuffer");
    let values: Vec<f32> = decoded.values().expect("holds values").iter().collect();
    assert_that!(values, eq VALUES.to_vec());
}

#[test]
fn reads_a_scalar_sequence_from_big_endian_cdr() {
    const VALUES: [f32; 3] = [0.5, -1.25, 3.0];

    let compiled = compile(schemas::CHANNEL_FLOAT32, messages::CHANNEL_FLOAT32).expect("compiles");
    let message = ChannelFloat32 {
        name: "intensity".into(),
        values: VALUES.to_vec().into(),
    };

    let flatbuffer = decode(&compiled, &channel_float32_as_big_endian_cdr(&message));

    let decoded = channel_float32::root_as_channel_float_32(&flatbuffer).expect("is a flatbuffer");
    let values: Vec<f32> = decoded.values().expect("holds values").iter().collect();
    assert_that!(values, eq VALUES.to_vec());
}

#[test]
fn writes_a_scalar_array_as_a_vector() {
    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE,
        messages::POSE_WITH_COVARIANCE,
    )
    .expect("compiles");
    let message = pose_with_covariance_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded =
        pose_with_covariance::root_as_pose_with_covariance(&flatbuffer).expect("is a flatbuffer");
    let covariance: Vec<f64> = decoded
        .covariance()
        .expect("holds a covariance")
        .iter()
        .collect();
    assert_that!(covariance, eq message.covariance.to_vec());
}

#[test]
fn writes_a_scalar_array_as_an_array_inside_a_struct() {
    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE_STAMPED,
        messages::POSE_WITH_COVARIANCE_STAMPED,
    )
    .expect("compiles");
    let message = pose_with_covariance_stamped_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = pose_with_covariance_stamped::root_as_pose_with_covariance_stamped(&flatbuffer)
        .expect("is a flatbuffer");
    let pose = decoded.pose().expect("holds a pose");
    let covariance: Vec<f64> = pose.covariance().iter().collect();
    assert_that!(covariance, eq message.pose.covariance.to_vec());
    assert_that!(pose.pose().orientation().w(), eq message.pose.pose.orientation.w);
}

#[test]
fn writes_a_string_sequence_as_a_vector_of_strings() {
    let compiled = compile(schemas::JOINT_STATE, messages::JOINT_STATE).expect("compiles");
    let message = joint_state_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = joint_state::root_as_joint_state(&flatbuffer).expect("is a flatbuffer");
    let names: Vec<String> = decoded
        .name()
        .expect("holds names")
        .iter()
        .map(str::to_string)
        .collect();
    let message_names: Vec<String> = message
        .name
        .as_slice()
        .iter()
        .map(|name| name.to_string())
        .collect();
    assert_that!(names, eq message_names);
    let effort: Vec<f64> = decoded.effort().expect("holds effort").iter().collect();
    assert_that!(effort, eq message.effort.as_slice().to_vec());
}

#[test]
fn writes_a_message_sequence_as_a_vector_of_tables() {
    const SECOND_POSE: usize = 1;

    let compiled = compile(schemas::POSE_ARRAY, messages::POSE_ARRAY).expect("compiles");
    let message = pose_array_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = pose_array::root_as_pose_array(&flatbuffer).expect("is a flatbuffer");
    let poses = decoded.poses().expect("holds poses");
    assert_that!(poses.len(), eq message.poses.as_slice().len());
    let pose = poses.get(SECOND_POSE);
    let expected = &message.poses.as_slice()[SECOND_POSE];
    assert_that!(pose.position().expect("holds a position").x(), eq expected.position.x);
    assert_that!(pose.orientation().expect("holds an orientation").w(), eq expected.orientation.w);
}

#[test]
fn writes_message_elements_of_varying_size() {
    const SECOND_CHANNEL: usize = 1;

    let compiled = compile(schemas::POINT_CLOUD, messages::POINT_CLOUD).expect("compiles");
    let message = point_cloud_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded =
        point_cloud::sensor_msgs::msg::root_as_point_cloud(&flatbuffer).expect("is a flatbuffer");
    let channels = decoded.channels().expect("holds channels");
    assert_that!(channels.len(), eq message.channels.as_slice().len());
    let channel = channels.get(SECOND_CHANNEL);
    let expected = &message.channels.as_slice()[SECOND_CHANNEL];
    let expected_name = expected.name.to_string();
    assert_that!(channel.name(), eq Some(expected_name.as_str()));
    let values: Vec<f32> = channel.values().expect("holds values").iter().collect();
    assert_that!(values, eq expected.values.as_slice().to_vec());
    let points = decoded.points().expect("holds points");
    assert_that!(points.len(), eq message.points.as_slice().len());
}

#[test]
fn writes_a_message_sequence_as_a_vector_of_structs() {
    const FIRST_POSE: usize = 0;
    const SECOND_POSE: usize = 1;

    let compiled = compile(schemas::POSE_ARRAY_STRUCT, messages::POSE_ARRAY).expect("compiles");
    let message = pose_array_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = pose_array_struct::root_as_pose_array(&flatbuffer).expect("is a flatbuffer");
    let poses = decoded.poses().expect("holds poses");
    let expected = message.poses.as_slice();
    assert_that!(poses.len(), eq expected.len());
    assert_that!(poses.get(FIRST_POSE).position().x(), eq expected[FIRST_POSE].position.x);
    assert_that!(poses.get(FIRST_POSE).orientation().w(), eq expected[FIRST_POSE].orientation.w);
    assert_that!(poses.get(SECOND_POSE).position().x(), eq expected[SECOND_POSE].position.x);
    assert_that!(poses.get(SECOND_POSE).orientation().w(), eq expected[SECOND_POSE].orientation.w);
}

#[test]
fn writes_string_and_message_arrays_as_vectors() {
    const SECOND_ELEMENT: usize = 1;
    const THIRD_ELEMENT: usize = 2;

    let compiled = compile(schemas::ARRAYS, messages::ARRAYS).expect("compiles");
    let message = arrays_message();

    let flatbuffer = decode(&compiled, &serialize(&message));

    let decoded = arrays::root_as_arrays(&flatbuffer).expect("is a flatbuffer");
    let strings = decoded.string_values().expect("holds strings");
    assert_that!(strings.len(), eq message.string_values.len());
    assert_that!(strings.get(THIRD_ELEMENT), eq "third");
    let basic_types = decoded.basic_types_values().expect("holds basic types");
    assert_that!(basic_types.len(), eq message.basic_types_values.len());
    assert_that!(
        basic_types.get(SECOND_ELEMENT).int32_value(),
        eq message.basic_types_values[SECOND_ELEMENT].int32_value
    );
}

#[test]
fn encoding_the_decoded_flatbuffer_reproduces_the_rmw_bytes_of_string_and_message_arrays() {
    let compiled = compile(schemas::ARRAYS, messages::ARRAYS).expect("compiles");
    let rmw_bytes = serialize(&arrays_message());

    let mut encoded_bytes = encode(&compiled, &decode(&compiled, &rmw_bytes));

    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn encoding_the_decoded_flatbuffer_reproduces_the_rmw_bytes_of_bounded_sequences() {
    let compiled = compile(schemas::ARRAYS, messages::BOUNDED_SEQUENCES).expect("compiles");
    let rmw_bytes = serialize(&BoundedSequences::default());

    let mut encoded_bytes = encode(&compiled, &decode(&compiled, &rmw_bytes));

    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}
