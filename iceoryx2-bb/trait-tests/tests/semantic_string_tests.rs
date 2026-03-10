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

use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_system_types::base64url::*;
use iceoryx2_bb_system_types::file_name::*;
use iceoryx2_bb_system_types::file_path::*;
use iceoryx2_bb_system_types::group_name::*;
use iceoryx2_bb_system_types::path::*;
use iceoryx2_bb_system_types::user_name::*;

use iceoryx2_bb_trait_tests_common::semantic_string_tests;

#[test]
fn display_error_enum_works() {
    semantic_string_tests::display_error_enum_works();
}

#[generic_tests::define]
mod semantic_string {

    use super::*;

    #[test]
    fn new_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::new_works::<CAPACITY, Sut>();
    }

    #[test]
    fn new_name_with_illegal_char_is_illegal<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY>,
    >() {
        semantic_string_tests::new_name_with_illegal_char_is_illegal::<CAPACITY, Sut>();
    }

    #[test]
    fn try_from_legal_str_succeeds<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY> + TryFrom<&'static str>,
    >() {
        semantic_string_tests::try_from_legal_str_succeeds::<CAPACITY, Sut>();
    }

    #[test]
    fn try_from_illegal_str_fails<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY> + TryFrom<&'static str>,
    >() {
        semantic_string_tests::try_from_illegal_str_fails::<CAPACITY, Sut>();
    }

    #[test]
    fn insert_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::insert_works::<CAPACITY, Sut>();
    }

    #[test]
    fn insert_illegal_character_fails<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::insert_illegal_character_fails::<CAPACITY, Sut>();
    }

    #[test]
    fn insert_bytes_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::insert_bytes_works::<CAPACITY, Sut>();
    }

    #[test]
    fn insert_bytes_with_illegal_character_fails<
        const CAPACITY: usize,
        Sut: SemanticString<CAPACITY>,
    >() {
        semantic_string_tests::insert_bytes_with_illegal_character_fails::<CAPACITY, Sut>();
    }

    #[test]
    fn pop_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::pop_works::<CAPACITY, Sut>();
    }

    #[test]
    fn remove_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::remove_works::<CAPACITY, Sut>();
    }

    #[test]
    fn remove_range_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::remove_range_works::<CAPACITY, Sut>();
    }

    #[test]
    fn retain_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::retain_works::<CAPACITY, Sut>();
    }

    #[test]
    fn strip_prefix_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::strip_prefix_works::<CAPACITY, Sut>();
    }

    #[test]
    fn strip_suffix_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::strip_suffix_works::<CAPACITY, Sut>();
    }

    #[test]
    fn truncate_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::truncate_works::<CAPACITY, Sut>();
    }

    #[test]
    fn invalid_utf8_characters_fail<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::invalid_utf8_characters_fail::<CAPACITY, Sut>();
    }

    #[test]
    fn is_full_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::is_full_works::<CAPACITY, Sut>();
    }

    #[test]
    fn capacity_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::capacity_works::<CAPACITY, Sut>();
    }

    #[test]
    fn insert_too_much_bytes_fails<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::insert_too_much_bytes_fails::<CAPACITY, Sut>();
    }

    #[test]
    fn pop_until_empty_works<const CAPACITY: usize, Sut: SemanticString<CAPACITY>>() {
        semantic_string_tests::pop_until_empty_works::<CAPACITY, Sut>();
    }

    #[instantiate_tests(<{FileName::max_len()}, FileName>)]
    mod file_name {}

    #[instantiate_tests(<64, RestrictedFileName::<64>>)]
    mod restricted_file_name {}

    #[instantiate_tests(<{Path::max_len()}, Path>)]
    mod path {}

    #[instantiate_tests(<{FilePath::max_len()}, FilePath>)]
    mod file_path {}

    #[instantiate_tests(<{UserName::max_len()}, UserName>)]
    mod user_name {}

    #[instantiate_tests(<{GroupName::max_len()}, GroupName>)]
    mod group_name {}

    #[instantiate_tests(<{Base64Url::max_len()}, Base64Url>)]
    mod base64url {}
}
