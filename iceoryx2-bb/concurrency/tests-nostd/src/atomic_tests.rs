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

use iceoryx2_bb_concurrency_tests_common::atomic_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test_generic;

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_new_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_new_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_as_ptr_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_as_ptr_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compare_exchange_success_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compare_exchange_success_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compare_exchange_weak_success_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compare_exchange_weak_success_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compare_exchange_failure_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compare_exchange_failure_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compare_exchange_weak_failure_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compare_exchange_weak_failure_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_add_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_add_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_and_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_and_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_max_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_max_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_min_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_min_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_nand_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_nand_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_or_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_or_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_sub_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_sub_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_update_success_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_update_success_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_update_failure_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_update_failure_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_fetch_xor_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_fetch_xor_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_into_inner_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_into_inner_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_load_store_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_load_store_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_swap_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_swap_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_new_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_new_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_as_ptr_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_as_ptr_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_compare_exchange_success_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_compare_exchange_success_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_compare_exchange_weak_success_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_compare_exchange_weak_success_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_compare_exchange_failure_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_compare_exchange_failure_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_compare_exchange_weak_failure_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_compare_exchange_weak_failure_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_add_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_add_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_and_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_and_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_max_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_max_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_min_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_min_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_nand_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_nand_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_or_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_or_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_sub_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_sub_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_update_success_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_update_success_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_update_failure_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_update_failure_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_fetch_xor_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_fetch_xor_works::<T>();
}

#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_compatibility_swap_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_compatibility_swap_works::<T>();
}
#[inventory_test_generic(u64, u128, i64, i128)]
fn atomic_placement_default_works<T: atomic_tests::Req>() {
    atomic_tests::atomic_placement_default_works::<T>();
}
