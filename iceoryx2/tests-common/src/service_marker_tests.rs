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

use iceoryx2::service::marker::is_flatbuffer_type_name;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
fn the_marker_and_composed_names_are_flatbuffer_type_names() {
    assert_that!(is_flatbuffer_type_name("iox2::Flatbuffer"), eq true);
    assert_that!(is_flatbuffer_type_name("iox2::Flatbuffer<Testing.UnboundedData>"), eq true);
}

#[test]
fn other_names_are_not_flatbuffer_type_names() {
    assert_that!(is_flatbuffer_type_name("iox2::FlatbufferLike"), eq false);
    assert_that!(is_flatbuffer_type_name("iox2::Flatbuffer<open"), eq false);
    assert_that!(is_flatbuffer_type_name("Testing.UnboundedData"), eq false);
}
