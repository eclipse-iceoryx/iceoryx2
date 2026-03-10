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

use iceoryx2_bb_posix_tests_common::deadline_queue_tests;

#[test]
fn deadline_queue_attach_detach_works() {
    deadline_queue_tests::deadline_queue_attach_detach_works();
}

#[test]
fn deadline_queue_duration_until_next_deadline_is_max_for_empty_queue() {
    deadline_queue_tests::deadline_queue_duration_until_next_deadline_is_max_for_empty_queue();
}

#[test]
fn deadline_queue_next_iteration_works_zero_deadline() {
    deadline_queue_tests::deadline_queue_next_iteration_works_zero_deadline();
}

#[test]
fn deadline_queue_next_iteration_works_smallest_deadline_added_first() {
    deadline_queue_tests::deadline_queue_next_iteration_works_smallest_deadline_added_first();
}

#[test]
fn deadline_queue_next_iteration_works_smallest_deadline_added_last() {
    deadline_queue_tests::deadline_queue_next_iteration_works_smallest_deadline_added_last();
}

#[test]
fn deadline_queue_removing_deadline_works() {
    deadline_queue_tests::deadline_queue_removing_deadline_works();
}

#[test]
fn deadline_queue_no_missed_deadline_works() {
    deadline_queue_tests::deadline_queue_no_missed_deadline_works();
}

#[test]
fn deadline_queue_one_missed_deadlines_works() {
    deadline_queue_tests::deadline_queue_one_missed_deadlines_works();
}

#[test]
fn deadline_queue_many_missed_deadlines_works() {
    deadline_queue_tests::deadline_queue_many_missed_deadlines_works();
}

#[test]
fn deadline_queue_missed_deadline_iteration_stops_when_requested() {
    deadline_queue_tests::deadline_queue_missed_deadline_iteration_stops_when_requested();
}

#[test]
fn deadline_queue_duration_until_next_deadline_is_zero_if_deadline_is_already_missed() {
    deadline_queue_tests::deadline_queue_duration_until_next_deadline_is_zero_if_deadline_is_already_missed();
}

#[test]
fn deadline_queue_duration_until_next_deadline_is_not_zero_if_missed_deadline_have_been_handled() {
    deadline_queue_tests::deadline_queue_duration_until_next_deadline_is_not_zero_if_missed_deadline_have_been_handled();
}
