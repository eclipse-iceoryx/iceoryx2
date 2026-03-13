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

use iceoryx2_bb_container_tests_common::queue_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn relocatable_push_pop_works_with_uninitialized_memory() {
    queue_tests::relocatable_push_pop_works_with_uninitialized_memory();
}

#[inventory_test]
fn relocatable_clear_empties_queue() {
    queue_tests::relocatable_clear_empties_queue();
}

#[inventory_test]
fn capacity_is_correct() {
    queue_tests::capacity_is_correct();
}

#[inventory_test]
fn newly_created_buffer_is_empty() {
    queue_tests::newly_created_buffer_is_empty();
}

#[inventory_test]
fn push_pop_works() {
    queue_tests::push_pop_works();
}

#[inventory_test]
fn valid_after_move() {
    queue_tests::valid_after_move();
}

#[inventory_test]
fn push_pop_alteration_works() {
    queue_tests::push_pop_alteration_works();
}

#[inventory_test]
fn clear_works() {
    queue_tests::clear_works();
}

#[inventory_test]
fn overflow_works() {
    queue_tests::overflow_works();
}

#[inventory_test]
fn iterate_with_get() {
    queue_tests::iterate_with_get();
}

#[inventory_test]
fn drops_all_objects_when_out_of_scope() {
    queue_tests::drops_all_objects_when_out_of_scope();
}

#[inventory_test]
fn drops_all_objects_with_clear() {
    queue_tests::drops_all_objects_with_clear();
}

#[inventory_test]
fn pop_releases_object() {
    queue_tests::pop_releases_object();
}

#[inventory_test]
fn queue_clear_drops_all_objects() {
    queue_tests::queue_clear_drops_all_objects();
}

#[inventory_test]
fn fixed_size_queue_clear_drops_all_objects() {
    queue_tests::fixed_size_queue_clear_drops_all_objects();
}

#[inventory_test]
fn get_invalid_index_panics() {
    queue_tests::get_invalid_index_panics();
}

#[inventory_test]
fn get_unchecked_works() {
    queue_tests::get_unchecked_works();
}

#[inventory_test]
fn placement_default_works() {
    queue_tests::placement_default_works();
}

#[inventory_test]
fn peek_works() {
    queue_tests::peek_works();
}

#[inventory_test]
fn double_init_call_causes_panic() {
    queue_tests::double_init_call_causes_panic();
}

#[inventory_test]
fn panic_is_called_in_debug_mode_if_queue_is_not_initialized() {
    queue_tests::panic_is_called_in_debug_mode_if_queue_is_not_initialized();
}
