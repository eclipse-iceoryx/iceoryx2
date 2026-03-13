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

use iceoryx2_bb_memory_tests_common::bump_allocator_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn bump_allocator_allocating_too_much_fails_with_out_of_memory() {
    bump_allocator_tests::bump_allocator_allocating_too_much_fails_with_out_of_memory();
}

#[inventory_test]
fn bump_allocator_allocating_all_memory_works() {
    bump_allocator_tests::bump_allocator_allocating_all_memory_works();
}

#[inventory_test]
fn bump_allocator_after_deallocate_allocating_all_memory_works() {
    bump_allocator_tests::bump_allocator_after_deallocate_allocating_all_memory_works();
}

#[inventory_test]
fn bump_allocator_used_free_and_total_space_work() {
    bump_allocator_tests::bump_allocator_used_free_and_total_space_work();
}

#[inventory_test]
fn bump_allocator_allocating_with_different_alignments_works() {
    bump_allocator_tests::bump_allocator_allocating_with_different_alignments_works();
}
