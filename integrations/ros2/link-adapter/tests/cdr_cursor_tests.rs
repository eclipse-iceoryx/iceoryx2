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

use core::ops::{Deref, DerefMut};

use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_adapter::cdr::{
    Endianness, ReadError, Reader, WriteError, Writer,
};
use iceoryx2_link_backend::wire::region::{ForwardRegion, Region, ResizeError};
use ros_env::builtin_interfaces::msg::rmw::{Duration, Time};
use ros_env::geometry_msgs::msg::rmw::{Point, Pose, PoseStamped, PoseWithCovariance, Quaternion};
use ros_env::sensor_msgs::msg::rmw::JointState;
use ros_env::std_msgs::msg::rmw::{
    Bool, Float32, Float64, Header, Int8, Int64, MultiArrayDimension, MultiArrayLayout,
    String as RosString, UInt8MultiArray, UInt16,
};
use ros_env::trajectory_msgs::msg::rmw::{JointTrajectory, JointTrajectoryPoint};

use crate::fixture::{deserialize, serialize};

const ENCAPSULATION_LE: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
const ENCAPSULATION_BE: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

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

fn empty_joint_state() -> JointState {
    JointState {
        header: Header {
            stamp: Time { sec: 0, nanosec: 0 },
            frame_id: "".into(),
        },
        name: vec![].into(),
        position: vec![].into(),
        velocity: vec![].into(),
        effort: vec![].into(),
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

fn pose_stamped() -> PoseStamped {
    PoseStamped {
        header: header(),
        pose: pose(),
    }
}

fn pose_with_covariance() -> PoseWithCovariance {
    PoseWithCovariance {
        pose: pose(),
        covariance: core::array::from_fn(|index| index as f64 * 0.5),
    }
}

fn uint8_multi_array() -> UInt8MultiArray {
    UInt8MultiArray {
        layout: MultiArrayLayout {
            dim: vec![
                MultiArrayDimension {
                    label: "rows".into(),
                    size: 2,
                    stride: 6,
                },
                MultiArrayDimension {
                    label: "columns".into(),
                    size: 3,
                    stride: 3,
                },
            ]
            .into(),
            data_offset: 0,
        },
        data: vec![1, 2, 3, 4, 5, 6].into(),
    }
}

fn joint_trajectory() -> JointTrajectory {
    JointTrajectory {
        header: header(),
        joint_names: vec!["shoulder".into(), "elbow".into()].into(),
        points: vec![
            JointTrajectoryPoint {
                positions: vec![0.1, 0.2].into(),
                velocities: vec![1.0, 2.0].into(),
                accelerations: vec![].into(),
                effort: vec![].into(),
                time_from_start: Duration { sec: 1, nanosec: 0 },
            },
            JointTrajectoryPoint {
                positions: vec![0.3, 0.4].into(),
                velocities: vec![].into(),
                accelerations: vec![3.0, 4.0].into(),
                effort: vec![5.0, 6.0].into(),
                time_from_start: Duration {
                    sec: 2,
                    nanosec: 500_000_000,
                },
            },
        ]
        .into(),
    }
}

fn write_header<R: ForwardRegion>(writer: &mut Writer<R>, header: &Header) {
    writer.write_i32(header.stamp.sec).expect("writes");
    writer.write_u32(header.stamp.nanosec).expect("writes");
    writer
        .write_string(header.frame_id.to_string().as_bytes())
        .expect("writes");
}

fn write_joint_state<R: ForwardRegion>(writer: &mut Writer<R>, message: &JointState) {
    write_header(writer, &message.header);
    writer.write_length(message.name.len()).expect("writes");
    for name in message.name.iter() {
        writer
            .write_string(name.to_string().as_bytes())
            .expect("writes");
    }
    write_f64_sequence(writer, &message.position);
    write_f64_sequence(writer, &message.velocity);
    write_f64_sequence(writer, &message.effort);
}

fn write_f64_sequence<R: ForwardRegion>(writer: &mut Writer<R>, values: &[f64]) {
    let elements: Vec<u8> = values
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    writer
        .write_sequence(&elements, size_of::<f64>())
        .expect("writes");
}

fn write_pose_stamped<R: ForwardRegion>(writer: &mut Writer<R>, message: &PoseStamped) {
    write_header(writer, &message.header);
    write_pose(writer, &message.pose);
}

fn write_pose_with_covariance<R: ForwardRegion>(
    writer: &mut Writer<R>,
    message: &PoseWithCovariance,
) {
    write_pose(writer, &message.pose);
    let elements: Vec<u8> = message
        .covariance
        .iter()
        .flat_map(|value| value.to_ne_bytes())
        .collect();
    writer
        .write_array(&elements, size_of::<f64>())
        .expect("writes");
}

fn write_uint8_multi_array<R: ForwardRegion>(writer: &mut Writer<R>, message: &UInt8MultiArray) {
    writer
        .write_length(message.layout.dim.len())
        .expect("writes");
    for dimension in message.layout.dim.iter() {
        writer
            .write_string(dimension.label.to_string().as_bytes())
            .expect("writes");
        writer.write_u32(dimension.size).expect("writes");
        writer.write_u32(dimension.stride).expect("writes");
    }
    writer
        .write_u32(message.layout.data_offset)
        .expect("writes");
    writer
        .write_sequence(&message.data, size_of::<u8>())
        .expect("writes");
}

fn write_joint_trajectory<R: ForwardRegion>(writer: &mut Writer<R>, message: &JointTrajectory) {
    write_header(writer, &message.header);
    writer
        .write_length(message.joint_names.len())
        .expect("writes");
    for name in message.joint_names.iter() {
        writer
            .write_string(name.to_string().as_bytes())
            .expect("writes");
    }
    writer.write_length(message.points.len()).expect("writes");
    for point in message.points.iter() {
        write_f64_sequence(writer, &point.positions);
        write_f64_sequence(writer, &point.velocities);
        write_f64_sequence(writer, &point.accelerations);
        write_f64_sequence(writer, &point.effort);
        writer.write_i32(point.time_from_start.sec).expect("writes");
        writer
            .write_u32(point.time_from_start.nanosec)
            .expect("writes");
    }
}

fn write_pose<R: ForwardRegion>(writer: &mut Writer<R>, pose: &Pose) {
    write_point(writer, &pose.position);
    write_quaternion(writer, &pose.orientation);
}

fn write_point<R: ForwardRegion>(writer: &mut Writer<R>, point: &Point) {
    writer.write_f64(point.x).expect("writes");
    writer.write_f64(point.y).expect("writes");
    writer.write_f64(point.z).expect("writes");
}

fn write_quaternion<R: ForwardRegion>(writer: &mut Writer<R>, quaternion: &Quaternion) {
    writer.write_f64(quaternion.x).expect("writes");
    writer.write_f64(quaternion.y).expect("writes");
    writer.write_f64(quaternion.z).expect("writes");
    writer.write_f64(quaternion.w).expect("writes");
}

fn read_header(reader: &mut Reader<'_>) -> Header {
    let sec = reader.read_i32().expect("reads");
    let nanosec = reader.read_u32().expect("reads");
    let frame_id = reader.read_string().expect("reads");
    Header {
        stamp: Time { sec, nanosec },
        frame_id: str::from_utf8(frame_id)
            .expect("the frame id is UTF-8")
            .into(),
    }
}

fn read_f64_sequence(reader: &mut Reader<'_>) -> Vec<f64> {
    let bytes = reader.read_sequence(size_of::<f64>()).expect("reads");
    assert_that!(reader.endianness(), eq Endianness::Little);
    f64_elements(bytes)
}

fn f64_elements(bytes: &[u8]) -> Vec<f64> {
    let (chunks, remainder) = bytes.as_chunks::<{ size_of::<f64>() }>();
    assert_that!(remainder, len 0);
    chunks.iter().copied().map(f64::from_le_bytes).collect()
}

fn read_joint_state(reader: &mut Reader<'_>) -> JointState {
    let header = read_header(reader);
    let number_of_names = reader.read_length().expect("reads");
    let name = (0..number_of_names)
        .map(|_| {
            let name = reader.read_string().expect("reads");
            str::from_utf8(name).expect("a name is UTF-8").into()
        })
        .collect();
    let position = read_f64_sequence(reader).into();
    let velocity = read_f64_sequence(reader).into();
    let effort = read_f64_sequence(reader).into();
    JointState {
        header,
        name,
        position,
        velocity,
        effort,
    }
}

fn read_pose_stamped(reader: &mut Reader<'_>) -> PoseStamped {
    let header = read_header(reader);
    let pose = read_pose(reader);
    PoseStamped { header, pose }
}

fn read_pose_with_covariance(reader: &mut Reader<'_>) -> PoseWithCovariance {
    const NUMBER_OF_ELEMENTS: usize = 36;
    let pose = read_pose(reader);
    let bytes = reader
        .read_array(NUMBER_OF_ELEMENTS, size_of::<f64>())
        .expect("reads");
    assert_that!(reader.endianness(), eq Endianness::Little);
    let covariance = f64_elements(bytes)
        .try_into()
        .expect("the covariance has 36 elements");
    PoseWithCovariance { pose, covariance }
}

fn read_uint8_multi_array(reader: &mut Reader<'_>) -> UInt8MultiArray {
    let number_of_dimensions = reader.read_length().expect("reads");
    let dim = (0..number_of_dimensions)
        .map(|_| {
            let label = reader.read_string().expect("reads");
            let size = reader.read_u32().expect("reads");
            let stride = reader.read_u32().expect("reads");
            MultiArrayDimension {
                label: str::from_utf8(label).expect("a label is UTF-8").into(),
                size,
                stride,
            }
        })
        .collect();
    let data_offset = reader.read_u32().expect("reads");
    let data = reader.read_sequence(size_of::<u8>()).expect("reads").into();
    UInt8MultiArray {
        layout: MultiArrayLayout { dim, data_offset },
        data,
    }
}

fn read_joint_trajectory(reader: &mut Reader<'_>) -> JointTrajectory {
    let header = read_header(reader);
    let number_of_names = reader.read_length().expect("reads");
    let joint_names = (0..number_of_names)
        .map(|_| {
            let name = reader.read_string().expect("reads");
            str::from_utf8(name).expect("a name is UTF-8").into()
        })
        .collect();
    let number_of_points = reader.read_length().expect("reads");
    let points = (0..number_of_points)
        .map(|_| {
            let positions = read_f64_sequence(reader).into();
            let velocities = read_f64_sequence(reader).into();
            let accelerations = read_f64_sequence(reader).into();
            let effort = read_f64_sequence(reader).into();
            let sec = reader.read_i32().expect("reads");
            let nanosec = reader.read_u32().expect("reads");
            JointTrajectoryPoint {
                positions,
                velocities,
                accelerations,
                effort,
                time_from_start: Duration { sec, nanosec },
            }
        })
        .collect();
    JointTrajectory {
        header,
        joint_names,
        points,
    }
}

fn read_pose(reader: &mut Reader<'_>) -> Pose {
    let position = read_point(reader);
    let orientation = read_quaternion(reader);
    Pose {
        position,
        orientation,
    }
}

fn read_point(reader: &mut Reader<'_>) -> Point {
    let x = reader.read_f64().expect("reads");
    let y = reader.read_f64().expect("reads");
    let z = reader.read_f64().expect("reads");
    Point { x, y, z }
}

fn read_quaternion(reader: &mut Reader<'_>) -> Quaternion {
    let x = reader.read_f64().expect("reads");
    let y = reader.read_f64().expect("reads");
    let z = reader.read_f64().expect("reads");
    let w = reader.read_f64().expect("reads");
    Quaternion { x, y, z, w }
}

/// A region refusing to grow beyond its capacity.
struct BoundedBuffer {
    bytes: Vec<u8>,
    capacity: usize,
}

impl Deref for BoundedBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.bytes
    }
}

impl DerefMut for BoundedBuffer {
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl ForwardRegion for BoundedBuffer {
    fn resize(&mut self, len: usize) -> Result<(), ResizeError> {
        if len > self.capacity {
            return Err(ResizeError::Exhausted);
        }
        self.bytes.resize(len, 0);
        Ok(())
    }
}

#[test]
fn writer_matches_the_rmw_for_a_bool() {
    let message = Bool { data: true };
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_bool(message.data).expect("writes");

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
}

#[test]
fn writer_matches_the_rmw_for_an_int8() {
    let message = Int8 { data: i8::MIN };
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_i8(message.data).expect("writes");

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
}

#[test]
fn writer_matches_the_rmw_for_a_uint16() {
    let message = UInt16 { data: u16::MAX };
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_u16(message.data).expect("writes");

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
}

#[test]
fn writer_matches_the_rmw_for_an_int64() {
    let message = Int64 { data: i64::MIN };
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_i64(message.data).expect("writes");

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
}

#[test]
fn writer_matches_the_rmw_for_a_float32() {
    let message = Float32 { data: 1.5 };
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_f32(message.data).expect("writes");

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
}

#[test]
fn writer_matches_the_rmw_for_a_float64() {
    const FLOAT64: f64 = 1.5;
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_f64(FLOAT64).expect("writes");

    let rmw_bytes = serialize(&Float64 { data: FLOAT64 });
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
}

#[test]
fn writer_matches_the_rmw_for_a_string() {
    const STRING: &str = "hello";
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_string(STRING.as_bytes()).expect("writes");

    let rmw_bytes = serialize(&RosString {
        data: STRING.into(),
    });
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
}

#[test]
fn writer_matches_the_rmw_for_a_joint_state() {
    let message = joint_state();
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    write_joint_state(&mut writer, &message);

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
    assert_that!(deserialize::<JointState>(&written_bytes), eq message);
}

#[test]
fn writer_matches_the_rmw_for_empty_sequences() {
    let message = empty_joint_state();
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    write_joint_state(&mut writer, &message);

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
    assert_that!(deserialize::<JointState>(&written_bytes), eq message);
}

#[test]
fn writer_matches_the_rmw_for_a_pose_stamped() {
    let message = pose_stamped();
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    write_pose_stamped(&mut writer, &message);

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
    assert_that!(deserialize::<PoseStamped>(&written_bytes), eq message);
}

#[test]
fn writer_matches_the_rmw_for_a_pose_with_covariance() {
    let message = pose_with_covariance();
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    write_pose_with_covariance(&mut writer, &message);

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
    assert_that!(deserialize::<PoseWithCovariance>(&written_bytes), eq message);
}

#[test]
fn writer_matches_the_rmw_for_a_uint8_multi_array() {
    let message = uint8_multi_array();
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    write_uint8_multi_array(&mut writer, &message);

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
    assert_that!(deserialize::<UInt8MultiArray>(&written_bytes), eq message);
}

#[test]
fn writer_matches_the_rmw_for_a_joint_trajectory() {
    let message = joint_trajectory();
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    write_joint_trajectory(&mut writer, &message);

    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let rmw_bytes = serialize(&message);
    assert_that!(written_bytes.len(), le rmw_bytes.len());
    written_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(written_bytes, eq rmw_bytes);
    assert_that!(deserialize::<JointTrajectory>(&written_bytes), eq message);
}

#[test]
fn writer_pads_to_the_alignment_of_a_primitive() {
    const VALUE: u64 = 0x0102_0304_0506_0708;
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_u8(1).expect("writes");
    writer.write_u64(VALUE).expect("writes");

    let mut expected = ENCAPSULATION_LE.to_vec();
    expected.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]);
    expected.extend_from_slice(&VALUE.to_le_bytes());
    assert_that!(writer.number_of_bytes(), eq expected.len());
    assert_that!(written_bytes, eq expected);
}

#[test]
fn writer_refuses_a_region_that_cannot_grow() {
    let buffer = BoundedBuffer {
        bytes: Vec::new(),
        capacity: ENCAPSULATION_LE.len() + size_of::<u32>(),
    };
    let mut writer = Writer::new(buffer).expect("starts");

    assert_that!(writer.write_u32(1), is_ok);
    assert_that!(writer.write_u8(1), eq Err(WriteError::Region(ResizeError::Exhausted)));
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_bool() {
    let message = Bool { data: true };
    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(reader.read_bool().expect("reads"), eq message.data);
}

#[test]
fn reader_reads_the_rmw_bytes_of_an_int8() {
    let message = Int8 { data: i8::MIN };
    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(reader.read_i8().expect("reads"), eq message.data);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_uint16() {
    let message = UInt16 { data: u16::MAX };
    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(reader.read_u16().expect("reads"), eq message.data);
}

#[test]
fn reader_reads_the_rmw_bytes_of_an_int64() {
    let message = Int64 { data: i64::MIN };
    let rmw_bytes = serialize(&message);
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(reader.read_i64().expect("reads"), eq message.data);
    assert_that!(reader.is_exhausted(), eq true);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_float32() {
    let message = Float32 { data: 1.5 };
    let rmw_bytes = serialize(&message);
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(reader.read_f32().expect("reads"), eq message.data);
    assert_that!(reader.is_exhausted(), eq true);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_float64() {
    const FLOAT64: f64 = 1.5;
    let rmw_bytes = serialize(&Float64 { data: FLOAT64 });
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(reader.endianness(), eq Endianness::Little);
    assert_that!(reader.read_f64(), eq Ok(FLOAT64));
    assert_that!(reader.is_exhausted(), eq true);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_string() {
    const STRING: &str = "hello";
    let rmw_bytes = serialize(&RosString {
        data: STRING.into(),
    });
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(reader.read_string(), eq Ok(STRING.as_bytes()));
    assert_that!(reader.is_exhausted(), eq true);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_joint_state() {
    let message = joint_state();
    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(read_joint_state(&mut reader), eq message);
}

#[test]
fn reader_reads_the_rmw_bytes_of_empty_sequences() {
    let message = empty_joint_state();
    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(read_joint_state(&mut reader), eq message);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_pose_stamped() {
    let message = pose_stamped();
    let rmw_bytes = serialize(&message);
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(read_pose_stamped(&mut reader), eq message);
    assert_that!(reader.is_exhausted(), eq true);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_pose_with_covariance() {
    let message = pose_with_covariance();
    let rmw_bytes = serialize(&message);
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(read_pose_with_covariance(&mut reader), eq message);
    assert_that!(reader.is_exhausted(), eq true);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_uint8_multi_array() {
    let message = uint8_multi_array();
    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(read_uint8_multi_array(&mut reader), eq message);
}

#[test]
fn reader_reads_the_rmw_bytes_of_a_joint_trajectory() {
    let message = joint_trajectory();
    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    let mut reader = Reader::new(&rmw_bytes).expect("starts");

    assert_that!(read_joint_trajectory(&mut reader), eq message);
}

#[test]
fn reader_reads_back_what_the_writer_wrote() {
    const FLOAT64: f64 = 1.5;
    const STRING: &str = "hello";
    const NUMBER_OF_ELEMENTS: usize = 42;
    let mut written_bytes = Vec::new();
    let mut writer = Writer::new(written_bytes.forward()).expect("starts");
    writer.write_bool(true).expect("writes");
    writer.write_u8(u8::MAX).expect("writes");
    writer.write_i8(i8::MIN).expect("writes");
    writer.write_u16(u16::MAX).expect("writes");
    writer.write_i16(i16::MIN).expect("writes");
    writer.write_u32(u32::MAX).expect("writes");
    writer.write_i32(i32::MIN).expect("writes");
    writer.write_u64(u64::MAX).expect("writes");
    writer.write_i64(i64::MIN).expect("writes");
    writer.write_f32(FLOAT64 as f32).expect("writes");
    writer.write_f64(FLOAT64).expect("writes");
    writer.write_length(NUMBER_OF_ELEMENTS).expect("writes");
    writer.write_string(STRING.as_bytes()).expect("writes");

    let mut reader = Reader::new(&written_bytes).expect("starts");
    assert_that!(reader.read_bool(), eq Ok(true));
    assert_that!(reader.read_u8(), eq Ok(u8::MAX));
    assert_that!(reader.read_i8(), eq Ok(i8::MIN));
    assert_that!(reader.read_u16(), eq Ok(u16::MAX));
    assert_that!(reader.read_i16(), eq Ok(i16::MIN));
    assert_that!(reader.read_u32(), eq Ok(u32::MAX));
    assert_that!(reader.read_i32(), eq Ok(i32::MIN));
    assert_that!(reader.read_u64(), eq Ok(u64::MAX));
    assert_that!(reader.read_i64(), eq Ok(i64::MIN));
    assert_that!(reader.read_f32(), eq Ok(FLOAT64 as f32));
    assert_that!(reader.read_f64(), eq Ok(FLOAT64));
    assert_that!(reader.read_length(), eq Ok(NUMBER_OF_ELEMENTS));
    assert_that!(reader.read_string(), eq Ok(STRING.as_bytes()));
    assert_that!(reader.is_exhausted(), eq true);
}

#[test]
fn reader_honors_a_big_endian_header() {
    const FLOAT64: f64 = 1.5;
    let mut message_bytes = ENCAPSULATION_BE.to_vec();
    message_bytes.extend_from_slice(&FLOAT64.to_be_bytes());
    let mut reader = Reader::new(&message_bytes).expect("starts");

    assert_that!(reader.endianness(), eq Endianness::Big);
    assert_that!(reader.read_f64(), eq Ok(FLOAT64));
}

#[test]
fn reader_refuses_other_encapsulations() {
    const PL_CDR_LE: [u8; 4] = [0x00, 0x03, 0x00, 0x00];
    const CDR2_LE: [u8; 4] = [0x00, 0x07, 0x00, 0x00];

    for header in [PL_CDR_LE, CDR2_LE] {
        let reader = Reader::new(&header);
        assert_that!(reader.err(), eq Some(ReadError::UnsupportedEncapsulation));
    }
}

#[test]
fn reader_refuses_a_truncated_header() {
    let reader = Reader::new(&ENCAPSULATION_LE[..3]);

    assert_that!(reader.err(), eq Some(ReadError::Truncated));
}

#[test]
fn reader_refuses_a_truncated_value() {
    let mut message_bytes = ENCAPSULATION_LE.to_vec();
    message_bytes.extend_from_slice(&[1, 2]);
    let mut reader = Reader::new(&message_bytes).expect("starts");

    assert_that!(reader.read_u32(), eq Err(ReadError::Truncated));
}

#[test]
fn reader_refuses_a_truncated_sequence() {
    const FLOAT64: f64 = 1.5;
    let mut message_bytes = ENCAPSULATION_LE.to_vec();
    message_bytes.extend_from_slice(&2u32.to_le_bytes());
    message_bytes.extend_from_slice(&FLOAT64.to_le_bytes());
    let mut reader = Reader::new(&message_bytes).expect("starts");

    assert_that!(reader.read_sequence(size_of::<f64>()), eq Err(ReadError::Truncated));
}

#[test]
fn reader_refuses_a_string_without_nul() {
    const STRING: &str = "hello";
    let mut message_bytes = ENCAPSULATION_LE.to_vec();
    message_bytes.extend_from_slice(&(STRING.len() as u32).to_le_bytes());
    message_bytes.extend_from_slice(STRING.as_bytes());
    let mut reader = Reader::new(&message_bytes).expect("starts");

    assert_that!(reader.read_string(), eq Err(ReadError::InvalidString));
}

#[test]
fn reader_refuses_an_invalid_bool() {
    let mut message_bytes = ENCAPSULATION_LE.to_vec();
    message_bytes.push(2);
    let mut reader = Reader::new(&message_bytes).expect("starts");

    assert_that!(reader.read_bool(), eq Err(ReadError::InvalidBool));
}
