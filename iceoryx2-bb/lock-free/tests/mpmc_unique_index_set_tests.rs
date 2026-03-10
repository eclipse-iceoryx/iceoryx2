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

use iceoryx2_bb_lock_free_tests_common::mpmc_unique_index_set_tests;

#[test]
fn mpmc_unique_index_set_capacity_is_set_correctly() {
    mpmc_unique_index_set_tests::mpmc_unique_index_set_capacity_is_set_correctly();
}

#[test]
fn mpmc_unique_index_set_when_created_contains_indices() {
    mpmc_unique_index_set_tests::mpmc_unique_index_set_when_created_contains_indices();
}

#[test]
fn mpmc_unique_index_release_mode_default_does_not_lock() {
    mpmc_unique_index_set_tests::mpmc_unique_index_release_mode_default_does_not_lock();
}

#[test]
fn mpmc_unique_index_release_mode_lock_if_last_index_works() {
    mpmc_unique_index_set_tests::mpmc_unique_index_release_mode_lock_if_last_index_works();
}

#[test]
fn mpmc_unique_index_set_acquire_and_release_works() {
    mpmc_unique_index_set_tests::mpmc_unique_index_set_acquire_and_release_works();
}

#[test]
fn mpmc_unique_index_set_borrowed_indices_works() {
    mpmc_unique_index_set_tests::mpmc_unique_index_set_borrowed_indices_works();
}

#[test]
fn mpmc_unique_index_set_acquire_and_release_works_with_uninitialized_memory() {
    mpmc_unique_index_set_tests::mpmc_unique_index_set_acquire_and_release_works_with_uninitialized_memory();
}

#[test]
fn mpmc_unique_index_set_acquire_release_as_lifo_behavior() {
    mpmc_unique_index_set_tests::mpmc_unique_index_set_acquire_release_as_lifo_behavior();
}

#[test]
fn mpmc_unique_index_set_concurrent_acquire_release() {
    mpmc_unique_index_set_tests::mpmc_unique_index_set_concurrent_acquire_release();
}
