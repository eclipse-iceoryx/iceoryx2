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

use iceoryx2_bb_posix_tests_common::thread_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
fn thread_set_name_works() {
    thread_tests::thread_set_name_works();
}

#[inventory_test]
fn thread_creation_does_not_block() {
    thread_tests::thread_creation_does_not_block();
}

#[inventory_test]
fn thread_affinity_is_set_to_all_existing_cores_when_nothing_was_configured() {
    thread_tests::thread_affinity_is_set_to_all_existing_cores_when_nothing_was_configured();
}

#[inventory_test]
fn thread_set_affinity_to_one_cpu_core_on_creation_works() {
    thread_tests::thread_set_affinity_to_one_cpu_core_on_creation_works();
}

#[inventory_test]
fn thread_set_affinity_to_two_cpu_cores_on_creation_works() {
    thread_tests::thread_set_affinity_to_two_cpu_cores_on_creation_works();
}

#[inventory_test]
fn thread_set_affinity_to_non_existing_cpu_cores_on_creation_fails() {
    thread_tests::thread_set_affinity_to_non_existing_cpu_cores_on_creation_fails();
}

#[inventory_test]
fn thread_set_affinity_to_cores_greater_than_cpu_set_size_fails() {
    thread_tests::thread_set_affinity_to_cores_greater_than_cpu_set_size_fails();
}

#[inventory_test]
fn thread_set_affinity_to_one_core_from_handle_works() {
    thread_tests::thread_set_affinity_to_one_core_from_handle_works();
}

#[inventory_test]
fn thread_set_affinity_to_two_cores_from_handle_works() {
    thread_tests::thread_set_affinity_to_two_cores_from_handle_works();
}

#[inventory_test]
fn thread_set_affinity_to_non_existing_cores_from_handle_fails() {
    thread_tests::thread_set_affinity_to_non_existing_cores_from_handle_fails();
}

#[inventory_test]
fn thread_set_affinity_to_one_core_from_thread_works() {
    thread_tests::thread_set_affinity_to_one_core_from_thread_works();
}

#[inventory_test]
fn thread_set_affinity_to_two_cores_from_thread_works() {
    thread_tests::thread_set_affinity_to_two_cores_from_thread_works();
}

#[inventory_test]
fn thread_set_affinity_to_non_existing_cores_from_thread_fails() {
    thread_tests::thread_set_affinity_to_non_existing_cores_from_thread_fails();
}

#[inventory_test]
fn thread_destructor_does_not_block_on_empty_thread() {
    thread_tests::thread_destructor_does_not_block_on_empty_thread();
}

#[inventory_test]
fn thread_destructor_does_block_on_busy_thread() {
    thread_tests::thread_destructor_does_block_on_busy_thread();
}

#[inventory_test]
fn thread_scoped_threads_work() {
    thread_tests::thread_scoped_threads_work();
}
