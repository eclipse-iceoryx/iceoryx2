// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary_tests_common::bump_allocator_tests;

#[test]
pub fn start_position_is_correctly_used() {
    bump_allocator_tests::start_position_is_correctly_used();
}

#[test]
pub fn allocated_memory_is_correctly_aligned() {
    bump_allocator_tests::allocated_memory_is_correctly_aligned();
}

#[test]
pub fn allocating_many_aligned_chunks_work() {
    bump_allocator_tests::allocating_many_aligned_chunks_work();
}

#[test]
pub fn deallocating_releases_everything() {
    bump_allocator_tests::deallocating_releases_everything();
}
