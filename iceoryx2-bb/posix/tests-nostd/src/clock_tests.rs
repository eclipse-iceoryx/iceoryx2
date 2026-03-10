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

use iceoryx2_bb_posix_tests_common::clock_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn clock_nanosleep_sleeps_at_least_given_amount_of_time() {
    clock_tests::clock_nanosleep_sleeps_at_least_given_amount_of_time();
}

#[inventory_test]
fn clock_nanosleep_with_clock_sleeps_at_least_given_amount_of_time() {
    clock_tests::clock_nanosleep_with_clock_sleeps_at_least_given_amount_of_time();
}

#[inventory_test]
fn clock_timebuilder_default_values_are_set_correctly() {
    clock_tests::clock_timebuilder_default_values_are_set_correctly();
}

#[inventory_test]
fn clock_timebuilder_creates_time_correctly() {
    clock_tests::clock_timebuilder_creates_time_correctly();
}

#[inventory_test]
fn clock_time_conversion_to_duration_works() {
    clock_tests::clock_time_conversion_to_duration_works();
}

#[inventory_test]
fn clock_time_now_is_monotonic_with_monotonic_clock() {
    clock_tests::clock_time_now_is_monotonic_with_monotonic_clock();
}

#[inventory_test]
fn clock_time_as_timespec_works() {
    clock_tests::clock_time_as_timespec_works();
}

#[inventory_test]
fn clock_relocatable_duration_roundtrip_conversion() {
    clock_tests::clock_relocatable_duration_roundtrip_conversion();
}

#[inventory_test]
fn clock_relocatable_duration_max_value() {
    clock_tests::clock_relocatable_duration_max_value();
}
