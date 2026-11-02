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

use iceoryx2_bb_flatbuffers::{SchemaError, root_type_name};
use iceoryx2_bb_flatbuffers_generated::schema::testing as schema;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn root_type_name_is_the_qualified_name_of_the_root_table() {
    let name = root_type_name(schema::UNBOUNDED_DATA);

    assert_that!(name, eq Ok("Testing.UnboundedData"));
}

#[test]
pub fn a_schema_without_root_type_has_no_root_type_name() {
    let name = root_type_name(schema::NO_ROOT_TYPE);

    assert_that!(name, eq Err(SchemaError::NoRootType));
}

#[test]
pub fn bytes_that_are_no_schema_have_no_root_type_name() {
    const NOT_A_SCHEMA: &[u8] = b"some binary content";

    let name = root_type_name(NOT_A_SCHEMA);

    assert_that!(name, eq Err(SchemaError::NotABinarySchema));
}
