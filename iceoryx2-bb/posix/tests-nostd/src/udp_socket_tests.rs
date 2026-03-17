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

use iceoryx2_bb_posix_tests_common::udp_socket_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn udp_socket_send_receive_works() {
    udp_socket_tests::udp_socket_send_receive_works();
}

#[inventory_test]
fn udp_socket_server_with_same_address_and_port_fails() {
    udp_socket_tests::udp_socket_server_with_same_address_and_port_fails();
}

#[inventory_test]
fn udp_socket_when_socket_goes_out_of_scope_address_is_free_again() {
    udp_socket_tests::udp_socket_when_socket_goes_out_of_scope_address_is_free_again();
}

#[inventory_test]
fn udp_socket_server_has_correct_address() {
    udp_socket_tests::udp_socket_server_has_correct_address();
}

#[inventory_test]
fn udp_socket_client_returns_address_of_server() {
    udp_socket_tests::udp_socket_client_returns_address_of_server();
}

#[inventory_test]
fn udp_socket_client_can_send_data_to_server() {
    udp_socket_tests::udp_socket_client_can_send_data_to_server();
}

#[inventory_test]
fn udp_socket_server_can_send_data_to_client() {
    udp_socket_tests::udp_socket_server_can_send_data_to_client();
}

#[inventory_test]
fn udp_socket_client_try_receive_does_not_block() {
    udp_socket_tests::udp_socket_client_try_receive_does_not_block();
}

#[inventory_test]
fn udp_socket_server_try_receive_from_does_not_block() {
    udp_socket_tests::udp_socket_server_try_receive_from_does_not_block();
}

#[inventory_test]
fn udp_socket_client_timed_receive_does_block_for_at_least_timeout() {
    udp_socket_tests::udp_socket_client_timed_receive_does_block_for_at_least_timeout();
}

#[inventory_test]
fn udp_socket_server_timed_receive_from_does_block_for_at_least_timeout() {
    udp_socket_tests::udp_socket_server_timed_receive_from_does_block_for_at_least_timeout();
}

#[inventory_test]
fn udp_socket_client_blocking_receive_does_block() {
    udp_socket_tests::udp_socket_client_blocking_receive_does_block();
}

#[inventory_test]
fn udp_socket_server_blocking_receive_from_does_block() {
    udp_socket_tests::udp_socket_server_blocking_receive_from_does_block();
}

#[inventory_test]
fn udp_socket_client_timed_receive_does_blocks() {
    udp_socket_tests::udp_socket_client_timed_receive_does_blocks();
}

#[inventory_test]
fn udp_socket_server_timed_receive_from_does_block() {
    udp_socket_tests::udp_socket_server_timed_receive_from_does_block();
}
