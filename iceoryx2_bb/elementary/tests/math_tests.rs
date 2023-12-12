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

use iceoryx2_bb_elementary::math::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn math_align_returns_input_when_already_aligned() {
    assert_that!(align(25, 5), eq 25);
}

#[test]
fn math_align_returns_input_to_next_greater_value() {
    assert_that!(align(30, 7), eq 35);
}

#[test]
fn math_log2_of_power_of_2_works() {
    assert_that!(0, eq log2_of_power_of_2(0));
    for i in 0..64 {
        assert_that!(i as u8, eq log2_of_power_of_2(2u64.pow(i)));
    }
}

#[test]
fn math_round_to_pow2_works() {
    assert_that!(round_to_pow2(1), eq 1);
    assert_that!(round_to_pow2(2), eq 2);
    assert_that!(round_to_pow2(3), eq 4);
    assert_that!(round_to_pow2(4), eq 4);
    assert_that!(round_to_pow2(5), eq 8);
    assert_that!(round_to_pow2(6), eq 8);
    assert_that!(round_to_pow2(8589934597), eq 17179869184);
}

#[test]
fn math_dec_to_64() {
    assert_that!(0u64.to_b64(), eq "0");
    assert_that!(9u64.to_b64(), eq "9");
    assert_that!(10u64.to_b64(), eq "A");
    assert_that!(35u64.to_b64(), eq "Z");
    assert_that!(36u64.to_b64(), eq "a");
    assert_that!(61u64.to_b64(), eq "z");
    assert_that!(62u64.to_b64(), eq "-");
    assert_that!(63u64.to_b64(), eq "_");

    assert_that!(64u64.to_b64(), eq "01");
    assert_that!(65u64.to_b64(), eq "11");
    assert_that!(127u64.to_b64(), eq "_1");
    assert_that!(128u64.to_b64(), eq "02");
    assert_that!(129u64.to_b64(), eq "12");

    assert_that!(4095u64.to_b64(), eq "__");
    assert_that!(4096u64.to_b64(), eq "001");

    assert_that!(262142u64.to_b64(), eq "-__");
    assert_that!(262143u64.to_b64(), eq "___");
    assert_that!(262144u64.to_b64(), eq "0001");
}
