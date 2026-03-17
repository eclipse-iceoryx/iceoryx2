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

use iceoryx2_bb_posix_tests_common::shared_memory_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn shared_memory_create_and_open_works() {
    shared_memory_tests::shared_memory_create_and_open_works();
}

#[inventory_test]
fn shared_memory_create_and_modify_open_works() {
    shared_memory_tests::shared_memory_create_and_modify_open_works();
}

#[inventory_test]
fn shared_memory_opening_with_non_fitting_size_fails() {
    shared_memory_tests::shared_memory_opening_with_non_fitting_size_fails();
}

#[inventory_test]
fn shared_memory_release_ownership_works() {
    shared_memory_tests::shared_memory_release_ownership_works();
}

#[inventory_test]
fn shared_memory_create_without_ownership_works() {
    shared_memory_tests::shared_memory_create_without_ownership_works();
}

#[inventory_test]
fn shared_memory_acquire_ownership_works() {
    shared_memory_tests::shared_memory_acquire_ownership_works();
}

#[inventory_test]
fn shared_memory_existing_shm_can_be_listed() {
    shared_memory_tests::shared_memory_existing_shm_can_be_listed();
}

#[inventory_test]
fn shared_memory_can_be_mapped_with_a_custom_offset() {
    shared_memory_tests::shared_memory_can_be_mapped_with_a_custom_offset();
}
