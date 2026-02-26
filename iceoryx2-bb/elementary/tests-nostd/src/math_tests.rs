// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2_bb_elementary_tests_common::math_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
pub fn math_align_returns_input_when_already_aligned() {
    math_tests::math_align_returns_input_when_already_aligned();
}

#[inventory_test]
pub fn math_align_returns_input_to_next_greater_value() {
    math_tests::math_align_returns_input_to_next_greater_value();
}

#[inventory_test]
pub fn math_dec_to_64() {
    math_tests::math_dec_to_64();
}

#[inventory_test]
pub fn const_max_works() {
    math_tests::const_max_works();
}
