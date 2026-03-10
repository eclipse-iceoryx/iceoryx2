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

use iceoryx2_bb_container_tests_common::vector_tests;
use iceoryx2_bb_container_tests_common::vector_tests::PolymorphicVecFactory;
use iceoryx2_bb_container_tests_common::vector_tests::RelocatableVecFactory;
use iceoryx2_bb_container_tests_common::vector_tests::StaticVecFactory;
use iceoryx2_bb_container_tests_common::vector_tests::VectorTestFactory;

#[generic_tests::define]
mod vector {

    use super::*;

    #[test]
    fn new_created_vec_is_empty<Factory: VectorTestFactory>() {
        vector_tests::new_created_vec_is_empty::<Factory>();
    }

    #[test]
    fn push_adds_element_at_the_end<Factory: VectorTestFactory>() {
        vector_tests::push_adds_element_at_the_end::<Factory>();
    }

    #[test]
    fn push_until_full_works<Factory: VectorTestFactory>() {
        vector_tests::push_until_full_works::<Factory>();
    }

    #[test]
    fn push_more_elements_than_capacity_fails<Factory: VectorTestFactory>() {
        vector_tests::push_more_elements_than_capacity_fails::<Factory>();
    }

    #[test]
    fn push_pop_alteration_works<Factory: VectorTestFactory>() {
        vector_tests::push_pop_alteration_works::<Factory>();
    }

    #[test]
    fn pop_returns_none_when_empty<Factory: VectorTestFactory>() {
        vector_tests::pop_returns_none_when_empty::<Factory>();
    }

    #[test]
    fn pop_removes_last_element<Factory: VectorTestFactory>() {
        vector_tests::pop_removes_last_element::<Factory>();
    }

    #[test]
    fn truncate_does_nothing_when_new_len_is_larger_than_current_len<Factory: VectorTestFactory>() {
        vector_tests::truncate_does_nothing_when_new_len_is_larger_than_current_len::<Factory>();
    }

    #[test]
    fn truncate_drops_all_elements_right_of_new_len<Factory: VectorTestFactory>() {
        vector_tests::truncate_drops_all_elements_right_of_new_len::<Factory>();
    }

    #[test]
    fn truncate_drops_elements_in_reverse_order<Factory: VectorTestFactory>() {
        vector_tests::truncate_drops_elements_in_reverse_order::<Factory>();
    }

    #[test]
    fn resize_increases_len_with_provided_value<Factory: VectorTestFactory>() {
        vector_tests::resize_increases_len_with_provided_value::<Factory>();
    }

    #[test]
    fn resize_reduces_len_and_drops_element_in_reverse_order<Factory: VectorTestFactory>() {
        vector_tests::resize_reduces_len_and_drops_element_in_reverse_order::<Factory>();
    }

    #[test]
    fn resize_fails_if_len_greater_than_capacity<Factory: VectorTestFactory>() {
        vector_tests::resize_fails_if_len_greater_than_capacity::<Factory>();
    }

    #[test]
    fn resize_with_increases_len_with_provided_value<Factory: VectorTestFactory>() {
        vector_tests::resize_with_increases_len_with_provided_value::<Factory>();
    }

    #[test]
    fn resize_with_reduces_len_and_drops_element_in_reverse_order<Factory: VectorTestFactory>() {
        vector_tests::resize_with_reduces_len_and_drops_element_in_reverse_order::<Factory>();
    }

    #[test]
    fn resize_with_calls_callback_only_for_the_newly_inserted_elements<
        Factory: VectorTestFactory,
    >() {
        vector_tests::resize_with_calls_callback_only_for_the_newly_inserted_elements::<Factory>();
    }

    #[test]
    fn resize_with_fails_if_len_greater_than_capacity<Factory: VectorTestFactory>() {
        vector_tests::resize_with_fails_if_len_greater_than_capacity::<Factory>();
    }

    #[test]
    fn remove_first_element_of_empty_vec_returns_none<Factory: VectorTestFactory>() {
        vector_tests::remove_first_element_of_empty_vec_returns_none::<Factory>();
    }

    #[test]
    fn remove_element_out_of_bounds_returns_none<Factory: VectorTestFactory>() {
        vector_tests::remove_element_out_of_bounds_returns_none::<Factory>();
    }

    #[test]
    fn remove_first_element_until_empty_works<Factory: VectorTestFactory>() {
        vector_tests::remove_first_element_until_empty_works::<Factory>();
    }

    #[test]
    fn remove_middle_element_works<Factory: VectorTestFactory>() {
        vector_tests::remove_middle_element_works::<Factory>();
    }

    #[test]
    fn insert_first_element_of_empty_vec_works<Factory: VectorTestFactory>() {
        vector_tests::insert_first_element_of_empty_vec_works::<Factory>();
    }

    #[test]
    fn insert_second_element_of_empty_vec_fails<Factory: VectorTestFactory>() {
        vector_tests::insert_second_element_of_empty_vec_fails::<Factory>();
    }

    #[test]
    fn insert_at_position_zero_fills_vector_in_reverse_order<Factory: VectorTestFactory>() {
        vector_tests::insert_at_position_zero_fills_vector_in_reverse_order::<Factory>();
    }

    #[test]
    fn insert_at_end_fills_vector_in_order<Factory: VectorTestFactory>() {
        vector_tests::insert_at_end_fills_vector_in_order::<Factory>();
    }

    #[test]
    fn insert_at_center_move_elements_to_the_rights<Factory: VectorTestFactory>() {
        vector_tests::insert_at_center_move_elements_to_the_rights::<Factory>();
    }

    #[test]
    fn insert_into_full_vec_fails<Factory: VectorTestFactory>() {
        vector_tests::insert_into_full_vec_fails::<Factory>();
    }

    #[test]
    fn clearing_empty_vector_does_nothing<Factory: VectorTestFactory>() {
        vector_tests::clearing_empty_vector_does_nothing::<Factory>();
    }

    #[test]
    fn clear_drops_elements_in_reverse_order<Factory: VectorTestFactory>() {
        vector_tests::clear_drops_elements_in_reverse_order::<Factory>();
    }

    #[test]
    fn as_slice_contains_elements<Factory: VectorTestFactory>() {
        vector_tests::as_slice_contains_elements::<Factory>();
    }

    #[test]
    fn as_mut_slice_contains_mutable_elements<Factory: VectorTestFactory>() {
        vector_tests::as_mut_slice_contains_mutable_elements::<Factory>();
    }

    #[test]
    fn adding_a_slice_to_empty_vec_that_exceeds_the_capacity_fails<Factory: VectorTestFactory>() {
        vector_tests::adding_a_slice_to_empty_vec_that_exceeds_the_capacity_fails::<Factory>();
    }

    #[test]
    fn adding_a_slice_to_empty_vec_that_has_the_same_capacity_works<Factory: VectorTestFactory>() {
        vector_tests::adding_a_slice_to_empty_vec_that_has_the_same_capacity_works::<Factory>();
    }

    #[test]
    fn adding_a_slice_to_filled_vec_appends_elements<Factory: VectorTestFactory>() {
        vector_tests::adding_a_slice_to_filled_vec_appends_elements::<Factory>();
    }

    #[test]
    fn when_vec_is_dropped_all_elements_are_dropped_in_reverse_order<Factory: VectorTestFactory>() {
        vector_tests::when_vec_is_dropped_all_elements_are_dropped_in_reverse_order::<Factory>();
    }

    #[instantiate_tests(<PolymorphicVecFactory>)]
    mod polymorphic_vec {}

    #[instantiate_tests(<RelocatableVecFactory>)]
    mod relocatable_vec {}

    #[instantiate_tests(<StaticVecFactory>)]
    mod static_vec {}
}
