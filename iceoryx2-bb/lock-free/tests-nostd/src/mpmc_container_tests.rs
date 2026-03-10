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

use iceoryx2_bb_lock_free_tests_common::mpmc_container_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test_generic;

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_add_elements_until_full_works<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_add_elements_until_full_works::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_add_and_remove_elements_works<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_add_and_remove_elements_works::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_add_and_remove_elements_works_with_uninitialized_memory<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_add_and_remove_elements_works_with_uninitialized_memory::<T>(
    );
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_add_and_unsafe_remove_with_handle_works<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_add_and_unsafe_remove_with_handle_works::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_state_of_empty_container_is_empty<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_state_of_empty_container_is_empty::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_state_not_updated_when_contents_do_not_change<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_state_not_updated_when_contents_do_not_change::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_state_updated_when_contents_are_removed<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_state_updated_when_contents_are_removed::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_state_updated_when_contents_are_changed<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_state_updated_when_contents_are_changed::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_state_updated_works_for_new_and_removed_elements<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_state_updated_works_for_new_and_removed_elements::<T>();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_state_updated_works_when_same_element_is_added_and_removed<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize>,
>() {
    mpmc_container_tests::mpmc_container_state_updated_works_when_same_element_is_added_and_removed::<
        T,
    >();
}

#[inventory_test_generic(usize, mpmc_container_tests::TestType)]
fn mpmc_container_concurrent_add_release_for_each<
    T: core::fmt::Debug + Copy + From<usize> + Into<usize> + Send,
>() {
    mpmc_container_tests::mpmc_container_concurrent_add_release_for_each::<T>();
}
