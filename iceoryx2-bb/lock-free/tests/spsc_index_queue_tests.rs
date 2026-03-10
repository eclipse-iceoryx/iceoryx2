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

use iceoryx2_bb_lock_free_tests_common::spsc_index_queue_tests;

#[test]
fn spsc_index_queue_push_works_until_full() {
    spsc_index_queue_tests::spsc_index_queue_push_works_until_full();
}

#[test]
fn spsc_index_queue_pop_works_until_empty() {
    spsc_index_queue_tests::spsc_index_queue_pop_works_until_empty();
}

#[test]
fn spsc_index_queue_push_pop_alteration_works() {
    spsc_index_queue_tests::spsc_index_queue_push_pop_alteration_works();
}

#[test]
fn spsc_index_queue_get_consumer_twice_fails() {
    spsc_index_queue_tests::spsc_index_queue_get_consumer_twice_fails();
}

#[test]
fn spsc_index_queue_get_consumer_after_release_succeeds() {
    spsc_index_queue_tests::spsc_index_queue_get_consumer_after_release_succeeds();
}

#[test]
fn spsc_index_queue_get_producer_twice_fails() {
    spsc_index_queue_tests::spsc_index_queue_get_producer_twice_fails();
}

#[test]
fn spsc_index_queue_get_producer_after_release_succeeds() {
    spsc_index_queue_tests::spsc_index_queue_get_producer_after_release_succeeds();
}

#[test]
fn spsc_index_queue_push_pop_works_concurrently() {
    spsc_index_queue_tests::spsc_index_queue_push_pop_works_concurrently();
}
