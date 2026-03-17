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

use iceoryx2_bb_posix_tests_common::signal_tests;

#[test]
fn signal_register_single_handler_works() {
    signal_tests::signal_register_single_handler_works();
}

#[test]
fn signal_register_multiple_handler_works() {
    signal_tests::signal_register_multiple_handler_works();
}

#[test]
fn signal_register_handler_with_multiple_signals_works() {
    signal_tests::signal_register_handler_with_multiple_signals_works();
}

#[test]
fn signal_guard_unregisters_on_drop() {
    signal_tests::signal_guard_unregisters_on_drop();
}

#[test]
fn signal_register_signal_twice_fails() {
    signal_tests::signal_register_signal_twice_fails();
}

#[test]
fn signal_call_and_fetch_works() {
    signal_tests::signal_call_and_fetch_works();
}

#[test]
fn signal_call_and_fetch_with_registered_handler_works() {
    signal_tests::signal_call_and_fetch_with_registered_handler_works();
}

#[test]
fn signal_wait_for_signal_blocks() {
    signal_tests::signal_wait_for_signal_blocks();
}

#[test]
fn signal_wait_twice_for_same_signal_blocks() {
    signal_tests::signal_wait_twice_for_same_signal_blocks();
}

#[test]
fn signal_timed_wait_blocks_at_least_for_timeout() {
    signal_tests::signal_timed_wait_blocks_at_least_for_timeout();
}

#[test]
fn signal_timed_wait_blocks_until_signal() {
    signal_tests::signal_timed_wait_blocks_until_signal();
}

#[test]
fn signal_termination_requested_with_terminate_works() {
    signal_tests::signal_termination_requested_with_terminate_works();
}

#[test]
fn signal_termination_requested_with_interrupt_works() {
    signal_tests::signal_termination_requested_with_interrupt_works();
}
