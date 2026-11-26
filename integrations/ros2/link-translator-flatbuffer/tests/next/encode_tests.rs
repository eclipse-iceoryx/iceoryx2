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
use iceoryx2_integrations_ros2_link_translator_flatbuffer::next::{
    self as translator, EncodeError,
};
use iceoryx2_link_backend::wire::region::Region;
use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::geometry_msgs::msg::rmw::{
    Point, Pose, PoseWithCovariance, PoseWithCovarianceStamped, Quaternion,
};
use ros_env::sensor_msgs::msg::rmw::ChannelFloat32;
use ros_env::std_msgs::msg::rmw::{Header, MultiArrayDimension, UInt8};

use crate::next::testing::flatbuffers::{
    ToFlatbuffer, empty_flatbuffer, header_flatbuffer_with_a_struct_stamp,
    header_flatbuffer_without_a_stamp, pose_with_covariance_flatbuffer,
};
use crate::next::testing::messages;
use crate::next::testing::messages::{
    pose_stamped_message, pose_with_covariance_message, pose_with_covariance_stamped_message,
};
use crate::next::testing::{compile, encode, serialize};
use crate::schemas;
use crate::schemas::uint8_enum::std_msgs::msg as uint8_enum;

#[test]
fn matches_the_rmw_for_scalars() {
    let compiled = compile(schemas::TIME, messages::TIME).expect("compiles");
    let message = Time {
        sec: 1_700_000_000,
        nanosec: 123_456_789,
    };

    let mut encoded_bytes = encode(&compiled, &message.to_flatbuffer());

    let rmw_bytes = serialize(&message);
    // The rmw reports its size estimate as the length and zero-fills past
    // the message.
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn matches_the_rmw_for_fields_declared_in_another_order() {
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

    let mut encoded_bytes = encode(&compiled, &message.to_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn writes_absent_fields_as_their_defaults() {
    const STRIDE_DEFAULT: u32 = 1;

    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");
    let message = MultiArrayDimension {
        label: "".into(),
        size: 0,
        stride: STRIDE_DEFAULT,
    };

    let mut encoded_bytes = encode(&compiled, &empty_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn refuses_a_flatbuffer_without_a_root_offset() {
    const NO_ROOT_OFFSET: [u8; 2] = [0; 2];

    let compiled = compile(schemas::TIME, messages::TIME).expect("compiles");
    let mut cdr: Vec<u8> = Vec::new();

    let error = translator::encode(&compiled, &NO_ROOT_OFFSET, cdr.forward()).expect_err("refuses");

    assert_that!(error, eq EncodeError::Truncated);
}

#[test]
fn matches_the_rmw_for_a_nested_table() {
    let compiled = compile(schemas::HEADER, messages::HEADER).expect("compiles");
    let message = Header {
        stamp: Time {
            sec: 1_700_000_000,
            nanosec: 123_456_789,
        },
        frame_id: "base_link".into(),
    };

    let mut encoded_bytes = encode(&compiled, &message.to_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn writes_an_absent_nested_table_as_its_defaults() {
    let compiled = compile(schemas::HEADER, messages::HEADER).expect("compiles");
    let message = Header {
        stamp: Time { sec: 0, nanosec: 0 },
        frame_id: "base_link".into(),
    };

    let mut encoded_bytes = encode(&compiled, &header_flatbuffer_without_a_stamp(&message));

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn matches_the_rmw_for_a_struct() {
    let compiled = compile(schemas::HEADER_STRUCT, messages::HEADER).expect("compiles");
    let message = Header {
        stamp: Time {
            sec: 1_700_000_000,
            nanosec: 123_456_789,
        },
        frame_id: "base_link".into(),
    };

    let mut encoded_bytes = encode(&compiled, &header_flatbuffer_with_a_struct_stamp(&message));

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn writes_an_absent_struct_as_zeros() {
    let compiled = compile(schemas::HEADER_STRUCT, messages::HEADER).expect("compiles");
    let message = Header {
        stamp: Time { sec: 0, nanosec: 0 },
        frame_id: "base_link".into(),
    };

    let mut encoded_bytes = encode(&compiled, &header_flatbuffer_without_a_stamp(&message));

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn matches_the_rmw_for_a_struct_inside_a_struct() {
    let compiled = compile(schemas::POSE_STAMPED, messages::POSE_STAMPED).expect("compiles");
    let message = pose_stamped_message();

    let mut encoded_bytes = encode(&compiled, &message.to_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn matches_the_rmw_for_a_scalar_sequence() {
    let compiled = compile(schemas::CHANNEL_FLOAT32, messages::CHANNEL_FLOAT32).expect("compiles");
    let message = ChannelFloat32 {
        name: "intensity".into(),
        values: vec![0.5, -1.25, 3.0].into(),
    };

    let mut encoded_bytes = encode(&compiled, &message.to_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn writes_an_absent_vector_as_an_empty_sequence() {
    let compiled = compile(schemas::CHANNEL_FLOAT32, messages::CHANNEL_FLOAT32).expect("compiles");
    let message = ChannelFloat32 {
        name: "".into(),
        values: vec![].into(),
    };

    let mut encoded_bytes = encode(&compiled, &empty_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn matches_the_rmw_for_a_scalar_array() {
    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE,
        messages::POSE_WITH_COVARIANCE,
    )
    .expect("compiles");
    let message = pose_with_covariance_message();

    let mut encoded_bytes = encode(&compiled, &message.to_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn writes_an_absent_vector_as_an_array_of_zeros() {
    const COVARIANCE_NUMBER_OF_ELEMENTS: usize = 36;

    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE,
        messages::POSE_WITH_COVARIANCE,
    )
    .expect("compiles");
    let message = PoseWithCovariance {
        pose: Pose {
            position: Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            orientation: Quaternion {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            },
        },
        covariance: [0.0; COVARIANCE_NUMBER_OF_ELEMENTS],
    };

    let mut encoded_bytes = encode(&compiled, &empty_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn refuses_a_vector_with_another_number_of_elements() {
    const COVARIANCE_NUMBER_OF_ELEMENTS: usize = 36;
    const TOO_FEW_ELEMENTS: [f64; 3] = [1.0, 2.0, 3.0];

    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE,
        messages::POSE_WITH_COVARIANCE,
    )
    .expect("compiles");
    let flatbuffer =
        pose_with_covariance_flatbuffer(&pose_with_covariance_message().pose, &TOO_FEW_ELEMENTS);
    let mut cdr: Vec<u8> = Vec::new();

    let error = translator::encode(&compiled, &flatbuffer, cdr.forward()).expect_err("refuses");

    assert_that!(
        error,
        eq EncodeError::ArrayLength {
            expected: COVARIANCE_NUMBER_OF_ELEMENTS,
            actual: TOO_FEW_ELEMENTS.len(),
        }
    );
}

#[test]
fn matches_the_rmw_for_an_array_inside_a_struct() {
    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE_STAMPED,
        messages::POSE_WITH_COVARIANCE_STAMPED,
    )
    .expect("compiles");
    let message = pose_with_covariance_stamped_message();

    let mut encoded_bytes = encode(&compiled, &message.to_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn writes_an_array_inside_an_absent_struct_as_zeros() {
    const COVARIANCE_NUMBER_OF_ELEMENTS: usize = 36;

    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE_STAMPED,
        messages::POSE_WITH_COVARIANCE_STAMPED,
    )
    .expect("compiles");
    let message = PoseWithCovarianceStamped {
        header: Header {
            stamp: Time { sec: 0, nanosec: 0 },
            frame_id: "".into(),
        },
        pose: PoseWithCovariance {
            pose: Pose {
                position: Point {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                orientation: Quaternion {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    w: 0.0,
                },
            },
            covariance: [0.0; COVARIANCE_NUMBER_OF_ELEMENTS],
        },
    };

    let mut encoded_bytes = encode(&compiled, &empty_flatbuffer());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}

#[test]
fn writes_an_enum_as_its_integer() {
    let compiled = compile(schemas::UINT8_ENUM, messages::UINT8).expect("compiles");
    let message = UInt8 {
        data: uint8_enum::Level::High.0,
    };
    let mut builder = FlatBufferBuilder::new();
    let root = uint8_enum::UInt8::create(
        &mut builder,
        &uint8_enum::UInt8Args {
            data: uint8_enum::Level::High,
        },
    );
    builder.finish_minimal(root);

    let mut encoded_bytes = encode(&compiled, builder.finished_data());

    let rmw_bytes = serialize(&message);
    assert_that!(encoded_bytes.len(), le rmw_bytes.len());
    encoded_bytes.resize(rmw_bytes.len(), 0);
    assert_that!(encoded_bytes, eq rmw_bytes);
}
