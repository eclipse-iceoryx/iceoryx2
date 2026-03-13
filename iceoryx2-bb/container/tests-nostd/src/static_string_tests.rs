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

use iceoryx2_bb_container_tests_common::static_string_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn default_is_empty() {
    static_string_tests::default_is_empty();
}

#[inventory_test]
fn placement_default_works() {
    static_string_tests::placement_default_works();
}

#[inventory_test]
fn from_bytes_unchecked_works() {
    static_string_tests::from_bytes_unchecked_works();
}

#[inventory_test]
fn from_bytes_unchecked_with_empty_slice_works() {
    static_string_tests::from_bytes_unchecked_with_empty_slice_works();
}

#[inventory_test]
fn from_bytes_with_len_smaller_capacity_works() {
    static_string_tests::from_bytes_with_len_smaller_capacity_works();
}

#[inventory_test]
fn from_bytes_with_empty_slice_works() {
    static_string_tests::from_bytes_with_empty_slice_works();
}

#[inventory_test]
fn from_bytes_fails_when_len_exceeds_capacity() {
    static_string_tests::from_bytes_fails_when_len_exceeds_capacity();
}

#[inventory_test]
fn from_bytes_truncated_works_with_empty_bytes() {
    static_string_tests::from_bytes_truncated_works_with_empty_bytes();
}

#[inventory_test]
fn from_bytes_truncated_works_with_len_smaller_than_capacity() {
    static_string_tests::from_bytes_truncated_works_with_len_smaller_than_capacity();
}

#[inventory_test]
fn from_bytes_truncated_works_with_len_greater_than_capacity() {
    static_string_tests::from_bytes_truncated_works_with_len_greater_than_capacity();
}

#[inventory_test]
fn from_bytes_truncated_fails_with_invalid_characters() {
    static_string_tests::from_bytes_truncated_fails_with_invalid_characters();
}

#[inventory_test]
fn from_str_with_len_smaller_capacity_works() {
    static_string_tests::from_str_with_len_smaller_capacity_works();
}

#[inventory_test]
fn from_str_with_len_zero_works() {
    static_string_tests::from_str_with_len_zero_works();
}

#[inventory_test]
fn from_str_with_len_greater_than_capacity_fails() {
    static_string_tests::from_str_with_len_greater_than_capacity_fails();
}

#[inventory_test]
fn from_str_truncated_with_len_smaller_capacity_works() {
    static_string_tests::from_str_truncated_with_len_smaller_capacity_works();
}

#[inventory_test]
fn from_str_truncated_with_len_greater_than_capacity_truncates() {
    static_string_tests::from_str_truncated_with_len_greater_than_capacity_truncates();
}

#[inventory_test]
fn from_str_truncated_fails_with_invalid_characters() {
    static_string_tests::from_str_truncated_fails_with_invalid_characters();
}

#[inventory_test]
fn from_c_str_works_for_empty_string() {
    static_string_tests::from_c_str_works_for_empty_string();
}

#[inventory_test]
fn from_c_str_works_when_len_is_smaller_than_capacity() {
    static_string_tests::from_c_str_works_when_len_is_smaller_than_capacity();
}

#[inventory_test]
fn from_c_str_fails_when_len_is_greater_than_capacity() {
    static_string_tests::from_c_str_fails_when_len_is_greater_than_capacity();
}

#[inventory_test]
fn eq_with_slice_works() {
    static_string_tests::eq_with_slice_works();
}

#[inventory_test]
fn serialization_works() {
    static_string_tests::serialization_works();
}

#[inventory_test]
fn try_from_str_fails_when_too_long() {
    static_string_tests::try_from_str_fails_when_too_long();
}

#[inventory_test]
fn try_from_str_fails_when_it_contains_invalid_characters() {
    static_string_tests::try_from_str_fails_when_it_contains_invalid_characters();
}

#[inventory_test]
fn try_from_str_with_valid_content_works() {
    static_string_tests::try_from_str_with_valid_content_works();
}

#[inventory_test]
fn try_from_u8_array_fails_when_too_long() {
    static_string_tests::try_from_u8_array_fails_when_too_long();
}

#[inventory_test]
fn try_from_u8_array_fails_when_it_contains_invalid_characters() {
    static_string_tests::try_from_u8_array_fails_when_it_contains_invalid_characters();
}

#[inventory_test]
fn try_from_u8_array_with_valid_content_works() {
    static_string_tests::try_from_u8_array_with_valid_content_works();
}
