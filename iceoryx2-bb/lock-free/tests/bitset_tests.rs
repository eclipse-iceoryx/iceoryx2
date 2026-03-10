// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_lock_free_tests_common::bitset_tests;

#[test]
fn bit_set_create_fill_and_reset_works() {
    bitset_tests::bit_set_create_fill_and_reset_works();
}

#[test]
fn fixed_size_bit_set_create_fill_and_reset_works() {
    bitset_tests::fixed_size_bit_set_create_fill_and_reset_works();
}

#[test]
fn bit_set_set_single_bit_works() {
    bitset_tests::bit_set_set_single_bit_works();
}

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn bit_set_set_bit_outside_of_bitset_leads_to_panic() {
    bitset_tests::bit_set_set_bit_outside_of_bitset_leads_to_panic();
}

#[test]
fn bit_set_set_and_reset_next_works() {
    bitset_tests::bit_set_set_and_reset_next_works();
}

#[test]
fn bit_set_reset_next_is_fair() {
    bitset_tests::bit_set_reset_next_is_fair();
}

#[test]
fn bit_set_concurrent_set_and_reset_works() {
    bitset_tests::bit_set_concurrent_set_and_reset_works();
}
