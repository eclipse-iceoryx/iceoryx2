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

use iceoryx2_bb_container_tests_common::queue_tests;

#[test]
fn relocatable_push_pop_works_with_uninitialized_memory() {
    queue_tests::relocatable_push_pop_works_with_uninitialized_memory();
}

#[test]
fn relocatable_clear_empties_queue() {
    queue_tests::relocatable_clear_empties_queue();
}

#[test]
fn capacity_is_correct() {
    queue_tests::capacity_is_correct();
}

#[test]
fn newly_created_buffer_is_empty() {
    queue_tests::newly_created_buffer_is_empty();
}

#[test]
fn push_pop_works() {
    queue_tests::push_pop_works();
}

#[test]
fn valid_after_move() {
    queue_tests::valid_after_move();
}

#[test]
fn push_pop_alteration_works() {
    queue_tests::push_pop_alteration_works();
}

#[test]
fn clear_works() {
    queue_tests::clear_works();
}

#[test]
fn overflow_works() {
    queue_tests::overflow_works();
}

#[test]
fn iterate_with_get() {
    queue_tests::iterate_with_get();
}

#[test]
fn drops_all_objects_when_out_of_scope() {
    queue_tests::drops_all_objects_when_out_of_scope();
}

#[test]
fn drops_all_objects_with_clear() {
    queue_tests::drops_all_objects_with_clear();
}

#[test]
fn pop_releases_object() {
    queue_tests::pop_releases_object();
}

#[test]
fn queue_clear_drops_all_objects() {
    queue_tests::queue_clear_drops_all_objects();
}

#[test]
fn fixed_size_queue_clear_drops_all_objects() {
    queue_tests::fixed_size_queue_clear_drops_all_objects();
}

#[test]
#[should_panic]
fn get_invalid_index_panics() {
    queue_tests::get_invalid_index_panics();
}

#[test]
fn get_unchecked_works() {
    queue_tests::get_unchecked_works();
}

#[test]
fn placement_default_works() {
    queue_tests::placement_default_works();
}

#[test]
fn peek_works() {
    queue_tests::peek_works();
}

#[test]
#[should_panic]
fn double_init_call_causes_panic() {
    queue_tests::double_init_call_causes_panic();
}

#[cfg(debug_assertions)]
#[test]
#[should_panic]
fn panic_is_called_in_debug_mode_if_queue_is_not_initialized() {
    queue_tests::panic_is_called_in_debug_mode_if_queue_is_not_initialized();
}
