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

use iceoryx2_bb_memory_tests_common::one_chunk_allocator_tests;

#[test]
fn one_chunk_allocator_acquire_works() {
    one_chunk_allocator_tests::one_chunk_allocator_acquire_works();
}

#[test]
fn one_chunk_allocator_acquire_with_alignment_works() {
    one_chunk_allocator_tests::one_chunk_allocator_acquire_with_alignment_works();
}

#[test]
fn one_chunk_allocator_allocate_zeroed_works() {
    one_chunk_allocator_tests::one_chunk_allocator_allocate_zeroed_works();
}

#[test]
fn one_chunk_allocator_shrink_works() {
    one_chunk_allocator_tests::one_chunk_allocator_shrink_works();
}

#[test]
fn one_chunk_allocator_shrink_fails_when_size_increases() {
    one_chunk_allocator_tests::one_chunk_allocator_shrink_fails_when_size_increases();
}

#[test]
fn one_chunk_allocator_shrink_fails_when_alignment_increases() {
    one_chunk_allocator_tests::one_chunk_allocator_shrink_fails_when_alignment_increases();
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn one_chunk_allocator_shrink_non_allocated_chunk_fails() {
    one_chunk_allocator_tests::one_chunk_allocator_shrink_non_allocated_chunk_fails();
}

#[test]
fn one_chunk_allocator_grow_works() {
    one_chunk_allocator_tests::one_chunk_allocator_grow_works();
}

#[test]
fn one_chunk_allocator_grow_zeroed_works() {
    one_chunk_allocator_tests::one_chunk_allocator_grow_zeroed_works();
}

#[test]
fn one_chunk_allocator_grow_with_decreased_size_fails() {
    one_chunk_allocator_tests::one_chunk_allocator_grow_with_decreased_size_fails();
}

#[test]
fn one_chunk_allocator_grow_with_increased_alignment_fails() {
    one_chunk_allocator_tests::one_chunk_allocator_grow_with_increased_alignment_fails();
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn one_chunk_allocator_grow_with_non_allocated_chunk_fails() {
    one_chunk_allocator_tests::one_chunk_allocator_grow_with_non_allocated_chunk_fails();
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn one_chunk_allocator_deallocate_non_allocated_chunk_fails() {
    one_chunk_allocator_tests::one_chunk_allocator_deallocate_non_allocated_chunk_fails();
}
