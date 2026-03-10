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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::flatmap_tests;

#[test]
fn new_creates_empty_flat_map() {
    flatmap_tests::new_creates_empty_flat_map();
}

#[test]
fn new_creates_empty_fixed_size_flat_map() {
    flatmap_tests::new_creates_empty_fixed_size_flat_map();
}

#[test]
fn default_creates_empty_flat_map() {
    flatmap_tests::default_creates_empty_flat_map();
}

#[test]
fn placement_default_works() {
    flatmap_tests::placement_default_works();
}

#[test]
fn drop_called_for_keys_and_values() {
    flatmap_tests::drop_called_for_keys_and_values();
}

#[test]
fn insert_into_empty_flat_map_works() {
    flatmap_tests::insert_into_empty_flat_map_works();
}

#[test]
fn insert_the_same_key_fails() {
    flatmap_tests::insert_the_same_key_fails();
}

#[test]
fn insert_until_full_works() {
    flatmap_tests::insert_until_full_works();
}

#[test]
fn get_value_from_flat_map_works() {
    flatmap_tests::get_value_from_flat_map_works();
}

#[test]
fn get_ref_value_from_flat_map_works() {
    flatmap_tests::get_ref_value_from_flat_map_works();
}

#[test]
fn get_mut_ref_value_from_flat_map_works() {
    flatmap_tests::get_mut_ref_value_from_flat_map_works();
}

#[test]
fn remove_keys_from_flat_map_works() {
    flatmap_tests::remove_keys_from_flat_map_works();
}

#[test]
fn remove_until_empty_and_reinsert_works() {
    flatmap_tests::remove_until_empty_and_reinsert_works();
}

#[test]
#[should_panic]
fn double_init_call_causes_panic() {
    flatmap_tests::double_init_call_causes_panic();
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn panic_is_called_in_debug_mode_if_map_is_not_initialized() {
    flatmap_tests::panic_is_called_in_debug_mode_if_map_is_not_initialized();
}

#[test]
fn list_keys_works_correctly() {
    flatmap_tests::list_keys_works_correctly();
}

#[test]
fn list_keys_works_correctly_on_empty_map() {
    flatmap_tests::list_keys_works_correctly_on_empty_map();
}

// BEGIN tests for passing custom compare function

#[test]
fn insert_the_same_key_fails_with_custom_cmp_func() {
    flatmap_tests::insert_the_same_key_fails_with_custom_cmp_func();
}

#[test]
fn insert_until_full_works_with_custom_cmp_func() {
    flatmap_tests::insert_until_full_works_with_custom_cmp_func();
}

#[test]
fn get_value_from_flat_map_works_with_custom_cmp_func() {
    flatmap_tests::get_value_from_flat_map_works_with_custom_cmp_func();
}

#[test]
fn get_ref_value_from_flat_map_works_with_custom_cmp_func() {
    flatmap_tests::get_ref_value_from_flat_map_works_with_custom_cmp_func();
}

#[test]
fn get_mut_ref_value_from_flat_map_works_with_custom_cmp_func() {
    flatmap_tests::get_mut_ref_value_from_flat_map_works_with_custom_cmp_func();
}

#[test]
fn remove_keys_from_flat_map_works_with_custom_cmp_func() {
    flatmap_tests::remove_keys_from_flat_map_works_with_custom_cmp_func();
}

#[test]
fn remove_until_empty_and_reinsert_works_with_custom_cmp_func() {
    flatmap_tests::remove_until_empty_and_reinsert_works_with_custom_cmp_func();
}

// END tests for passing custom compare function
