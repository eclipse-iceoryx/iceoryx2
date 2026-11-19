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

use iceoryx2_bb_flatbuffers::SchemaError;
use iceoryx2_bb_flatbuffers_generated::schema::ros2::builtin_interfaces::msg::TIME_TABLE;
use iceoryx2_bb_flatbuffers_generated::schema::ros2::geometry_msgs::msg::POSE_STAMPED;
use iceoryx2_bb_flatbuffers_generated::schema::ros2::sensor_msgs::msg::{
    JOINT_STATE, JOINT_STATE_FLOAT_POSITION, JOINT_STATE_WITHOUT_EFFORT,
};
use iceoryx2_bb_flatbuffers_generated::schema::ros2::std_msgs::msg::UINT8_ENUM;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_translator_flatbuffer::pairing::{
    Default, ElementKind, FieldKind, Refusal, Scalar, StructKind, compile,
};

#[test]
fn a_joint_state_schema_pairs_member_by_member() {
    let table = compile(JOINT_STATE, "sensor_msgs/msg/JointState").expect("pairs");

    assert_that!(table.fields, len 5);
    let FieldKind::Table(header) = &table.fields[0].kind else {
        panic!("the header is a nested message");
    };
    assert_that!(header.fields, len 2);
    let FieldKind::Struct(stamp) = &header.fields[0].kind else {
        panic!("the stamp is a struct");
    };
    assert_that!(stamp.size, eq 8);
    assert_that!(stamp.alignment, eq 4);
    assert_that!(stamp.fields[0].kind, eq StructKind::Scalar(Scalar::I32));
    assert_that!(stamp.fields[1].kind, eq StructKind::Scalar(Scalar::U32));
    assert_that!(header.fields[1].kind, eq FieldKind::String);
    assert_that!(table.fields[1].kind, eq FieldKind::Sequence(ElementKind::String));
    assert_that!(table.fields[2].kind, eq FieldKind::Sequence(ElementKind::Scalar(Scalar::F64)));
    assert_that!(table.fields[4].kind, eq FieldKind::Sequence(ElementKind::Scalar(Scalar::F64)));
}

#[test]
fn fields_are_located_by_their_vtable_offset() {
    const FIRST_VTABLE_OFFSET: u16 = 4;
    const VTABLE_OFFSET_STEP: u16 = 2;
    let table = compile(JOINT_STATE, "sensor_msgs/msg/JointState").expect("pairs");

    for (index, field) in table.fields.iter().enumerate() {
        assert_that!(field.vtable_offset, eq FIRST_VTABLE_OFFSET + index as u16 * VTABLE_OFFSET_STEP);
    }
}

#[test]
fn a_pose_stamped_schema_pairs_structs_within_tables() {
    let table = compile(POSE_STAMPED, "geometry_msgs/msg/PoseStamped").expect("pairs");

    let FieldKind::Table(pose) = &table.fields[1].kind else {
        panic!("the pose is a nested message");
    };
    let FieldKind::Struct(position) = &pose.fields[0].kind else {
        panic!("the position is a struct");
    };
    let FieldKind::Struct(orientation) = &pose.fields[1].kind else {
        panic!("the orientation is a struct");
    };
    assert_that!(position.fields, len 3);
    assert_that!(position.size, eq 24);
    assert_that!(orientation.fields, len 4);
    assert_that!(orientation.fields[3].offset, eq 24);
    assert_that!(orientation.fields[3].kind, eq StructKind::Scalar(Scalar::F64));
}

#[test]
fn an_enum_pairs_with_its_underlying_scalar() {
    let table = compile(UINT8_ENUM, "std_msgs/msg/UInt8").expect("pairs");

    let FieldKind::Scalar { scalar, default } = &table.fields[0].kind else {
        panic!("the data is a scalar");
    };
    assert_that!(*scalar, eq Scalar::U8);
    assert_that!(default.integer, eq 0);
}

#[test]
fn a_table_pairs_with_a_nested_message_too() {
    let table = compile(TIME_TABLE, "builtin_interfaces/msg/Time").expect("pairs");

    assert_that!(table.fields, len 2);
    assert_that!(table.fields[1].kind, eq FieldKind::Scalar {
        scalar: Scalar::U32,
        default: Default { integer: 0, real: 0.0 },
    });
}

#[test]
fn a_member_without_a_field_is_refused_by_path() {
    let error = compile(JOINT_STATE_WITHOUT_EFFORT, "sensor_msgs/msg/JointState")
        .expect_err("the schema lacks the header and the effort");

    assert_that!(error.path, eq "header");
    assert_that!(error.refusal, eq Refusal::MissingField);
}

#[test]
fn a_field_of_another_width_is_refused_by_path() {
    let error = compile(JOINT_STATE_FLOAT_POSITION, "sensor_msgs/msg/JointState")
        .expect_err("the position is a vector of floats");

    assert_that!(error.path, eq "position");
    assert_that!(
        error.refusal,
        eq Refusal::Mismatch { member: "float64[]".into(), field: "[Float]".into() }
    );
}

#[test]
fn bytes_that_are_no_schema_are_refused() {
    let error = compile(b"not a schema", "builtin_interfaces/msg/Time")
        .expect_err("the bytes are no schema");

    assert_that!(error.refusal, eq Refusal::Schema(SchemaError::NotABinarySchema));
}

#[test]
fn a_message_without_introspection_is_refused() {
    let error = compile(TIME_TABLE, "no_such_msgs/msg/Nothing")
        .expect_err("the typesupport does not exist");

    assert_that!(matches!(error.refusal, Refusal::Introspection(_)), eq true);
}
