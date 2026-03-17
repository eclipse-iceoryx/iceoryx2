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

use iceoryx2_bb_posix_tests_common::adaptive_wait_tests;

#[test]
fn adaptive_wait_wait_at_different_time_depends_on_repetition_times() {
    adaptive_wait_tests::adaptive_wait_wait_at_different_time_depends_on_repetition_times();
}

#[test]
fn adaptive_wait_on_default_builder_uses_default_clock() {
    adaptive_wait_tests::adaptive_wait_on_default_builder_uses_default_clock();
}

#[test]
fn adaptive_wait_custom_clock_is_set_correctly() {
    adaptive_wait_tests::adaptive_wait_custom_clock_is_set_correctly();
}

#[test]
fn adaptive_wait_wait_increases_yield_counter() {
    adaptive_wait_tests::adaptive_wait_wait_increases_yield_counter();
}

#[test]
fn adaptive_wait_timed_wait_while_wait_at_least_for_timeout() {
    adaptive_wait_tests::adaptive_wait_timed_wait_while_wait_at_least_for_timeout();
}

#[test]
fn adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_false() {
    adaptive_wait_tests::adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_false();
}

#[test]
fn adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_error() {
    adaptive_wait_tests::adaptive_wait_timed_wait_does_not_wait_when_predicate_returns_error();
}
