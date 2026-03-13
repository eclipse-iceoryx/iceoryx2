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

use iceoryx2_bb_container_tests_common::static_vec_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn default_created_vec_is_empty() {
    static_vec_tests::default_created_vec_is_empty();
}

#[inventory_test]
fn two_vectors_with_same_content_are_equal() {
    static_vec_tests::two_vectors_with_same_content_are_equal();
}

#[inventory_test]
fn two_vectors_with_different_content_are_not_equal() {
    static_vec_tests::two_vectors_with_different_content_are_not_equal();
}

#[inventory_test]
fn two_vectors_with_different_len_are_not_equal() {
    static_vec_tests::two_vectors_with_different_len_are_not_equal();
}

#[inventory_test]
fn placement_default_works() {
    static_vec_tests::placement_default_works();
}

#[inventory_test]
fn serialization_works() {
    static_vec_tests::serialization_works();
}

#[inventory_test]
fn valid_after_move() {
    static_vec_tests::valid_after_move();
}

#[inventory_test]
fn clone_clones_empty_vec() {
    static_vec_tests::clone_clones_empty_vec();
}

#[inventory_test]
fn clone_clones_filled_vec() {
    static_vec_tests::clone_clones_filled_vec();
}

#[inventory_test]
fn try_from_succeeds_when_slice_len_is_smaller_or_equal_capacity() {
    static_vec_tests::try_from_succeeds_when_slice_len_is_smaller_or_equal_capacity();
}

#[inventory_test]
fn try_from_fails_when_slice_len_is_greater_than_capacity() {
    static_vec_tests::try_from_fails_when_slice_len_is_greater_than_capacity();
}
