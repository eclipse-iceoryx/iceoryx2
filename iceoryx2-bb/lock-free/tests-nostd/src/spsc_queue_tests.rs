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

use iceoryx2_bb_lock_free_tests_common::spsc_queue_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn spsc_queue_push_works_until_full() {
    spsc_queue_tests::spsc_queue_push_works_until_full();
}

#[inventory_test]
fn spsc_queue_pop_works_until_empty() {
    spsc_queue_tests::spsc_queue_pop_works_until_empty();
}

#[inventory_test]
fn spsc_queue_push_pop_alteration_works() {
    spsc_queue_tests::spsc_queue_push_pop_alteration_works();
}

#[inventory_test]
fn spsc_queue_get_consumer_twice_fails() {
    spsc_queue_tests::spsc_queue_get_consumer_twice_fails();
}

#[inventory_test]
fn spsc_queue_get_consumer_after_release_succeeds() {
    spsc_queue_tests::spsc_queue_get_consumer_after_release_succeeds();
}

#[inventory_test]
fn spsc_queue_get_producer_twice_fails() {
    spsc_queue_tests::spsc_queue_get_producer_twice_fails();
}

#[inventory_test]
fn spsc_queue_get_producer_after_release_succeeds() {
    spsc_queue_tests::spsc_queue_get_producer_after_release_succeeds();
}

#[inventory_test]
fn spsc_queue_push_pop_works_concurrently() {
    spsc_queue_tests::spsc_queue_push_pop_works_concurrently();
}
