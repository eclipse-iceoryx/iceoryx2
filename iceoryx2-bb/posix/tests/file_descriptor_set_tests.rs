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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::file_descriptor_set_tests;

#[test]
fn file_descriptor_set_timed_wait_blocks_at_least_timeout() {
    file_descriptor_set_tests::file_descriptor_set_timed_wait_blocks_at_least_timeout();
}

#[test]
fn file_descriptor_set_add_and_remove_works() {
    file_descriptor_set_tests::file_descriptor_set_add_and_remove_works();
}

#[test]
fn file_descriptor_set_add_same_fd_twice_fails() {
    file_descriptor_set_tests::file_descriptor_set_add_same_fd_twice_fails();
}

#[test]
fn file_descriptor_set_timed_wait_works() {
    file_descriptor_set_tests::file_descriptor_set_timed_wait_works();
}

#[test]
fn file_descriptor_set_blocking_wait_immediately_returns_notifications() {
    file_descriptor_set_tests::file_descriptor_set_blocking_wait_immediately_returns_notifications(
    );
}

#[test]
fn file_descriptor_guard_has_access_to_underlying_fd() {
    file_descriptor_set_tests::file_descriptor_guard_has_access_to_underlying_fd();
}

#[test]
fn file_descriptor_debug_works() {
    file_descriptor_set_tests::file_descriptor_debug_works();
}

#[test]
fn file_descriptor_triggering_many_returns_correct_number_of_notifications() {
    file_descriptor_set_tests::file_descriptor_triggering_many_returns_correct_number_of_notifications();
}
