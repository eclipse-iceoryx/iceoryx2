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

use iceoryx2_bb_system_types_tests_common::file_path_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    #[inventory_test]
    fn file_path_new_with_illegal_name_fails() {
        file_path_tests::file_path_new_with_illegal_name_fails();
    }

    #[inventory_test]
    fn file_path_new_with_legal_name_works() {
        file_path_tests::file_path_new_with_legal_name_works();
    }

    #[inventory_test]
    fn file_path_from_legal_path_and_file_works() {
        file_path_tests::file_path_from_legal_path_and_file_works();
    }

    #[inventory_test]
    fn file_path_extract_file_name_works() {
        file_path_tests::file_path_extract_file_name_works();
    }

    #[inventory_test]
    fn file_path_extract_path_works() {
        file_path_tests::file_path_extract_path_works();
    }
}

#[cfg(not(target_os = "windows"))]
mod unix {
    use super::*;

    #[inventory_test]
    fn file_path_new_with_illegal_name_fails() {
        file_path_tests::file_path_new_with_illegal_name_fails();
    }

    #[inventory_test]
    fn file_path_new_with_legal_name_works() {
        file_path_tests::file_path_new_with_legal_name_works();
    }

    #[inventory_test]
    fn file_path_from_legal_path_and_file_works() {
        file_path_tests::file_path_from_legal_path_and_file_works();
    }

    #[inventory_test]
    fn file_path_extract_file_name_works() {
        file_path_tests::file_path_extract_file_name_works();
    }

    #[inventory_test]
    fn file_path_extract_path_works() {
        file_path_tests::file_path_extract_path_works();
    }
}

#[inventory_test]
fn file_path_new_with_illegal_relative_name_fails() {
    file_path_tests::file_path_new_with_illegal_relative_name_fails();
}

#[inventory_test]
fn file_path_from_empty_path_and_legal_file_works() {
    file_path_tests::file_path_from_empty_path_and_legal_file_works();
}

#[inventory_test]
fn file_path_extract_file_name_from_path_consisting_only_of_a_file_works() {
    file_path_tests::file_path_extract_file_name_from_path_consisting_only_of_a_file_works();
}

#[inventory_test]
fn file_path_extract_path_from_path_consisting_only_of_a_file_works() {
    file_path_tests::file_path_extract_path_from_path_consisting_only_of_a_file_works();
}
