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

use iceoryx2_bb_posix::memory::MemoryError;
use iceoryx2_bb_posix_tests_common::memory_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
fn memory_allocate_and_deallocate_works() -> Result<(), MemoryError> {
    memory_tests::memory_allocate_and_deallocate_works()
}

#[inventory_test]
fn memory_allocating_memory_with_size_of_zero_fails() {
    memory_tests::memory_allocating_memory_with_size_of_zero_fails();
}

#[inventory_test]
fn memory_allocate_zeroed_and_free_works() -> Result<(), MemoryError> {
    memory_tests::memory_allocate_zeroed_and_free_works()
}

#[inventory_test]
fn memory_allocating_zeroed_memory_with_size_of_zero_fails() {
    memory_tests::memory_allocating_zeroed_memory_with_size_of_zero_fails();
}

#[inventory_test]
fn memory_increasing_memory_keeps_content() -> Result<(), MemoryError> {
    memory_tests::memory_increasing_memory_keeps_content()
}

#[inventory_test]
fn memory_decreasing_memory_keeps_content() -> Result<(), MemoryError> {
    memory_tests::memory_decreasing_memory_keeps_content()
}

#[inventory_test]
fn memory_decreasing_memory_to_zero_fails() -> Result<(), MemoryError> {
    memory_tests::memory_decreasing_memory_to_zero_fails()
}

#[inventory_test]
fn memory_resize_memory_with_increased_alignment_fails() -> Result<(), MemoryError> {
    memory_tests::memory_resize_memory_with_increased_alignment_fails()
}
