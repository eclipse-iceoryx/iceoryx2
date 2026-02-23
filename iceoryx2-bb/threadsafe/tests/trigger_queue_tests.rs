// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_threadsafe_tests_common::trigger_queue_tests;

#[test]
fn trigger_queue_new_queue_is_empty() {
    trigger_queue_tests::trigger_queue_new_queue_is_empty();
}

#[test]
fn trigger_queue_try_push_pop_works() {
    trigger_queue_tests::trigger_queue_try_push_pop_works();
}

#[test]
fn trigger_queue_timed_push_pop_works() {
    trigger_queue_tests::trigger_queue_timed_push_pop_works();
}

#[test]
fn trigger_queue_blocking_push_pop_works() {
    trigger_queue_tests::trigger_queue_blocking_push_pop_works();
}

#[test]
fn trigger_queue_timed_push_blocks_at_least_until_timeout_has_passed() {
    trigger_queue_tests::trigger_queue_timed_push_blocks_at_least_until_timeout_has_passed();
}

#[test]
fn trigger_queue_timed_pop_blocks_at_least_until_timeout_has_passed() {
    trigger_queue_tests::trigger_queue_timed_pop_blocks_at_least_until_timeout_has_passed();
}

#[test]
fn trigger_queue_blocking_push_blocks_until_there_is_space_again() {
    trigger_queue_tests::trigger_queue_blocking_push_blocks_until_there_is_space_again();
}

#[test]
fn trigger_queue_blocking_pop_blocks_until_there_is_something_pushed() {
    trigger_queue_tests::trigger_queue_blocking_pop_blocks_until_there_is_something_pushed();
}

#[test]
fn trigger_queue_one_pop_notifies_exactly_one_blocking_push() {
    trigger_queue_tests::trigger_queue_one_pop_notifies_exactly_one_blocking_push();
}

#[test]
fn trigger_queue_one_pop_notifies_exactly_one_timed_push() {
    trigger_queue_tests::trigger_queue_one_pop_notifies_exactly_one_timed_push();
}

#[test]
fn trigger_queue_one_push_notifies_exactly_one_blocking_pop() {
    trigger_queue_tests::trigger_queue_one_push_notifies_exactly_one_blocking_pop();
}

#[test]
fn trigger_queue_one_push_notifies_exactly_one_timed_pop() {
    trigger_queue_tests::trigger_queue_one_push_notifies_exactly_one_timed_pop();
}
