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

use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_integrations_ros2_link_translator_flatbuffer::model::{
    Kind, ModelIndex, ScalarType, ScalarValue, ValueIndex,
};
use iceoryx2_integrations_ros2_link_translator_flatbuffer::{Refusal, cdr, flatbuffer};

use crate::schemas::multi_array_dimension::std_msgs::msg as multi_array_dimension;
use crate::schemas::pose_with_covariance_stamped::geometry_msgs::msg as pose_with_covariance_stamped;
use crate::schemas::{self, header, header_struct, pose_stamped};
use crate::testing::compile;
use crate::testing::messages;

#[test]
fn adds_one_value_per_member() {
    let compiled = compile(schemas::TIME, messages::TIME).expect("compiles");

    assert_that!(
        compiled.model.values,
        eq vec![Kind::Scalar(ScalarType::I32), Kind::Scalar(ScalarType::U32)]
    );
}

#[test]
fn tells_strings_from_scalars() {
    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");

    assert_that!(
        compiled.model.values,
        eq vec![Kind::String, Kind::Scalar(ScalarType::U32), Kind::Scalar(ScalarType::U32)]
    );
}

#[test]
fn keeps_the_member_order_for_cdr() {
    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");

    assert_that!(
        compiled.cdr.order,
        eq vec![ValueIndex(0), ValueIndex(1), ValueIndex(2)]
    );
    assert_that!(
        compiled.cdr.members,
        eq vec![
            cdr::MemberEntry { kind: cdr::MemberKind::String },
            cdr::MemberEntry { kind: cdr::MemberKind::Scalar(ScalarType::U32) },
            cdr::MemberEntry { kind: cdr::MemberKind::Scalar(ScalarType::U32) },
        ]
    );
}

#[test]
fn records_the_vtable_offset_of_each_field() {
    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");

    assert_that!(compiled.flatbuffer.fields, len 3);
    assert_that!(compiled.flatbuffer.fields[0].placement.slot().expect("sits in a table").vtable_offset, eq multi_array_dimension::MultiArrayDimension::VT_LABEL);
    assert_that!(compiled.flatbuffer.fields[0].kind, eq flatbuffer::FieldKind::String);
    assert_that!(compiled.flatbuffer.fields[1].placement.slot().expect("sits in a table").vtable_offset, eq multi_array_dimension::MultiArrayDimension::VT_SIZE);
    assert_that!(compiled.flatbuffer.fields[2].placement.slot().expect("sits in a table").vtable_offset, eq multi_array_dimension::MultiArrayDimension::VT_STRIDE);
}

#[test]
fn records_the_default_of_a_scalar_field() {
    const STRIDE_DEFAULT: u32 = 1;

    let compiled = compile(
        schemas::MULTI_ARRAY_DIMENSION,
        messages::MULTI_ARRAY_DIMENSION,
    )
    .expect("compiles");

    assert_that!(
        compiled.flatbuffer.fields[1].kind,
        eq flatbuffer::FieldKind::Scalar { scalar: ScalarType::U32, default: ScalarValue::U32(0) }
    );
    assert_that!(
        compiled.flatbuffer.fields[2].kind,
        eq flatbuffer::FieldKind::Scalar {
            scalar: ScalarType::U32,
            default: ScalarValue::U32(STRIDE_DEFAULT),
        }
    );
}

#[test]
fn refuses_a_member_without_a_field() {
    let error = compile(schemas::TIME_RENAMED, messages::TIME).expect_err("refuses");

    assert_that!(error.path, eq "nanosec");
    assert_that!(error.refusal, eq Refusal::MissingField);
}

#[test]
fn refuses_a_field_of_another_kind() {
    let error = compile(schemas::TIME_MISMATCHED, messages::TIME).expect_err("refuses");

    assert_that!(error.path, eq "nanosec");
    assert_that!(
        error.refusal,
        eq Refusal::Mismatch { member: "uint32".into(), field: "Bool".into() }
    );
}

#[test]
fn adds_the_values_of_a_nested_message() {
    let compiled = compile(schemas::HEADER, messages::HEADER).expect("compiles");

    assert_that!(
        compiled.model.values,
        eq vec![Kind::Scalar(ScalarType::I32), Kind::Scalar(ScalarType::U32), Kind::String]
    );
}

#[test]
fn records_which_table_holds_each_value() {
    const ROOT_TABLE_INDEX: flatbuffer::TableIndex = flatbuffer::TableIndex(0);
    const STAMP_TABLE_INDEX: flatbuffer::TableIndex = flatbuffer::TableIndex(1);
    const SEC_VALUE_INDEX: ValueIndex = ValueIndex(0);
    const FRAME_ID_VALUE_INDEX: ValueIndex = ValueIndex(2);

    let compiled = compile(schemas::HEADER, messages::HEADER).expect("compiles");

    assert_that!(compiled.flatbuffer.tables, len 2);
    assert_that!(compiled.flatbuffer.table(ROOT_TABLE_INDEX).placement, eq flatbuffer::TablePlacement::Root);
    assert_that!(
        compiled.flatbuffer.table(STAMP_TABLE_INDEX).placement,
        eq flatbuffer::TablePlacement::Nested(flatbuffer::Slot {
            table: ROOT_TABLE_INDEX,
            vtable_offset: header::std_msgs::msg::Header::VT_STAMP,
        })
    );
    assert_that!(compiled.flatbuffer.field(SEC_VALUE_INDEX).placement.slot().expect("sits in a table").table, eq STAMP_TABLE_INDEX);
    assert_that!(compiled.flatbuffer.field(FRAME_ID_VALUE_INDEX).placement.slot().expect("sits in a table").table, eq ROOT_TABLE_INDEX);
}

#[test]
fn reports_the_path_of_a_refused_nested_member() {
    let error = compile(schemas::HEADER_MISMATCHED, messages::HEADER).expect_err("refuses");

    assert_that!(error.path, eq "stamp.nanosec");
    assert_that!(
        error.refusal,
        eq Refusal::Mismatch { member: "uint32".into(), field: "Bool".into() }
    );
}

#[test]
fn records_the_struct_that_holds_each_value() {
    const ROOT_TABLE_INDEX: flatbuffer::TableIndex = flatbuffer::TableIndex(0);
    const SEC_VALUE_INDEX: ValueIndex = ValueIndex(0);
    const NANOSEC_VALUE_INDEX: ValueIndex = ValueIndex(1);
    const NANOSEC_OFFSET: flatbuffer::StructOffset = flatbuffer::StructOffset(4);
    const TIME_NUMBER_OF_BYTES: u16 = 8;
    const TIME_ALIGNMENT: u16 = 4;
    const STAMP_SLOT: flatbuffer::Slot = flatbuffer::Slot {
        table: ROOT_TABLE_INDEX,
        vtable_offset: header_struct::std_msgs::msg::Header::VT_STAMP,
    };

    let compiled = compile(schemas::HEADER_STRUCT, messages::HEADER).expect("compiles");

    assert_that!(
        compiled.flatbuffer.structs,
        eq vec![flatbuffer::StructEntry {
            slot: STAMP_SLOT,
            number_of_bytes: TIME_NUMBER_OF_BYTES,
            alignment: TIME_ALIGNMENT,
            values: vec![SEC_VALUE_INDEX, NANOSEC_VALUE_INDEX],
        }]
    );
    assert_that!(
        compiled.flatbuffer.field(NANOSEC_VALUE_INDEX).placement,
        eq flatbuffer::FieldPlacement::Struct { slot: STAMP_SLOT, offset: NANOSEC_OFFSET }
    );
}

#[test]
fn adds_the_values_of_a_struct_inside_a_struct_to_the_outer_struct() {
    const ROOT_TABLE_INDEX: flatbuffer::TableIndex = flatbuffer::TableIndex(0);
    const POSITION_X_VALUE_INDEX: u16 = 3;
    const ORIENTATION_W_VALUE_INDEX: u16 = 9;
    const ORIENTATION_W_OFFSET: flatbuffer::StructOffset = flatbuffer::StructOffset(48);
    const POSE_STRUCT_INDEX: usize = 1;
    const POSE_NUMBER_OF_BYTES: u16 = 56;
    const POSE_ALIGNMENT: u16 = 8;
    const POSE_SLOT: flatbuffer::Slot = flatbuffer::Slot {
        table: ROOT_TABLE_INDEX,
        vtable_offset: pose_stamped::geometry_msgs::msg::PoseStamped::VT_POSE,
    };

    let compiled = compile(schemas::POSE_STAMPED, messages::POSE_STAMPED).expect("compiles");

    assert_that!(
        compiled.flatbuffer.structs[POSE_STRUCT_INDEX],
        eq flatbuffer::StructEntry {
            slot: POSE_SLOT,
            number_of_bytes: POSE_NUMBER_OF_BYTES,
            alignment: POSE_ALIGNMENT,
            values: (POSITION_X_VALUE_INDEX..=ORIENTATION_W_VALUE_INDEX)
                .map(ValueIndex)
                .collect(),
        }
    );
    assert_that!(
        compiled.flatbuffer.field(ValueIndex(ORIENTATION_W_VALUE_INDEX)).placement,
        eq flatbuffer::FieldPlacement::Struct { slot: POSE_SLOT, offset: ORIENTATION_W_OFFSET }
    );
}

#[test]
fn tells_scalar_sequences_from_scalars() {
    let compiled = compile(schemas::CHANNEL_FLOAT32, messages::CHANNEL_FLOAT32).expect("compiles");

    assert_that!(
        compiled.model.values,
        eq vec![Kind::String, Kind::ScalarSequence(ScalarType::F32)]
    );
}

#[test]
fn refuses_a_vector_of_another_scalar() {
    let error = compile(
        schemas::CHANNEL_FLOAT32_MISMATCHED,
        messages::CHANNEL_FLOAT32,
    )
    .expect_err("refuses");

    assert_that!(error.path, eq "values");
    assert_that!(
        error.refusal,
        eq Refusal::Mismatch { member: "float32[]".into(), field: "[Double]".into() }
    );
}

#[test]
fn records_the_number_of_elements_of_a_scalar_array() {
    const COVARIANCE_VALUE_INDEX: usize = 7;
    const COVARIANCE_NUMBER_OF_ELEMENTS: u32 = 36;

    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE,
        messages::POSE_WITH_COVARIANCE,
    )
    .expect("compiles");

    assert_that!(
        compiled.model.values[COVARIANCE_VALUE_INDEX],
        eq Kind::ScalarArray {
            scalar: ScalarType::F64,
            number_of_elements: COVARIANCE_NUMBER_OF_ELEMENTS,
        }
    );
}

#[test]
fn records_the_offset_of_an_array_inside_a_struct() {
    const ROOT_TABLE_INDEX: flatbuffer::TableIndex = flatbuffer::TableIndex(0);
    const COVARIANCE_VALUE_INDEX: ValueIndex = ValueIndex(10);
    const COVARIANCE_NUMBER_OF_ELEMENTS: u32 = 36;
    const COVARIANCE_OFFSET: flatbuffer::StructOffset = flatbuffer::StructOffset(56);
    const POSE_SLOT: flatbuffer::Slot = flatbuffer::Slot {
        table: ROOT_TABLE_INDEX,
        vtable_offset: pose_with_covariance_stamped::PoseWithCovarianceStamped::VT_POSE,
    };

    let compiled = compile(
        schemas::POSE_WITH_COVARIANCE_STAMPED,
        messages::POSE_WITH_COVARIANCE_STAMPED,
    )
    .expect("compiles");

    assert_that!(
        *compiled.flatbuffer.field(COVARIANCE_VALUE_INDEX),
        eq flatbuffer::FieldEntry {
            placement: flatbuffer::FieldPlacement::Struct {
                slot: POSE_SLOT,
                offset: COVARIANCE_OFFSET,
            },
            kind: flatbuffer::FieldKind::ScalarArray {
                scalar: ScalarType::F64,
                number_of_elements: COVARIANCE_NUMBER_OF_ELEMENTS,
            },
        }
    );
}

#[test]
fn tells_string_sequences_from_strings() {
    let compiled = compile(schemas::JOINT_STATE, messages::JOINT_STATE).expect("compiles");

    assert_that!(
        compiled.model.values,
        eq vec![
            Kind::Scalar(ScalarType::I32),
            Kind::Scalar(ScalarType::U32),
            Kind::String,
            Kind::StringSequence,
            Kind::ScalarSequence(ScalarType::F64),
            Kind::ScalarSequence(ScalarType::F64),
            Kind::ScalarSequence(ScalarType::F64),
        ]
    );
}

#[test]
fn gives_a_repeated_message_its_own_model() {
    const POSES_MODEL_INDEX: ModelIndex = ModelIndex(0);
    const NUMBER_OF_POSE_VALUES: usize = 7;

    let compiled = compile(schemas::POSE_ARRAY, messages::POSE_ARRAY).expect("compiles");

    assert_that!(
        compiled.model.values,
        eq vec![
            Kind::Scalar(ScalarType::I32),
            Kind::Scalar(ScalarType::U32),
            Kind::String,
            Kind::MessageSequence(POSES_MODEL_INDEX),
        ]
    );
    assert_that!(
        compiled.model.repeated[usize::from(POSES_MODEL_INDEX.0)].values,
        eq vec![Kind::Scalar(ScalarType::F64); NUMBER_OF_POSE_VALUES]
    );
}

#[test]
fn gives_a_repeated_struct_a_layout_with_a_struct_root() {
    const POSES_VALUE_INDEX: ValueIndex = ValueIndex(3);
    const POSES_MODEL_INDEX: ModelIndex = ModelIndex(0);
    const NUMBER_OF_POSE_VALUES: u16 = 7;
    const POSE_NUMBER_OF_BYTES: u16 = 56;
    const POSE_ALIGNMENT: u16 = 8;
    const ORIENTATION_W_VALUE_INDEX: ValueIndex = ValueIndex(6);
    const ORIENTATION_W_OFFSET: flatbuffer::StructOffset = flatbuffer::StructOffset(48);

    let compiled = compile(schemas::POSE_ARRAY_STRUCT, messages::POSE_ARRAY).expect("compiles");

    assert_that!(
        compiled.flatbuffer.field(POSES_VALUE_INDEX).kind,
        eq flatbuffer::FieldKind::StructVector(POSES_MODEL_INDEX)
    );
    let poses = compiled.flatbuffer.repeated(POSES_MODEL_INDEX);
    assert_that!(
        poses.root,
        eq flatbuffer::RootObject::Struct {
            number_of_bytes: POSE_NUMBER_OF_BYTES,
            alignment: POSE_ALIGNMENT,
            values: (0..NUMBER_OF_POSE_VALUES).map(ValueIndex).collect(),
        }
    );
    assert_that!(
        poses.field(ORIENTATION_W_VALUE_INDEX).placement,
        eq flatbuffer::FieldPlacement::VectorElement { offset: ORIENTATION_W_OFFSET }
    );
}

#[test]
fn refuses_a_member_of_a_type_no_field_carries() {
    let error = compile(schemas::WSTRING, messages::WSTRING).expect_err("refuses");

    assert_that!(error.path, eq "data");
    assert_that!(error.refusal, eq Refusal::Unsupported { member: "wstring".into() });
}

#[test]
fn pairs_an_enum_with_its_underlying_scalar() {
    let compiled = compile(schemas::UINT8_ENUM, messages::UINT8).expect("compiles");

    assert_that!(compiled.model.values, eq vec![Kind::Scalar(ScalarType::U8)]);
}

#[test]
fn records_the_number_of_elements_of_string_and_message_arrays() {
    const STRING_VALUES_INDEX: usize = 13;
    const BASIC_TYPES_VALUES_INDEX: usize = 14;
    const BASIC_TYPES_MODEL_INDEX: ModelIndex = ModelIndex(0);
    const NUMBER_OF_ELEMENTS: u32 = 3;

    let compiled = compile(schemas::ARRAYS, messages::ARRAYS).expect("compiles");

    assert_that!(
        compiled.model.values[STRING_VALUES_INDEX],
        eq Kind::StringArray { number_of_elements: NUMBER_OF_ELEMENTS }
    );
    assert_that!(
        compiled.model.values[BASIC_TYPES_VALUES_INDEX],
        eq Kind::MessageArray {
            model: BASIC_TYPES_MODEL_INDEX,
            number_of_elements: NUMBER_OF_ELEMENTS,
        }
    );
}

#[test]
fn pairs_bounded_sequences_like_sequences() {
    const BOOL_VALUES_INDEX: usize = 0;
    const STRING_VALUES_INDEX: usize = 13;
    const BASIC_TYPES_VALUES_INDEX: usize = 14;
    const BASIC_TYPES_MODEL_INDEX: ModelIndex = ModelIndex(0);

    let compiled = compile(schemas::ARRAYS, messages::BOUNDED_SEQUENCES).expect("compiles");

    assert_that!(
        compiled.model.values[BOOL_VALUES_INDEX],
        eq Kind::ScalarSequence(ScalarType::Bool)
    );
    assert_that!(compiled.model.values[STRING_VALUES_INDEX], eq Kind::StringSequence);
    assert_that!(
        compiled.model.values[BASIC_TYPES_VALUES_INDEX],
        eq Kind::MessageSequence(BASIC_TYPES_MODEL_INDEX)
    );
}
