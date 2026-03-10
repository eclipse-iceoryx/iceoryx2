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

use iceoryx2_bb_system_types_tests_common::path_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[cfg(target_os = "windows")]
mod windows {
    use super::*;

    #[inventory_test]
    fn path_new_with_illegal_name_fails() {
        path_tests::path_new_with_illegal_name_fails();
    }

    #[inventory_test]
    fn path_new_with_legal_name_works() {
        path_tests::path_new_with_legal_name_works();
    }

    #[inventory_test]
    fn path_add_works() {
        path_tests::path_add_works();
    }

    #[inventory_test]
    fn path_is_absolute_works() {
        path_tests::path_is_absolute_works();
    }
}

#[cfg(not(target_os = "windows"))]
mod unix {
    use super::*;

    #[inventory_test]
    fn path_new_with_illegal_name_fails() {
        path_tests::path_new_with_illegal_name_fails();
    }

    #[inventory_test]
    fn path_new_with_legal_name_works() {
        path_tests::path_new_with_legal_name_works();
    }

    #[inventory_test]
    fn path_add_works() {
        path_tests::path_add_works();
    }

    #[inventory_test]
    fn path_list_all_entries_works() {
        path_tests::path_list_all_entries_works();
    }

    #[inventory_test]
    fn path_is_absolute_works() {
        path_tests::path_is_absolute_works();
    }
}
