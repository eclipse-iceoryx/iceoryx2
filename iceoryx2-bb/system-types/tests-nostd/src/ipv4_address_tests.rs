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

use iceoryx2_bb_system_types_tests_common::ipv4_address_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
fn ipv4_address_is_created_correctly() {
    ipv4_address_tests::ipv4_address_is_created_correctly();
}

#[inventory_test]
fn ipv4_address_is_unspecified_works() {
    ipv4_address_tests::ipv4_address_is_unspecified_works();
}

#[inventory_test]
fn ipv4_address_is_loopback_works() {
    ipv4_address_tests::ipv4_address_is_loopback_works();
}

#[inventory_test]
fn ipv4_address_is_private_works() {
    ipv4_address_tests::ipv4_address_is_private_works();
}

#[inventory_test]
fn ipv4_address_is_link_local_works() {
    ipv4_address_tests::ipv4_address_is_link_local_works();
}

#[inventory_test]
fn ipv4_address_is_shared_works() {
    ipv4_address_tests::ipv4_address_is_shared_works();
}

#[inventory_test]
fn ipv4_address_is_benchmarking_works() {
    ipv4_address_tests::ipv4_address_is_benchmarking_works();
}

#[inventory_test]
fn ipv4_address_is_reserved_works() {
    ipv4_address_tests::ipv4_address_is_reserved_works();
}

#[inventory_test]
fn ipv4_address_is_multicast_works() {
    ipv4_address_tests::ipv4_address_is_multicast_works();
}

#[inventory_test]
fn ipv4_address_is_broadcast_works() {
    ipv4_address_tests::ipv4_address_is_broadcast_works();
}

#[inventory_test]
fn ipv4_address_is_documentation_works() {
    ipv4_address_tests::ipv4_address_is_documentation_works();
}

#[inventory_test]
fn ipv4_address_is_global_works() {
    ipv4_address_tests::ipv4_address_is_global_works();
}

#[inventory_test]
fn ipv4_address_try_from_str_works() {
    ipv4_address_tests::ipv4_address_try_from_str_works();
}

#[inventory_test]
fn ipv4_address_try_from_fails_with_wrong_format_when_too_few_parts_are_given() {
    ipv4_address_tests::ipv4_address_try_from_fails_with_wrong_format_when_too_few_parts_are_given(
    );
}

#[inventory_test]
fn ipv4_address_try_from_fails_with_wrong_format_when_too_many_parts_are_given() {
    ipv4_address_tests::ipv4_address_try_from_fails_with_wrong_format_when_too_many_parts_are_given(
    );
}

#[inventory_test]
fn ipv4_address_try_from_fails_with_wrong_format_when_it_ends_with_a_dot() {
    ipv4_address_tests::ipv4_address_try_from_fails_with_wrong_format_when_it_ends_with_a_dot();
}

#[inventory_test]
fn ipv4_address_try_from_fails_when_part_is_not_an_u8_number() {
    ipv4_address_tests::ipv4_address_try_from_fails_when_part_is_not_an_u8_number();
}
