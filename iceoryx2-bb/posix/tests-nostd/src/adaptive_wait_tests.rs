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

use iceoryx2_bb_posix_tests_common::adaptive_wait_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
fn adaptive_wait_wait_at_different_time_depends_on_repetition_times() {
    adaptive_wait_tests::adaptive_wait_wait_at_different_time_depends_on_repetition_times();
}

#[inventory_test]
fn adaptive_wait_on_default_builder_uses_default_clock() {
    adaptive_wait_tests::adaptive_wait_on_default_builder_uses_default_clock();
}

#[inventory_test]
fn adaptive_wait_custom_clock_is_set_correctly() {
    adaptive_wait_tests::adaptive_wait_custom_clock_is_set_correctly();
}

#[inventory_test]
fn adaptive_wait_wait_increases_yield_counter() {
    adaptive_wait_tests::adaptive_wait_wait_increases_yield_counter();
}

#[inventory_test]
fn adaptive_wait_timed_wait_while_wait_at_least_for_timeout() {
    adaptive_wait_tests::adaptive_wait_timed_wait_while_wait_at_least_for_timeout();
}

#[inventory_test]
fn adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_false() {
    adaptive_wait_tests::adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_false();
}

#[inventory_test]
fn adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_error() {
    adaptive_wait_tests::adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_error();
}
