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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::static_string_tests;

#[test]
fn default_is_empty() {
    static_string_tests::default_is_empty();
}

#[test]
fn placement_default_works() {
    static_string_tests::placement_default_works();
}

#[test]
fn from_bytes_unchecked_works() {
    static_string_tests::from_bytes_unchecked_works();
}

#[test]
fn from_bytes_unchecked_with_empty_slice_works() {
    static_string_tests::from_bytes_unchecked_with_empty_slice_works();
}

#[test]
fn from_bytes_with_len_smaller_capacity_works() {
    static_string_tests::from_bytes_with_len_smaller_capacity_works();
}

#[test]
fn from_bytes_with_empty_slice_works() {
    static_string_tests::from_bytes_with_empty_slice_works();
}

#[test]
fn from_bytes_fails_when_len_exceeds_capacity() {
    static_string_tests::from_bytes_fails_when_len_exceeds_capacity();
}

#[test]
fn from_bytes_truncated_works_with_empty_bytes() {
    static_string_tests::from_bytes_truncated_works_with_empty_bytes();
}

#[test]
fn from_bytes_truncated_works_with_len_smaller_than_capacity() {
    static_string_tests::from_bytes_truncated_works_with_len_smaller_than_capacity();
}

#[test]
fn from_bytes_truncated_works_with_len_greater_than_capacity() {
    static_string_tests::from_bytes_truncated_works_with_len_greater_than_capacity();
}

#[test]
fn from_bytes_truncated_fails_with_invalid_characters() {
    static_string_tests::from_bytes_truncated_fails_with_invalid_characters();
}

#[test]
fn from_str_with_len_smaller_capacity_works() {
    static_string_tests::from_str_with_len_smaller_capacity_works();
}

#[test]
fn from_str_with_len_zero_works() {
    static_string_tests::from_str_with_len_zero_works();
}

#[test]
fn from_str_with_len_greater_than_capacity_fails() {
    static_string_tests::from_str_with_len_greater_than_capacity_fails();
}

#[test]
fn from_str_truncated_with_len_smaller_capacity_works() {
    static_string_tests::from_str_truncated_with_len_smaller_capacity_works();
}

#[test]
fn from_str_truncated_with_len_greater_than_capacity_truncates() {
    static_string_tests::from_str_truncated_with_len_greater_than_capacity_truncates();
}

#[test]
fn from_str_truncated_fails_with_invalid_characters() {
    static_string_tests::from_str_truncated_fails_with_invalid_characters();
}

#[test]
fn from_c_str_works_for_empty_string() {
    static_string_tests::from_c_str_works_for_empty_string();
}

#[test]
fn from_c_str_works_when_len_is_smaller_than_capacity() {
    static_string_tests::from_c_str_works_when_len_is_smaller_than_capacity();
}

#[test]
fn from_c_str_fails_when_len_is_greater_than_capacity() {
    static_string_tests::from_c_str_fails_when_len_is_greater_than_capacity();
}

#[test]
fn eq_with_slice_works() {
    static_string_tests::eq_with_slice_works();
}

#[test]
fn serialization_works() {
    static_string_tests::serialization_works();
}

#[test]
fn try_from_str_fails_when_too_long() {
    static_string_tests::try_from_str_fails_when_too_long();
}

#[test]
fn try_from_str_fails_when_it_contains_invalid_characters() {
    static_string_tests::try_from_str_fails_when_it_contains_invalid_characters();
}

#[test]
fn try_from_str_with_valid_content_works() {
    static_string_tests::try_from_str_with_valid_content_works();
}

#[test]
fn try_from_u8_array_fails_when_too_long() {
    static_string_tests::try_from_u8_array_fails_when_too_long();
}

#[test]
fn try_from_u8_array_fails_when_it_contains_invalid_characters() {
    static_string_tests::try_from_u8_array_fails_when_it_contains_invalid_characters();
}

#[test]
fn try_from_u8_array_with_valid_content_works() {
    static_string_tests::try_from_u8_array_with_valid_content_works();
}
