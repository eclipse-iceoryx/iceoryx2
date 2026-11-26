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
use iceoryx2_integrations_ros2_link_translator_flatbuffer::next::model::{
    Kind, ScalarType, ScalarValue, ValueIndex,
};
use iceoryx2_integrations_ros2_link_translator_flatbuffer::next::{Refusal, cdr, flatbuffer};

use crate::next::testing::compile;
use crate::next::testing::messages;
use crate::schemas;
use crate::schemas::multi_array_dimension::std_msgs::msg as multi_array_dimension;

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
    assert_that!(compiled.flatbuffer.fields[0].placement.slot().vtable_offset, eq multi_array_dimension::MultiArrayDimension::VT_LABEL);
    assert_that!(compiled.flatbuffer.fields[0].kind, eq flatbuffer::FieldKind::String);
    assert_that!(compiled.flatbuffer.fields[1].placement.slot().vtable_offset, eq multi_array_dimension::MultiArrayDimension::VT_SIZE);
    assert_that!(compiled.flatbuffer.fields[2].placement.slot().vtable_offset, eq multi_array_dimension::MultiArrayDimension::VT_STRIDE);
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
