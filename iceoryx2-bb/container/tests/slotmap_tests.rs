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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::slotmap_tests;

#[test]
fn new_slotmap_is_empty() {
    slotmap_tests::new_slotmap_is_empty();
}

#[test]
fn new_fixed_size_slotmap_is_empty() {
    slotmap_tests::new_fixed_size_slotmap_is_empty();
}

#[test]
fn inserting_elements_works() {
    slotmap_tests::inserting_elements_works();
}

#[test]
fn insert_when_full_fails() {
    slotmap_tests::insert_when_full_fails();
}

#[test]
fn removing_elements_works() {
    slotmap_tests::removing_elements_works();
}

#[test]
fn removing_out_of_bounds_key_returns_false() {
    slotmap_tests::removing_out_of_bounds_key_returns_false();
}

#[test]
fn insert_at_works() {
    slotmap_tests::insert_at_works();
}

#[test]
fn insert_at_and_remove_adjust_map_len_correctly() {
    slotmap_tests::insert_at_and_remove_adjust_map_len_correctly();
}

#[test]
fn insert_does_not_use_insert_at_indices() {
    slotmap_tests::insert_does_not_use_insert_at_indices();
}

#[test]
fn insert_at_out_of_bounds_key_returns_false() {
    slotmap_tests::insert_at_out_of_bounds_key_returns_false();
}

#[test]
fn iterating_works() {
    slotmap_tests::iterating_works();
}

#[test]
fn insert_remove_and_insert_works() {
    slotmap_tests::insert_remove_and_insert_works();
}

#[test]
fn next_free_key_returns_key_used_for_insert() {
    slotmap_tests::next_free_key_returns_key_used_for_insert();
}

#[test]
fn next_free_key_returns_none_when_full() {
    slotmap_tests::next_free_key_returns_none_when_full();
}

#[test]
fn placement_default_works() {
    slotmap_tests::placement_default_works();
}

#[test]
#[should_panic]
fn double_init_call_causes_panic() {
    slotmap_tests::double_init_call_causes_panic();
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn panic_is_called_in_debug_mode_if_map_is_not_initialized() {
    slotmap_tests::panic_is_called_in_debug_mode_if_map_is_not_initialized();
}
