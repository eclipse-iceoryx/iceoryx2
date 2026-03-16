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

#![allow(clippy::disallowed_types)]

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::deadline_queue_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
fn deadline_queue_attach_detach_works() {
    deadline_queue_tests::deadline_queue_attach_detach_works();
}

#[inventory_test]
fn deadline_queue_duration_until_next_deadline_is_max_for_empty_queue() {
    deadline_queue_tests::deadline_queue_duration_until_next_deadline_is_max_for_empty_queue();
}

#[inventory_test]
fn deadline_queue_next_iteration_works_zero_deadline() {
    deadline_queue_tests::deadline_queue_next_iteration_works_zero_deadline();
}

#[inventory_test]
fn deadline_queue_next_iteration_works_smallest_deadline_added_first() {
    deadline_queue_tests::deadline_queue_next_iteration_works_smallest_deadline_added_first();
}

#[inventory_test]
fn deadline_queue_next_iteration_works_smallest_deadline_added_last() {
    deadline_queue_tests::deadline_queue_next_iteration_works_smallest_deadline_added_last();
}

#[inventory_test]
fn deadline_queue_removing_deadline_works() {
    deadline_queue_tests::deadline_queue_removing_deadline_works();
}

#[inventory_test]
fn deadline_queue_no_missed_deadline_works() {
    deadline_queue_tests::deadline_queue_no_missed_deadline_works();
}

#[inventory_test]
fn deadline_queue_one_missed_deadlines_works() {
    deadline_queue_tests::deadline_queue_one_missed_deadlines_works();
}

#[inventory_test]
fn deadline_queue_many_missed_deadlines_works() {
    deadline_queue_tests::deadline_queue_many_missed_deadlines_works();
}

#[inventory_test]
fn deadline_queue_missed_deadline_iteration_stops_when_requested() {
    deadline_queue_tests::deadline_queue_missed_deadline_iteration_stops_when_requested();
}

#[inventory_test]
fn deadline_queue_duration_until_next_deadline_is_zero_if_deadline_is_already_missed() {
    deadline_queue_tests::deadline_queue_duration_until_next_deadline_is_zero_if_deadline_is_already_missed();
}

#[inventory_test]
fn deadline_queue_duration_until_next_deadline_is_not_zero_if_missed_deadline_have_been_handled() {
    deadline_queue_tests::deadline_queue_duration_until_next_deadline_is_not_zero_if_missed_deadline_have_been_handled();
}
