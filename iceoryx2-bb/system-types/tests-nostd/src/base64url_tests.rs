// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_system_types_tests_common::base64url_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn base64url_new_with_legal_content_works() {
    base64url_tests::base64url_new_with_legal_content_works();
}

#[inventory_test]
fn base64url_new_with_illegal_content_fails() {
    base64url_tests::base64url_new_with_illegal_content_fails();
}

#[inventory_test]
fn base64url_as_file_name_works() {
    base64url_tests::base64url_as_file_name_works();
}
