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
    self as translator, DecodeError,
};
use ros_env::builtin_interfaces::msg::rmw::Time;
use ros_env::std_msgs::msg::rmw::MultiArrayDimension;

use crate::next::testing::messages;
use crate::next::testing::{compile, decode, encode, serialize};
use crate::schemas;
use crate::schemas::multi_array_dimension::std_msgs::msg as multi_array_dimension;
use crate::schemas::time::builtin_interfaces::msg as time;

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
