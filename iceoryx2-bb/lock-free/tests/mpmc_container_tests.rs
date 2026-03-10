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

#[generic_tests::define]
mod mpmc_container {

    use iceoryx2_bb_lock_free_tests_common::mpmc_container_tests;

    #[test]
    fn mpmc_container_add_elements_until_full_works<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_add_elements_until_full_works::<T>();
    }

    #[test]
    fn mpmc_container_add_and_remove_elements_works<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_add_and_remove_elements_works::<T>();
    }

    #[test]
    fn mpmc_container_add_and_remove_elements_works_with_uninitialized_memory<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_add_and_remove_elements_works_with_uninitialized_memory::<T>();
    }

    #[test]
    fn mpmc_container_add_and_unsafe_remove_with_handle_works<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_add_and_unsafe_remove_with_handle_works::<T>();
    }

    #[test]
    fn mpmc_container_state_of_empty_container_is_empty<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_state_of_empty_container_is_empty::<T>();
    }

    #[test]
    fn mpmc_container_state_not_updated_when_contents_do_not_change<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_state_not_updated_when_contents_do_not_change::<T>();
    }

    #[test]
    fn mpmc_container_state_updated_when_contents_are_removed<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_state_updated_when_contents_are_removed::<T>();
    }

    #[test]
    fn mpmc_container_state_updated_when_contents_are_changed<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_state_updated_when_contents_are_changed::<T>();
    }

    #[test]
    fn mpmc_container_state_updated_works_for_new_and_removed_elements<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_state_updated_works_for_new_and_removed_elements::<T>(
        );
    }

    #[test]
    fn mpmc_container_state_updated_works_when_same_element_is_added_and_removed<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
    >() {
        mpmc_container_tests::mpmc_container_state_updated_works_when_same_element_is_added_and_removed::<T>();
    }

    #[test]
    fn mpmc_container_concurrent_add_release_for_each<
        T: core::fmt::Debug + Copy + From<usize> + Into<usize> + Send,
    >() {
        mpmc_container_tests::mpmc_container_concurrent_add_release_for_each::<T>();
    }

    #[instantiate_tests(<usize>)]
    mod usize {}

    #[instantiate_tests(<mpmc_container_tests::TestType>)]
    mod test_type {}
}
