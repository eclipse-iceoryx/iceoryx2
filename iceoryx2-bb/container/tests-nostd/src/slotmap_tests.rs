// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_container_tests_common::slotmap_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn new_slotmap_is_empty() {
    slotmap_tests::new_slotmap_is_empty();
}

#[inventory_test]
fn new_fixed_size_slotmap_is_empty() {
    slotmap_tests::new_fixed_size_slotmap_is_empty();
}

#[inventory_test]
fn inserting_elements_works() {
    slotmap_tests::inserting_elements_works();
}

#[inventory_test]
fn insert_when_full_fails() {
    slotmap_tests::insert_when_full_fails();
}

#[inventory_test]
fn removing_elements_works() {
    slotmap_tests::removing_elements_works();
}

#[inventory_test]
fn removing_out_of_bounds_key_returns_false() {
    slotmap_tests::removing_out_of_bounds_key_returns_false();
}

#[inventory_test]
fn insert_at_works() {
    slotmap_tests::insert_at_works();
}

#[inventory_test]
fn insert_at_and_remove_adjust_map_len_correctly() {
    slotmap_tests::insert_at_and_remove_adjust_map_len_correctly();
}

#[inventory_test]
fn insert_does_not_use_insert_at_indices() {
    slotmap_tests::insert_does_not_use_insert_at_indices();
}

#[inventory_test]
fn insert_at_out_of_bounds_key_returns_false() {
    slotmap_tests::insert_at_out_of_bounds_key_returns_false();
}

#[inventory_test]
fn iterating_works() {
    slotmap_tests::iterating_works();
}

#[inventory_test]
fn insert_remove_and_insert_works() {
    slotmap_tests::insert_remove_and_insert_works();
}

#[inventory_test]
fn next_free_key_returns_key_used_for_insert() {
    slotmap_tests::next_free_key_returns_key_used_for_insert();
}

#[inventory_test]
fn next_free_key_returns_none_when_full() {
    slotmap_tests::next_free_key_returns_none_when_full();
}

#[inventory_test]
fn placement_default_works() {
    slotmap_tests::placement_default_works();
}

#[inventory_test]
fn double_init_call_causes_panic() {
    slotmap_tests::double_init_call_causes_panic();
}

#[inventory_test]
fn panic_is_called_in_debug_mode_if_map_is_not_initialized() {
    slotmap_tests::panic_is_called_in_debug_mode_if_map_is_not_initialized();
}
