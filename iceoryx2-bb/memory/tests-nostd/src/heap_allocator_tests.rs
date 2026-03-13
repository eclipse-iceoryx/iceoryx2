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

use iceoryx2_bb_memory::pool_allocator::AllocationError;
use iceoryx2_bb_memory_tests_common::heap_allocator_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn heap_allocator_allocate_deallocate_works() {
    heap_allocator_tests::heap_allocator_allocate_deallocate_works();
}

#[inventory_test]
fn heap_allocator_allocating_memory_with_size_of_zero_fails() {
    heap_allocator_tests::heap_allocator_allocating_memory_with_size_of_zero_fails();
}

#[inventory_test]
fn heap_allocator_allocate_zeroed_and_free_works() {
    heap_allocator_tests::heap_allocator_allocate_zeroed_and_free_works();
}

#[inventory_test]
fn heap_allocator_allocating_zeroed_memory_with_size_of_zero_fails() {
    heap_allocator_tests::heap_allocator_allocating_zeroed_memory_with_size_of_zero_fails();
}

#[inventory_test]
fn heap_allocator_grow_memory_keeps_content() {
    heap_allocator_tests::heap_allocator_grow_memory_keeps_content();
}

#[inventory_test]
fn heap_allocator_shrink_memory_keeps_content() {
    heap_allocator_tests::heap_allocator_shrink_memory_keeps_content();
}

#[inventory_test]
fn heap_allocator_shrink_memory_to_zero_fails() -> Result<(), AllocationError> {
    heap_allocator_tests::heap_allocator_shrink_memory_to_zero_fails()
}

#[inventory_test]
fn heap_allocator_grow_memory_with_increased_alignment_fails() -> Result<(), AllocationError> {
    heap_allocator_tests::heap_allocator_grow_memory_with_increased_alignment_fails()
}
