// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

#![allow(clippy::disallowed_types)]

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::string_utils_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn strnlen_returns_len_for_non_empty_string() {
    string_utils_tests::strnlen_returns_len_for_non_empty_string();
}

#[inventory_test]
fn strnlen_returns_len_for_empty_string() {
    string_utils_tests::strnlen_returns_len_for_empty_string();
}

#[inventory_test]
fn strnlen_returns_max_len_when_string_is_longer_than_max_len() {
    string_utils_tests::strnlen_returns_max_len_when_string_is_longer_than_max_len();
}

#[inventory_test]
fn as_escaped_string_escapes_all_escapable_characters() {
    string_utils_tests::as_escaped_string_escapes_all_escapable_characters();
}

#[inventory_test]
fn as_escaped_string_does_not_escape_printable_characters() {
    string_utils_tests::as_escaped_string_does_not_escape_printable_characters();
}
