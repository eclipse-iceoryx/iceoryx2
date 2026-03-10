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

use iceoryx2_bb_lock_free_tests_common::spmc_unrestricted_atomic_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn spmc_unrestricted_atomic_acquire_multiple_producer_fails() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_acquire_multiple_producer_fails();
}

#[inventory_test]
fn spmc_unrestricted_atomic_load_store_works() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_load_store_works();
}

#[inventory_test]
fn spmc_unrestricted_atomic_load_store_works_concurrently() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_load_store_works_concurrently();
}

#[inventory_test]
fn spmc_unrestricted_atomic_get_ptr_write_and_update_works() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_get_ptr_write_and_update_works();
}

#[inventory_test]
fn spmc_unrestricted_atomic_get_ptr_write_and_update_works_concurrently() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_get_ptr_write_and_update_works_concurrently();
}

#[inventory_test]
fn spmc_unrestricted_atomic_get_write_cell_works() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_get_write_cell_works();
}

#[inventory_test]
fn spmc_unrestricted_atomic_mgmt_release_producer_allows_new_acquire() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_mgmt_release_producer_allows_new_acquire();
}

#[inventory_test]
fn spmc_unrestricted_atomic_mgmt_get_ptr_write_and_update_works() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_mgmt_get_ptr_write_and_update_works();
}

#[inventory_test]
fn spmc_unrestricted_atomic_internal_ptr_calculation_works_with_integers() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_internal_ptr_calculation_works_with_integers();
}

#[inventory_test]
fn spmc_unrestricted_atomic_internal_get_data_cell_with_integers() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_internal_get_data_cell_with_integers();
}

#[inventory_test]
fn spmc_unrestricted_atomic_internal_size_and_alignment_calculation_with_integers() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_internal_size_and_alignment_calculation_with_integers();
}

#[inventory_test]
fn spmc_unrestricted_atomic_mgmt_get_write_cell_works() {
    spmc_unrestricted_atomic_tests::spmc_unrestricted_atomic_mgmt_get_write_cell_works();
}
