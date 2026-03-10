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

use iceoryx2_bb_memory_tests_common::pool_allocator_tests;

#[test]
fn pool_allocator_set_up_correctly() {
    pool_allocator_tests::pool_allocator_set_up_correctly();
}

#[test]
fn pool_allocator_acquire_all_memory_works() {
    pool_allocator_tests::pool_allocator_acquire_all_memory_works();
}

#[test]
fn pool_allocator_allocate_more_than_bucket_size_fails() {
    pool_allocator_tests::pool_allocator_allocate_more_than_bucket_size_fails();
}

#[test]
fn pool_allocator_allocate_more_than_bucket_alignment_fails() {
    pool_allocator_tests::pool_allocator_allocate_more_than_bucket_alignment_fails();
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn pool_allocator_deallocate_non_allocated_chunk_fails() {
    pool_allocator_tests::pool_allocator_deallocate_non_allocated_chunk_fails();
}

#[test]
fn pool_allocator_acquire_and_release_works() {
    pool_allocator_tests::pool_allocator_acquire_and_release_works();
}

#[test]
fn pool_allocator_acquire_too_large_sample_fails() {
    pool_allocator_tests::pool_allocator_acquire_too_large_sample_fails();
}

#[test]
fn pool_allocator_acquire_sample_with_to_large_alignment_fails() {
    pool_allocator_tests::pool_allocator_acquire_sample_with_to_large_alignment_fails();
}

#[test]
fn pool_allocator_allocate_zeroed_works() {
    pool_allocator_tests::pool_allocator_allocate_zeroed_works();
}

#[test]
fn pool_allocator_grow_works() {
    pool_allocator_tests::pool_allocator_grow_works();
}

#[test]
fn pool_allocator_grow_with_size_larger_bucket_fails() {
    pool_allocator_tests::pool_allocator_grow_with_size_larger_bucket_fails();
}

#[test]
fn pool_allocator_grow_with_size_decrease_fails() {
    pool_allocator_tests::pool_allocator_grow_with_size_decrease_fails();
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn pool_allocator_grow_with_non_allocated_chunk_fails() {
    pool_allocator_tests::pool_allocator_grow_with_non_allocated_chunk_fails();
}

#[test]
fn pool_allocator_grow_with_too_alignment_larger_bucket_alignment_fails() {
    pool_allocator_tests::pool_allocator_grow_with_too_alignment_larger_bucket_alignment_fails();
}

#[test]
fn pool_allocator_grow_zeroed_works() {
    pool_allocator_tests::pool_allocator_grow_zeroed_works();
}

#[test]
fn pool_allocator_shrink_works() {
    pool_allocator_tests::pool_allocator_shrink_works();
}

#[test]
fn pool_allocator_shrink_with_increased_size_fails() {
    pool_allocator_tests::pool_allocator_shrink_with_increased_size_fails();
}

#[test]
fn pool_allocator_shrink_with_alignment_larger_than_bucket_alignment_fails() {
    pool_allocator_tests::pool_allocator_shrink_with_alignment_larger_than_bucket_alignment_fails();
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn pool_allocator_shrink_non_allocated_chunk_fails() {
    pool_allocator_tests::pool_allocator_shrink_non_allocated_chunk_fails();
}

#[test]
fn pool_allocator_relocatable_acquire_all_memory_works() {
    pool_allocator_tests::pool_allocator_relocatable_acquire_all_memory_works();
}
