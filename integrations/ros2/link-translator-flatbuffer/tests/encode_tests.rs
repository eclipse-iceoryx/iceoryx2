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

use flatbuffers::{
    FlatBufferBuilder, Push, PushAlignment, TableFinishedWIPOffset, VOffsetT, WIPOffset,
};
use iceoryx2_bb_flatbuffers_generated::schema::ros2::geometry_msgs::msg::{
    POSE_STAMPED, POSE_WITH_COVARIANCE,
};
use iceoryx2_bb_flatbuffers_generated::schema::ros2::sensor_msgs::msg::JOINT_STATE;
use iceoryx2_bb_flatbuffers_generated::schema::ros2::std_msgs::msg::UINT8_ENUM;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_translator_flatbuffer::encode;
use iceoryx2_integrations_ros2_link_translator_flatbuffer::pairing::{Table, compile};
use iceoryx2_link_backend::wire::region::Region;
use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::geometry_msgs::msg::rmw::{Point, Pose, PoseStamped, PoseWithCovariance, Quaternion};
use ros_env::sensor_msgs::msg::rmw::JointState;
use ros_env::std_msgs::msg::rmw::{Header, UInt8};

use crate::testing::{deserialize, serialize};

/// The vtable offset of the `index`th field of a table.
const fn slot(index: u16) -> VOffsetT {
    4 + 2 * index
}

/// The bytes of a flatbuffer struct of `N` bytes aligned to `A`.
struct StructBytes<const N: usize, const A: usize>([u8; N]);

impl<const N: usize, const A: usize> Push for StructBytes<N, A> {
    type Output = Self;

    unsafe fn push(&self, dst: &mut [u8], _written_len: usize) {
        dst[..N].copy_from_slice(&self.0);
    }

    fn alignment() -> PushAlignment {
        PushAlignment::new(A)
    }
}

fn time_bytes(time: &Time) -> StructBytes<8, 4> {
    let mut bytes = [0; 8];
    bytes[..4].copy_from_slice(&time.sec.to_le_bytes());
    bytes[4..].copy_from_slice(&time.nanosec.to_le_bytes());
    StructBytes(bytes)
}

fn point_bytes(point: &Point) -> StructBytes<24, 8> {
    let mut bytes = [0; 24];
    bytes[..8].copy_from_slice(&point.x.to_le_bytes());
    bytes[8..16].copy_from_slice(&point.y.to_le_bytes());
    bytes[16..].copy_from_slice(&point.z.to_le_bytes());
    StructBytes(bytes)
}

fn quaternion_bytes(quaternion: &Quaternion) -> StructBytes<32, 8> {
    let mut bytes = [0; 32];
    bytes[..8].copy_from_slice(&quaternion.x.to_le_bytes());
    bytes[8..16].copy_from_slice(&quaternion.y.to_le_bytes());
    bytes[16..24].copy_from_slice(&quaternion.z.to_le_bytes());
    bytes[24..].copy_from_slice(&quaternion.w.to_le_bytes());
    StructBytes(bytes)
}

fn build_header(
    builder: &mut FlatBufferBuilder<'_>,
    header: &Header,
) -> WIPOffset<TableFinishedWIPOffset> {
    let frame_id = builder.create_string(&header.frame_id.to_string());
    let table = builder.start_table();
    builder.push_slot_always(slot(0), time_bytes(&header.stamp));
    builder.push_slot_always(slot(1), frame_id);
    builder.end_table(table)
}

fn build_joint_state(message: &JointState) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let header = build_header(&mut builder, &message.header);
    let names = message
        .name
        .iter()
        .map(|name| builder.create_string(&name.to_string()))
        .collect::<Vec<_>>();
    let name = builder.create_vector(&names);
    let position = builder.create_vector(message.position.as_slice());
    let velocity = builder.create_vector(message.velocity.as_slice());
    let effort = builder.create_vector(message.effort.as_slice());
    let table = builder.start_table();
    builder.push_slot_always(slot(0), header);
    builder.push_slot_always(slot(1), name);
    builder.push_slot_always(slot(2), position);
    builder.push_slot_always(slot(3), velocity);
    builder.push_slot_always(slot(4), effort);
    let root = builder.end_table(table);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

fn build_pose(
    builder: &mut FlatBufferBuilder<'_>,
    pose: &Pose,
) -> WIPOffset<TableFinishedWIPOffset> {
    let table = builder.start_table();
    builder.push_slot_always(slot(0), point_bytes(&pose.position));
    builder.push_slot_always(slot(1), quaternion_bytes(&pose.orientation));
    builder.end_table(table)
}

fn build_pose_stamped(message: &PoseStamped) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let header = build_header(&mut builder, &message.header);
    let pose = build_pose(&mut builder, &message.pose);
    let table = builder.start_table();
    builder.push_slot_always(slot(0), header);
    builder.push_slot_always(slot(1), pose);
    let root = builder.end_table(table);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

fn build_pose_with_covariance(pose: &Pose, covariance: &[f64]) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let pose = build_pose(&mut builder, pose);
    let covariance = builder.create_vector(covariance);
    let table = builder.start_table();
    builder.push_slot_always(slot(0), pose);
    builder.push_slot_always(slot(1), covariance);
    let root = builder.end_table(table);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

fn build_uint8(data: u8) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let table = builder.start_table();
    builder.push_slot_always(slot(0), data);
    let root = builder.end_table(table);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

fn build_empty_table() -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    let table = builder.start_table();
    let root = builder.end_table(table);
    builder.finish_minimal(root);
    builder.finished_data().to_vec()
}

fn encode(table: &Table, flatbuffer: &[u8]) -> Vec<u8> {
    let mut cdr: Vec<u8> = Vec::new();
    encode::cdr(table, flatbuffer, cdr.forward()).expect("encodes");
    cdr
}

fn header() -> Header {
    Header {
        stamp: Time {
            sec: 1_700_000_000,
            nanosec: 123_456_789,
        },
        frame_id: "base_link".into(),
    }
}

fn joint_state() -> JointState {
    JointState {
        header: header(),
        name: vec!["shoulder".into(), "elbow".into(), "wrist_1".into()].into(),
        position: vec![0.1, -0.2, 0.3].into(),
        velocity: vec![].into(),
        effort: vec![1.0, 2.0, 3.0].into(),
    }
}

fn pose() -> Pose {
    Pose {
        position: Point {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        },
        orientation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        },
    }
}

#[test]
fn encodes_a_joint_state_like_the_rmw() {
    let table = compile(JOINT_STATE, "sensor_msgs/msg/JointState").expect("pairs");
    let message = joint_state();

    let mut encoded_bytes = encode(&table, &build_joint_state(&message));

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
    assert_that!(deserialize::<JointState>(&encoded_bytes), eq message);
}

#[test]
fn encodes_absent_fields_as_their_defaults() {
    let table = compile(JOINT_STATE, "sensor_msgs/msg/JointState").expect("pairs");
    let message = JointState {
        header: Header {
            stamp: Time { sec: 0, nanosec: 0 },
            frame_id: "".into(),
        },
        name: vec![].into(),
        position: vec![].into(),
        velocity: vec![].into(),
        effort: vec![].into(),
    };

    let mut encoded_bytes = encode(&table, &build_empty_table());

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
    assert_that!(deserialize::<JointState>(&encoded_bytes), eq message);
}

#[test]
fn encodes_a_pose_stamped_like_the_rmw() {
    let table = compile(POSE_STAMPED, "geometry_msgs/msg/PoseStamped").expect("pairs");
    let message = PoseStamped {
        header: header(),
        pose: pose(),
    };

    let mut encoded_bytes = encode(&table, &build_pose_stamped(&message));

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
    assert_that!(deserialize::<PoseStamped>(&encoded_bytes), eq message);
}

#[test]
fn encodes_a_fixed_array_like_the_rmw() {
    let table =
        compile(POSE_WITH_COVARIANCE, "geometry_msgs/msg/PoseWithCovariance").expect("pairs");
    let message = PoseWithCovariance {
        pose: pose(),
        covariance: core::array::from_fn(|index| index as f64 * 0.5),
    };

    let mut encoded_bytes = encode(
        &table,
        &build_pose_with_covariance(&message.pose, &message.covariance),
    );

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
    assert_that!(deserialize::<PoseWithCovariance>(&encoded_bytes), eq message);
}

#[test]
fn encodes_an_enum_as_its_integer() {
    const HIGH: u8 = 1;
    let table = compile(UINT8_ENUM, "std_msgs/msg/UInt8").expect("pairs");
    let message = UInt8 { data: HIGH };

    let mut encoded_bytes = encode(&table, &build_uint8(HIGH));

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
    assert_that!(deserialize::<UInt8>(&encoded_bytes), eq message);
}

#[test]
fn refuses_a_vector_of_the_wrong_length_for_a_fixed_array() {
    const COVARIANCE_LEN: usize = 36;
    let table =
        compile(POSE_WITH_COVARIANCE, "geometry_msgs/msg/PoseWithCovariance").expect("pairs");
    let covariance = [1.0, 2.0, 3.0];

    let mut cdr: Vec<u8> = Vec::new();
    let error = encode::cdr(
        &table,
        &build_pose_with_covariance(&pose(), &covariance),
        cdr.forward(),
    )
    .expect_err("refuses");

    assert_that!(
        error,
        eq encode::EncodeError::ArrayLength { expected: COVARIANCE_LEN, actual: covariance.len() }
    );
}

#[test]
fn refuses_a_flatbuffer_without_a_root_offset() {
    let table = compile(UINT8_ENUM, "std_msgs/msg/UInt8").expect("pairs");

    let mut cdr: Vec<u8> = Vec::new();
    let error = encode::cdr(&table, &[0; 2], cdr.forward()).expect_err("refuses");

    assert_that!(error, eq encode::EncodeError::Truncated);
}
