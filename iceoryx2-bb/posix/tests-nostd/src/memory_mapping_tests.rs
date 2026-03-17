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

#![allow(clippy::disallowed_types)]

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::memory_mapping_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
fn memory_mapping_mapping_anonymous_memory_works() {
    memory_mapping_tests::memory_mapping_mapping_anonymous_memory_works();
}

#[inventory_test]
fn memory_mapping_setting_permission_to_read_works() {
    memory_mapping_tests::memory_mapping_setting_permission_to_read_works();
}

#[inventory_test]
fn memory_mapping_mapping_file_works() {
    memory_mapping_tests::memory_mapping_mapping_file_works();
}

#[inventory_test]
fn memory_mapping_mapping_file_descriptor_works() {
    memory_mapping_tests::memory_mapping_mapping_file_descriptor_works();
}

#[inventory_test]
fn memory_mapping_mapping_size_of_zero_fails() {
    memory_mapping_tests::memory_mapping_mapping_size_of_zero_fails();
}

#[inventory_test]
fn memory_mapping_update_permissions_offset_fails_when_offset_is_not_multiple_of_page_size() {
    memory_mapping_tests::memory_mapping_update_permissions_offset_fails_when_offset_is_not_multiple_of_page_size(
    );
}

#[inventory_test]
fn memory_mapping_update_permissions_offset_fails_when_size_is_not_multiple_of_page_size() {
    memory_mapping_tests::memory_mapping_update_permissions_offset_fails_when_size_is_not_multiple_of_page_size();
}

#[inventory_test]
fn memory_mapping_update_permissions_offset_fails_when_size_is_zero() {
    memory_mapping_tests::memory_mapping_update_permissions_offset_fails_when_size_is_zero();
}

#[inventory_test]
fn memory_mapping_update_permissions_offset_fails_when_range_is_greater_than_mapped_range() {
    memory_mapping_tests::memory_mapping_update_permissions_offset_fails_when_range_is_greater_than_mapped_range();
}

#[inventory_test]
fn memory_mapping_fails_when_it_is_not_mapped_to_address_hint() {
    memory_mapping_tests::memory_mapping_fails_when_it_is_not_mapped_to_address_hint();
}
