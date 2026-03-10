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

use iceoryx2_bb_posix_tests_common::unix_datagram_socket_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn unix_datagram_socket_send_receive_works() {
    unix_datagram_socket_tests::unix_datagram_socket_send_receive_works();
}

#[inventory_test]
fn unix_datagram_socket_adjust_buffer_size_works() {
    unix_datagram_socket_tests::unix_datagram_socket_adjust_buffer_size_works();
}

#[inventory_test]
fn unix_datagram_socket_non_blocking_mode_returns_zero_when_nothing_was_received() {
    unix_datagram_socket_tests::unix_datagram_socket_non_blocking_mode_returns_zero_when_nothing_was_received();
}

#[inventory_test]
fn unix_datagram_socket_blocking_mode_blocks() {
    unix_datagram_socket_tests::unix_datagram_socket_blocking_mode_blocks();
}

#[inventory_test]
fn unix_datagram_socket_timeout_blocks_at_least() {
    unix_datagram_socket_tests::unix_datagram_socket_timeout_blocks_at_least();
}

// TODO iox2-#320
#[inventory_test(ignore)]
fn unix_datagram_socket_sending_receiving_with_single_fd_works() {
    unix_datagram_socket_tests::unix_datagram_socket_sending_receiving_with_single_fd_works();
}

#[inventory_test(ignore)]
fn unix_datagram_socket_sending_receiving_credentials_works() {
    unix_datagram_socket_tests::unix_datagram_socket_sending_receiving_credentials_works();
}

#[inventory_test(ignore)]
fn unix_datagram_socket_sending_receiving_with_max_supported_fd_and_credentials_works() {
    unix_datagram_socket_tests::unix_datagram_socket_sending_receiving_with_max_supported_fd_and_credentials_works();
}
