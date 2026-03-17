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

use iceoryx2_bb_posix_tests_common::directory_tests;

#[test]
fn directory_test_directory_does_exist() {
    directory_tests::directory_test_directory_does_exist();
}

#[test]
fn directory_non_existing_directory_does_not_exist() {
    directory_tests::directory_non_existing_directory_does_not_exist();
}

#[test]
fn directory_file_is_not_a_directory() {
    directory_tests::directory_file_is_not_a_directory();
}

#[test]
fn directory_create_from_path_works() {
    directory_tests::directory_create_from_path_works();
}

#[test]
fn directory_create_from_path_works_recursively() {
    directory_tests::directory_create_from_path_works_recursively();
}

#[test]
fn directory_create_from_path_is_thread_safe() {
    directory_tests::directory_create_from_path_is_thread_safe();
}

#[test]
fn directory_open_from_path_works() {
    directory_tests::directory_open_from_path_works();
}

#[test]
fn directory_list_contents_works() {
    directory_tests::directory_list_contents_works();
}
