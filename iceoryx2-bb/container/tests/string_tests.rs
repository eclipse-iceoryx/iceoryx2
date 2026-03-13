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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container_tests_common::string_tests;
use iceoryx2_bb_container_tests_common::string_tests::PolymorphicStringFactory;
use iceoryx2_bb_container_tests_common::string_tests::RelocatableStringFactory;
use iceoryx2_bb_container_tests_common::string_tests::StaticStringFactory;
use iceoryx2_bb_container_tests_common::string_tests::StringTestFactory;

#[generic_tests::define]
mod string {

    use super::*;

    #[test]
    fn new_string_is_empty<Factory: StringTestFactory>() {
        string_tests::new_string_is_empty::<Factory>();
    }

    #[test]
    fn capacity_is_correct<Factory: StringTestFactory>() {
        string_tests::capacity_is_correct::<Factory>();
    }

    #[test]
    fn push_valid_bytes_works<Factory: StringTestFactory>() {
        string_tests::push_valid_bytes_works::<Factory>();
    }

    #[test]
    fn push_invalid_byte_fails<Factory: StringTestFactory>() {
        string_tests::push_invalid_byte_fails::<Factory>();
    }

    #[test]
    fn push_into_full_string_fails<Factory: StringTestFactory>() {
        string_tests::push_into_full_string_fails::<Factory>();
    }

    #[test]
    fn as_bytes_or_str_returns_push_content<Factory: StringTestFactory>() {
        string_tests::as_bytes_or_str_returns_push_content::<Factory>();
    }

    #[test]
    fn clear_of_empty_string_does_nothing<Factory: StringTestFactory>() {
        string_tests::clear_of_empty_string_does_nothing::<Factory>();
    }

    #[test]
    fn clear_removes_all_contents<Factory: StringTestFactory>() {
        string_tests::clear_removes_all_contents::<Factory>();
    }

    #[test]
    fn find_of_character_in_empty_string_returns_none<Factory: StringTestFactory>() {
        string_tests::find_of_character_in_empty_string_returns_none::<Factory>();
    }

    #[test]
    fn find_of_range_in_empty_string_returns_none<Factory: StringTestFactory>() {
        string_tests::find_of_range_in_empty_string_returns_none::<Factory>();
    }

    #[test]
    fn find_of_non_existing_char_returns_none<Factory: StringTestFactory>() {
        string_tests::find_of_non_existing_char_returns_none::<Factory>();
    }

    #[test]
    fn find_returns_first_char_match_from_start<Factory: StringTestFactory>() {
        string_tests::find_returns_first_char_match_from_start::<Factory>();
    }

    #[test]
    fn find_of_char_located_at_the_beginning_works<Factory: StringTestFactory>() {
        string_tests::find_of_char_located_at_the_beginning_works::<Factory>();
    }

    #[test]
    fn find_of_char_located_in_the_middle_works<Factory: StringTestFactory>() {
        string_tests::find_of_char_located_in_the_middle_works::<Factory>();
    }

    #[test]
    fn find_of_char_located_at_the_end_works<Factory: StringTestFactory>() {
        string_tests::find_of_char_located_at_the_end_works::<Factory>();
    }

    #[test]
    fn find_of_range_located_at_the_beginning_works<Factory: StringTestFactory>() {
        string_tests::find_of_range_located_at_the_beginning_works::<Factory>();
    }

    #[test]
    fn find_of_range_located_in_the_middle_works<Factory: StringTestFactory>() {
        string_tests::find_of_range_located_in_the_middle_works::<Factory>();
    }

    #[test]
    fn find_of_range_located_at_the_end_works<Factory: StringTestFactory>() {
        string_tests::find_of_range_located_at_the_end_works::<Factory>();
    }

    #[test]
    fn find_where_range_is_equal_to_sut_works<Factory: StringTestFactory>() {
        string_tests::find_where_range_is_equal_to_sut_works::<Factory>();
    }

    #[test]
    fn insert_of_valid_character_at_the_beginning_works<Factory: StringTestFactory>() {
        string_tests::insert_of_valid_character_at_the_beginning_works::<Factory>();
    }

    #[test]
    fn insert_of_invalid_character_fails<Factory: StringTestFactory>() {
        string_tests::insert_of_invalid_character_fails::<Factory>();
    }

    #[test]
    fn insert_into_full_string_fails<Factory: StringTestFactory>() {
        string_tests::insert_into_full_string_fails::<Factory>();
    }

    #[test]
    fn insert_of_valid_character_in_the_middle_works<Factory: StringTestFactory>() {
        string_tests::insert_of_valid_character_in_the_middle_works::<Factory>();
    }

    #[test]
    fn insert_of_valid_character_at_the_end_works<Factory: StringTestFactory>() {
        string_tests::insert_of_valid_character_at_the_end_works::<Factory>();
    }

    #[test]
    fn insert_bytes_at_the_start_works<Factory: StringTestFactory>() {
        string_tests::insert_bytes_at_the_start_works::<Factory>();
    }

    #[test]
    fn insert_bytes_in_the_middle_works<Factory: StringTestFactory>() {
        string_tests::insert_bytes_in_the_middle_works::<Factory>();
    }

    #[test]
    fn insert_bytes_at_the_end_works<Factory: StringTestFactory>() {
        string_tests::insert_bytes_at_the_end_works::<Factory>();
    }

    #[test]
    fn insert_bytes_when_it_would_exceed_capacity_fails<Factory: StringTestFactory>() {
        string_tests::insert_bytes_when_it_would_exceed_capacity_fails::<Factory>();
    }

    #[test]
    fn insert_bytes_with_invalid_characters_fails<Factory: StringTestFactory>() {
        string_tests::insert_bytes_with_invalid_characters_fails::<Factory>();
    }

    #[test]
    fn insert_bytes_with_valid_characters_works<Factory: StringTestFactory>() {
        string_tests::insert_bytes_with_valid_characters_works::<Factory>();
    }

    #[should_panic]
    #[test]
    fn insert_bytes_out_of_bounds_panics<Factory: StringTestFactory>() {
        string_tests::insert_bytes_out_of_bounds_panics::<Factory>();
    }

    #[test]
    fn pop_removes_the_last_element<Factory: StringTestFactory>() {
        string_tests::pop_removes_the_last_element::<Factory>();
    }

    #[test]
    fn push_bytes_with_invalid_characters_fails<Factory: StringTestFactory>() {
        string_tests::push_bytes_with_invalid_characters_fails::<Factory>();
    }

    #[test]
    fn push_bytes_with_valid_characters_works<Factory: StringTestFactory>() {
        string_tests::push_bytes_with_valid_characters_works::<Factory>();
    }

    #[test]
    fn push_bytes_fails_when_it_exceeds_the_capacity<Factory: StringTestFactory>() {
        string_tests::push_bytes_fails_when_it_exceeds_the_capacity::<Factory>();
    }

    #[test]
    fn push_multiple_valid_bytes_works<Factory: StringTestFactory>() {
        string_tests::push_multiple_valid_bytes_works::<Factory>();
    }

    #[test]
    fn remove_first_character_works<Factory: StringTestFactory>() {
        string_tests::remove_first_character_works::<Factory>();
    }

    #[test]
    fn remove_last_character_works<Factory: StringTestFactory>() {
        string_tests::remove_last_character_works::<Factory>();
    }

    #[test]
    fn remove_non_existing_entry_returns_none<Factory: StringTestFactory>() {
        string_tests::remove_non_existing_entry_returns_none::<Factory>();
    }

    #[test]
    fn remove_non_existing_range_returns_false<Factory: StringTestFactory>() {
        string_tests::remove_non_existing_range_returns_false::<Factory>();
    }

    #[test]
    fn remove_non_existing_range_from_non_empty_string_returns_false<Factory: StringTestFactory>() {
        string_tests::remove_non_existing_range_from_non_empty_string_returns_false::<Factory>();
    }

    #[test]
    fn remove_full_range_ends_up_in_empty_string<Factory: StringTestFactory>() {
        string_tests::remove_full_range_ends_up_in_empty_string::<Factory>();
    }

    #[test]
    fn remove_range_from_start_works<Factory: StringTestFactory>() {
        string_tests::remove_range_from_start_works::<Factory>();
    }

    #[test]
    fn remove_range_from_center_works<Factory: StringTestFactory>() {
        string_tests::remove_range_from_center_works::<Factory>();
    }

    #[test]
    fn retain_works<Factory: StringTestFactory>() {
        string_tests::retain_works::<Factory>();
    }

    #[test]
    fn rfind_of_character_in_empty_string_returns_none<Factory: StringTestFactory>() {
        string_tests::rfind_of_character_in_empty_string_returns_none::<Factory>();
    }

    #[test]
    fn rfind_of_range_in_empty_string_returns_none<Factory: StringTestFactory>() {
        string_tests::rfind_of_range_in_empty_string_returns_none::<Factory>();
    }

    #[test]
    fn rfind_of_char_located_at_the_beginning_works<Factory: StringTestFactory>() {
        string_tests::rfind_of_char_located_at_the_beginning_works::<Factory>();
    }

    #[test]
    fn rfind_of_non_existing_char_returns_none<Factory: StringTestFactory>() {
        string_tests::rfind_of_non_existing_char_returns_none::<Factory>();
    }

    #[test]
    fn rfind_returns_first_char_match_from_end<Factory: StringTestFactory>() {
        string_tests::rfind_returns_first_char_match_from_end::<Factory>();
    }

    #[test]
    fn rfind_of_char_located_in_the_middle_works<Factory: StringTestFactory>() {
        string_tests::rfind_of_char_located_in_the_middle_works::<Factory>();
    }

    #[test]
    fn rfind_of_char_located_at_the_end_works<Factory: StringTestFactory>() {
        string_tests::rfind_of_char_located_at_the_end_works::<Factory>();
    }

    #[test]
    fn rfind_of_range_located_at_the_beginning_works<Factory: StringTestFactory>() {
        string_tests::rfind_of_range_located_at_the_beginning_works::<Factory>();
    }

    #[test]
    fn rfind_of_range_located_in_the_middle_works<Factory: StringTestFactory>() {
        string_tests::rfind_of_range_located_in_the_middle_works::<Factory>();
    }

    #[test]
    fn rfind_of_range_located_at_the_end_works<Factory: StringTestFactory>() {
        string_tests::rfind_of_range_located_at_the_end_works::<Factory>();
    }

    #[test]
    fn rfind_where_range_is_equal_to_sut_works<Factory: StringTestFactory>() {
        string_tests::rfind_where_range_is_equal_to_sut_works::<Factory>();
    }

    #[test]
    fn truncate_to_larger_string_does_nothing<Factory: StringTestFactory>() {
        string_tests::truncate_to_larger_string_does_nothing::<Factory>();
    }

    #[test]
    fn truncate_to_smaller_string_works<Factory: StringTestFactory>() {
        string_tests::truncate_to_smaller_string_works::<Factory>();
    }

    #[test]
    fn truncate_to_string_len_does_nothing<Factory: StringTestFactory>() {
        string_tests::truncate_to_string_len_does_nothing::<Factory>();
    }

    #[test]
    fn strip_prefix_from_empty_string_does_nothing<Factory: StringTestFactory>() {
        string_tests::strip_prefix_from_empty_string_does_nothing::<Factory>();
    }

    #[test]
    fn strip_non_existing_prefix_does_nothing<Factory: StringTestFactory>() {
        string_tests::strip_non_existing_prefix_does_nothing::<Factory>();
    }

    #[test]
    fn strip_existing_prefix_works<Factory: StringTestFactory>() {
        string_tests::strip_existing_prefix_works::<Factory>();
    }

    #[test]
    fn strip_existing_range_that_is_not_a_prefix_does_nothing<Factory: StringTestFactory>() {
        string_tests::strip_existing_range_that_is_not_a_prefix_does_nothing::<Factory>();
    }

    #[test]
    fn strip_non_existing_suffix_does_nothing<Factory: StringTestFactory>() {
        string_tests::strip_non_existing_suffix_does_nothing::<Factory>();
    }

    #[test]
    fn strip_existing_suffix_works<Factory: StringTestFactory>() {
        string_tests::strip_existing_suffix_works::<Factory>();
    }

    #[test]
    fn strip_existing_range_that_is_not_a_suffix_does_nothing<Factory: StringTestFactory>() {
        string_tests::strip_existing_range_that_is_not_a_suffix_does_nothing::<Factory>();
    }

    #[test]
    fn strip_suffix_from_empty_string_does_nothing<Factory: StringTestFactory>() {
        string_tests::strip_suffix_from_empty_string_does_nothing::<Factory>();
    }

    #[test]
    fn ordering_works<Factory: StringTestFactory>() {
        string_tests::ordering_works::<Factory>();
    }

    #[test]
    fn partial_ordering_works<Factory: StringTestFactory>() {
        string_tests::partial_ordering_works::<Factory>();
    }

    #[test]
    fn hash_works<Factory: StringTestFactory>() {
        string_tests::hash_works::<Factory>();
    }

    #[test]
    fn deref_mut_works<Factory: StringTestFactory>() {
        string_tests::deref_mut_works::<Factory>();
    }

    #[test]
    fn equality_works<Factory: StringTestFactory>() {
        string_tests::equality_works::<Factory>();
    }

    #[instantiate_tests(<PolymorphicStringFactory>)]
    mod polymorphic_string {}

    #[instantiate_tests(<RelocatableStringFactory>)]
    mod relocatable_string {}

    #[instantiate_tests(<StaticStringFactory>)]
    mod static_string {}
}

#[test]
fn error_display_works() {
    string_tests::error_display_works();
}
