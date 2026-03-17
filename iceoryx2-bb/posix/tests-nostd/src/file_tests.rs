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

use iceoryx2_bb_posix::file::FileError;
use iceoryx2_bb_posix_tests_common::file_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
fn file_opening_non_existing_file_fails() {
    file_tests::file_opening_non_existing_file_fails();
}

#[inventory_test]
fn file_creating_non_existing_file_succeeds() {
    file_tests::file_creating_non_existing_file_succeeds();
}

#[inventory_test]
fn file_creating_existing_file_fails() {
    file_tests::file_creating_existing_file_fails();
}

#[inventory_test]
fn file_purge_and_create_non_existing_file_succeeds() {
    file_tests::file_purge_and_create_non_existing_file_succeeds();
}

#[inventory_test]
fn file_purge_and_create_existing_file_succeeds() {
    file_tests::file_purge_and_create_existing_file_succeeds();
}

#[inventory_test]
fn file_open_or_create_with_existing_file_succeeds() {
    file_tests::file_open_or_create_with_existing_file_succeeds();
}

#[inventory_test]
fn file_open_or_create_with_non_existing_file_succeeds() {
    file_tests::file_open_or_create_with_non_existing_file_succeeds();
}

#[inventory_test]
fn file_creating_file_applies_additional_settings() {
    file_tests::file_creating_file_applies_additional_settings();
}

#[inventory_test]
fn file_simple_read_write_works() {
    file_tests::file_simple_read_write_works();
}

#[inventory_test]
fn file_write_appends_content_to_file() {
    file_tests::file_write_appends_content_to_file();
}

#[inventory_test]
fn file_multiple_read_calls_move_file_cursor() {
    file_tests::file_multiple_read_calls_move_file_cursor();
}

#[inventory_test]
fn file_read_line_works() {
    file_tests::file_read_line_works();
}

#[inventory_test]
fn file_two_file_objects_read_work_with_ranges_in_same_file() {
    file_tests::file_two_file_objects_read_work_with_ranges_in_same_file();
}

#[inventory_test]
fn file_created_file_does_exist() -> Result<(), FileError> {
    file_tests::file_created_file_does_exist()
}

#[inventory_test]
fn file_truncate_works() -> Result<(), FileError> {
    file_tests::file_truncate_works()
}

#[inventory_test]
fn file_non_existing_file_does_not_exist() -> Result<(), FileError> {
    file_tests::file_non_existing_file_does_not_exist()
}

#[inventory_test]
fn file_remove_returns_true_when_file_exists() -> Result<(), FileError> {
    file_tests::file_remove_returns_true_when_file_exists()
}

#[inventory_test]
fn file_remove_returns_false_when_file_not_exists() -> Result<(), FileError> {
    file_tests::file_remove_returns_false_when_file_not_exists()
}

#[inventory_test]
fn file_newly_created_file_is_removed_when_it_has_ownership() -> Result<(), FileError> {
    file_tests::file_newly_created_file_is_removed_when_it_has_ownership()
}

#[inventory_test]
fn file_newly_created_file_has_not_ownership_by_default() -> Result<(), FileError> {
    file_tests::file_newly_created_file_has_not_ownership_by_default()
}

#[inventory_test]
fn file_opened_file_is_removed_when_it_has_ownership() -> Result<(), FileError> {
    file_tests::file_opened_file_is_removed_when_it_has_ownership()
}

#[inventory_test]
fn file_opened_file_has_not_ownership_by_default() -> Result<(), FileError> {
    file_tests::file_opened_file_has_not_ownership_by_default()
}

#[inventory_test]
fn file_acquire_ownership_works() -> Result<(), FileError> {
    file_tests::file_acquire_ownership_works()
}

#[inventory_test]
fn file_release_ownership_works() -> Result<(), FileError> {
    file_tests::file_release_ownership_works()
}
