// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_system_types_tests_common::file_name_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn file_name_new_with_illegal_name_fails() {
    file_name_tests::file_name_new_with_illegal_name_fails();
}

#[inventory_test]
fn file_name_new_name_with_slash_is_illegal() {
    file_name_tests::file_name_new_name_with_slash_is_illegal();
}

#[inventory_test]
fn file_name_pop_fails_when_it_results_in_illegal_name() {
    file_name_tests::file_name_pop_fails_when_it_results_in_illegal_name();
}

#[inventory_test]
fn file_name_remove_fails_when_it_results_in_illegal_name() {
    file_name_tests::file_name_remove_fails_when_it_results_in_illegal_name();
}

#[inventory_test]
fn file_name_remove_range_fails_when_it_results_in_illegal_name() {
    file_name_tests::file_name_remove_range_fails_when_it_results_in_illegal_name();
}

#[inventory_test]
fn file_name_retain_fails_when_it_results_in_illegal_name() {
    file_name_tests::file_name_retain_fails_when_it_results_in_illegal_name();
}
