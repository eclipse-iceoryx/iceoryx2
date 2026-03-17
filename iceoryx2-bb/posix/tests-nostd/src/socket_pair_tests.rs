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

use iceoryx2_bb_posix_tests_common::socket_pair_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn try_receive_never_blocks() {
    socket_pair_tests::try_receive_never_blocks();
}

#[inventory_test]
fn send_receive_works() {
    socket_pair_tests::send_receive_works();
}

#[inventory_test]
fn bidirectional_send_receive_works() {
    socket_pair_tests::bidirectional_send_receive_works();
}

#[inventory_test]
fn cannot_receive_my_own_data() {
    socket_pair_tests::cannot_receive_my_own_data();
}

#[inventory_test]
fn timed_receive_blocks_for_at_least_timeout() {
    socket_pair_tests::timed_receive_blocks_for_at_least_timeout();
}

#[inventory_test]
fn timed_receive_blocks_until_message_arrives() {
    socket_pair_tests::timed_receive_blocks_until_message_arrives();
}

#[inventory_test]
fn blocking_receive_blocks_until_message_arrives() {
    socket_pair_tests::blocking_receive_blocks_until_message_arrives();
}

#[inventory_test]
fn timed_send_blocks_for_at_least_timeout() {
    socket_pair_tests::timed_send_blocks_for_at_least_timeout();
}

#[inventory_test]
fn timed_send_blocks_until_message_buffer_is_free_again() {
    socket_pair_tests::timed_send_blocks_until_message_buffer_is_free_again();
}

#[inventory_test]
fn blocking_send_blocks_until_message_buffer_is_free_again() {
    socket_pair_tests::blocking_send_blocks_until_message_buffer_is_free_again();
}

#[inventory_test]
fn peeking_message_does_not_remove_message() {
    socket_pair_tests::peeking_message_does_not_remove_message();
}

#[inventory_test]
fn send_from_duplicated_socket_works() {
    socket_pair_tests::send_from_duplicated_socket_works();
}

#[inventory_test]
fn receive_from_duplicated_socket_works() {
    socket_pair_tests::receive_from_duplicated_socket_works();
}

#[inventory_test]
fn multiple_duplicated_sockets_can_send() {
    socket_pair_tests::multiple_duplicated_sockets_can_send();
}
