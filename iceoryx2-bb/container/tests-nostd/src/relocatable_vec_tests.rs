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

use iceoryx2_bb_container_tests_common::relocatable_vec_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn double_init_call_causes_panic() {
    relocatable_vec_tests::double_init_call_causes_panic();
}

#[inventory_test]
fn panic_is_called_in_debug_mode_if_vec_is_not_initialized() {
    relocatable_vec_tests::panic_is_called_in_debug_mode_if_vec_is_not_initialized();
}

#[inventory_test]
fn two_vectors_with_same_content_are_equal() {
    relocatable_vec_tests::two_vectors_with_same_content_are_equal();
}

#[inventory_test]
fn two_vectors_with_different_content_are_not_equal() {
    relocatable_vec_tests::two_vectors_with_different_content_are_not_equal();
}

#[inventory_test]
fn two_vectors_with_different_len_are_not_equal() {
    relocatable_vec_tests::two_vectors_with_different_len_are_not_equal();
}
