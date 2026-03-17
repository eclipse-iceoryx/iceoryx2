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

use iceoryx2_bb_posix_tests_common::signal_set_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn signal_set_new_empty_signal_set_does_not_contain_a_signal() {
    signal_set_tests::signal_set_new_empty_signal_set_does_not_contain_a_signal();
}

#[inventory_test]
fn signal_set_new_filled_signal_set_does_contain_all_signals() {
    signal_set_tests::signal_set_new_filled_signal_set_does_contain_all_signals();
}

#[inventory_test]
fn signal_set_adding_new_signals_works() {
    signal_set_tests::signal_set_adding_new_signals_works();
}

#[inventory_test]
fn signal_set_removing_signals_works() {
    signal_set_tests::signal_set_removing_signals_works();
}

#[inventory_test]
fn signal_set_create_from_pending_signals_with_no_pending_signals_is_empty() {
    signal_set_tests::signal_set_create_from_pending_signals_with_no_pending_signals_is_empty();
}

#[inventory_test]
fn signal_set_new_empty_fetchable_signal_set_does_not_contain_a_signal() {
    signal_set_tests::signal_set_new_empty_fetchable_signal_set_does_not_contain_a_signal();
}

#[inventory_test]
fn signal_set_new_filled_fetchable_signal_set_does_contain_all_signals() {
    signal_set_tests::signal_set_new_filled_fetchable_signal_set_does_contain_all_signals();
}

#[inventory_test]
fn signal_set_adding_new_fetchable_signals_works() {
    signal_set_tests::signal_set_adding_new_fetchable_signals_works();
}

#[inventory_test]
fn signal_set_removing_fetchable_signals_works() {
    signal_set_tests::signal_set_removing_fetchable_signals_works();
}

#[inventory_test]
fn signal_set_create_from_pending_fetchable_signals_works() {
    signal_set_tests::signal_set_create_from_pending_fetchable_signals_works();
}
