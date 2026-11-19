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

use flatbuffers::{FlatBufferBuilder, ForwardsUOffset, Table, VOffsetT, Vector};
use iceoryx2_bb_flatbuffers_generated::schema::ros2::geometry_msgs::msg::{
    POLYGON, POSE_STAMPED, POSE_WITH_COVARIANCE,
};
use iceoryx2_bb_flatbuffers_generated::schema::ros2::sensor_msgs::msg::JOINT_STATE;
use iceoryx2_bb_flatbuffers_generated::schema::ros2::std_msgs::msg::UINT8_ENUM;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_translator_flatbuffer::pairing::{self, compile};
use iceoryx2_integrations_ros2_link_translator_flatbuffer::{decode, encode};
use iceoryx2_link_backend::wire::region::Region;
use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::geometry_msgs::msg::rmw::{
    Point, Point32, Polygon, Pose, PoseStamped, PoseWithCovariance, Quaternion,
};
use ros_env::sensor_msgs::msg::rmw::JointState;
use ros_env::std_msgs::msg::rmw::{Header, UInt8};

use crate::testing::serialize;

/// The vtable offset of the `index`th field of a table.
const fn slot(index: u16) -> VOffsetT {
    4 + 2 * index
}

fn decode(table: &pairing::Table, cdr: &[u8]) -> Vec<u8> {
    let mut builder = FlatBufferBuilder::new();
    decode::flatbuffer(table, cdr, &mut builder).expect("decodes");
    builder.finished_data().to_vec()
}

fn encode(table: &pairing::Table, flatbuffer: &[u8]) -> Vec<u8> {
    let mut cdr: Vec<u8> = Vec::new();
    encode::cdr(table, flatbuffer, cdr.forward()).expect("encodes");
    cdr
}

fn root(flatbuffer: &[u8]) -> Table<'_> {
    // SAFETY: the flatbuffer was built by the decoder.
    unsafe { flatbuffers::root_unchecked::<Table<'_>>(flatbuffer) }
}

fn nested_table<'a>(table: Table<'a>, index: u16) -> Table<'a> {
    // SAFETY: the slot holds a table.
    unsafe { table.get::<ForwardsUOffset<Table<'a>>>(slot(index), None) }.expect("is present")
}

fn string<'a>(table: Table<'a>, index: u16) -> &'a str {
    // SAFETY: the slot holds a string.
    unsafe { table.get::<ForwardsUOffset<&'a str>>(slot(index), None) }.expect("is present")
}

fn f64_vector<'a>(table: Table<'a>, index: u16) -> Vec<f64> {
    // SAFETY: the slot holds a vector of doubles.
    unsafe { table.get::<ForwardsUOffset<Vector<'a, f64>>>(slot(index), None) }
        .expect("is present")
        .iter()
        .collect()
}

/// The bytes of the struct in the slot `index` of `table`.
fn struct_bytes<'a>(table: Table<'a>, index: u16) -> &'a [u8] {
    let offset = table.vtable().get(slot(index)) as usize;
    assert_that!(offset, ne 0);
    &table.buf()[table.loc() + offset..]
}

fn f64_at(bytes: &[u8], offset: usize) -> f64 {
    f64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("8 bytes"))
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
fn decodes_a_joint_state_into_its_flatbuffer() {
    let table = compile(JOINT_STATE, "sensor_msgs/msg/JointState").expect("pairs");
    let message = JointState {
        header: header(),
        name: vec!["shoulder".into(), "elbow".into(), "wrist_1".into()].into(),
        position: vec![0.1, -0.2, 0.3].into(),
        velocity: vec![].into(),
        effort: vec![1.0, 2.0, 3.0].into(),
    };
    let rmw_bytes = serialize(&message);

    let flatbuffer = decode(&table, &rmw_bytes);

    let joint_state = root(&flatbuffer);
    let header = nested_table(joint_state, 0);
    let stamp = struct_bytes(header, 0);
    assert_that!(i32::from_le_bytes(stamp[..4].try_into().expect("4 bytes")), eq message.header.stamp.sec);
    assert_that!(u32::from_le_bytes(stamp[4..8].try_into().expect("4 bytes")), eq message.header.stamp.nanosec);
    assert_that!(string(header, 1), eq "base_link");
    // SAFETY: the slot holds a vector of strings.
    let names = unsafe {
        joint_state.get::<ForwardsUOffset<Vector<'_, ForwardsUOffset<&str>>>>(slot(1), None)
    }
    .expect("is present");
    assert_that!(names.iter().collect::<Vec<_>>(), eq vec!["shoulder", "elbow", "wrist_1"]);
    assert_that!(f64_vector(joint_state, 2), eq vec![0.1, -0.2, 0.3]);
    assert_that!(f64_vector(joint_state, 3), eq Vec::<f64>::new());
    assert_that!(f64_vector(joint_state, 4), eq vec![1.0, 2.0, 3.0]);
}

#[test]
fn decodes_a_pose_stamped_into_its_flatbuffer() {
    let table = compile(POSE_STAMPED, "geometry_msgs/msg/PoseStamped").expect("pairs");
    let message = PoseStamped {
        header: header(),
        pose: pose(),
    };
    let rmw_bytes = serialize(&message);

    let flatbuffer = decode(&table, &rmw_bytes);

    let pose = nested_table(root(&flatbuffer), 1);
    let position = struct_bytes(pose, 0);
    assert_that!(f64_at(position, 0), eq 1.0);
    assert_that!(f64_at(position, 8), eq 2.0);
    assert_that!(f64_at(position, 16), eq 3.0);
    let orientation = struct_bytes(pose, 1);
    assert_that!(f64_at(orientation, 0), eq 0.0);
    assert_that!(f64_at(orientation, 24), eq 1.0);
}

#[test]
fn decodes_a_vector_of_structs() {
    let table = compile(POLYGON, "geometry_msgs/msg/Polygon").expect("pairs");
    let message = Polygon {
        points: vec![
            Point32 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            Point32 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
        ]
        .into(),
    };
    let rmw_bytes = serialize(&message);

    let flatbuffer = decode(&table, &rmw_bytes);

    const POINT32_SIZE: usize = 12;
    // SAFETY: the slot holds a vector, read as bytes to count its elements.
    let points = unsafe { root(&flatbuffer).get::<ForwardsUOffset<Vector<'_, u8>>>(slot(0), None) }
        .expect("is present");
    assert_that!(points.len(), eq 2);
    // The elements follow the length prefix, `Vector<u8>` counts them as bytes.
    let elements = points.bytes().as_ptr() as usize - flatbuffer.as_ptr() as usize;
    let second = &flatbuffer[elements + POINT32_SIZE..elements + 2 * POINT32_SIZE];
    assert_that!(f32::from_le_bytes(second[..4].try_into().expect("4 bytes")), eq 4.0);
    assert_that!(f32::from_le_bytes(second[8..].try_into().expect("4 bytes")), eq 6.0);
}

#[test]
fn omits_a_scalar_at_its_default() {
    let table = compile(UINT8_ENUM, "std_msgs/msg/UInt8").expect("pairs");
    let rmw_bytes = serialize(&UInt8 { data: 0 });

    let flatbuffer = decode(&table, &rmw_bytes);

    assert_that!(root(&flatbuffer).vtable().get(slot(0)), eq 0);
}

#[test]
fn keeps_a_scalar_off_its_default() {
    const HIGH: u8 = 1;
    let table = compile(UINT8_ENUM, "std_msgs/msg/UInt8").expect("pairs");
    let rmw_bytes = serialize(&UInt8 { data: HIGH });

    let flatbuffer = decode(&table, &rmw_bytes);

    // SAFETY: the slot holds a byte.
    let data = unsafe { root(&flatbuffer).get::<u8>(slot(0), Some(0)) };
    assert_that!(data, eq Some(HIGH));
}

#[test]
fn a_decoded_joint_state_encodes_back_to_the_rmw_bytes() {
    let table = compile(JOINT_STATE, "sensor_msgs/msg/JointState").expect("pairs");
    let message = JointState {
        header: header(),
        name: vec!["shoulder".into(), "elbow".into(), "wrist_1".into()].into(),
        position: vec![0.1, -0.2, 0.3].into(),
        velocity: vec![].into(),
        effort: vec![1.0, 2.0, 3.0].into(),
    };
    let rmw_bytes = serialize(&message);

    let mut encoded_bytes = encode(&table, &decode(&table, &rmw_bytes));

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn a_decoded_pose_stamped_encodes_back_to_the_rmw_bytes() {
    let table = compile(POSE_STAMPED, "geometry_msgs/msg/PoseStamped").expect("pairs");
    let message = PoseStamped {
        header: header(),
        pose: pose(),
    };
    let rmw_bytes = serialize(&message);

    let mut encoded_bytes = encode(&table, &decode(&table, &rmw_bytes));

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn a_decoded_fixed_array_encodes_back_to_the_rmw_bytes() {
    let table =
        compile(POSE_WITH_COVARIANCE, "geometry_msgs/msg/PoseWithCovariance").expect("pairs");
    let message = PoseWithCovariance {
        pose: pose(),
        covariance: core::array::from_fn(|index| index as f64 * 0.5),
    };
    let rmw_bytes = serialize(&message);

    let mut encoded_bytes = encode(&table, &decode(&table, &rmw_bytes));

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn a_decoded_vector_of_structs_encodes_back_to_the_rmw_bytes() {
    let table = compile(POLYGON, "geometry_msgs/msg/Polygon").expect("pairs");
    let message = Polygon {
        points: vec![
            Point32 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            Point32 {
                x: 4.0,
                y: 5.0,
                z: 6.0,
            },
            Point32 {
                x: 7.0,
                y: 8.0,
                z: 9.0,
            },
        ]
        .into(),
    };
    let rmw_bytes = serialize(&message);

    let mut encoded_bytes = encode(&table, &decode(&table, &rmw_bytes));

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn refuses_truncated_cdr() {
    const TRUNCATED_LEN: usize = 8;
    let table = compile(JOINT_STATE, "sensor_msgs/msg/JointState").expect("pairs");
    let rmw_bytes = serialize(&JointState::default());

    let mut builder = FlatBufferBuilder::new();
    let error =
        decode::flatbuffer(&table, &rmw_bytes[..TRUNCATED_LEN], &mut builder).expect_err("refuses");

    assert_that!(error, eq decode::Error::Truncated);
}
