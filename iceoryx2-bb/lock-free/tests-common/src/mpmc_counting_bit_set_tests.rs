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

use iceoryx2_bb_lock_free::mpmc::counting_bit_set::{CountingBitSet, FixedSizeCountingBitSet};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

const SUT_CAPACITY: usize = 32;

type FixedSizeSut = FixedSizeCountingBitSet<SUT_CAPACITY>;

#[test]
pub fn set_every_bit_individually_works() {
    let sut = FixedSizeSut::new();

    for i in 0..SUT_CAPACITY {
        sut.set(i);
        let mut callback_counter = 0;
        sut.reset_all(|state| {
            callback_counter += 1;
            assert_that!(state.bit(), eq i);
            assert_that!(state.count(), eq 1);
        });
        assert_that!(callback_counter, eq 1);
    }
}

#[test]
pub fn set_every_bit_multiple_times_works() {
    const NUMBER_OF_SETS: usize = 5;
    let sut = FixedSizeSut::new();

    for i in 0..SUT_CAPACITY {
        for _ in 0..NUMBER_OF_SETS {
            sut.set(i);
        }
        let mut callback_counter = 0;
        sut.reset_all(|state| {
            callback_counter += 1;
            assert_that!(state.bit(), eq i);
            assert_that!(state.count(), eq NUMBER_OF_SETS as u64);
        });
        assert_that!(callback_counter, eq 1);
    }
}

#[test]
pub fn set_all_bits_at_once() {
    let sut = FixedSizeSut::new();

    for n in 0..SUT_CAPACITY {
        for _ in 0..n {
            sut.set(n);
        }
    }

    let mut callback_counter = 0;
    sut.reset_all(|state| {
        callback_counter += 1;
        assert_that!(state.bit(), eq state.count() as usize);
    });
    assert_that!(callback_counter, eq SUT_CAPACITY - 1);
}

#[test]
pub fn reset_sets_all_counters_to_zero() {
    let sut = FixedSizeSut::new();

    for n in 0..SUT_CAPACITY {
        sut.set(n);
    }

    let mut callback_counter = 0;
    sut.reset_all(|_| {
        callback_counter += 1;
    });
    assert_that!(callback_counter, eq SUT_CAPACITY);

    callback_counter = 0;
    sut.reset_all(|_| {
        callback_counter += 1;
    });
    assert_that!(callback_counter, eq 0);
}

#[test]
pub fn heap_based_counting_bitset_works() {
    let sut = CountingBitSet::new(SUT_CAPACITY);

    for i in 0..SUT_CAPACITY {
        sut.set(i);
        let mut callback_counter = 0;
        sut.reset_all(|state| {
            callback_counter += 1;
            assert_that!(state.bit(), eq i);
            assert_that!(state.count(), eq 1);
        });
        assert_that!(callback_counter, eq 1);
    }
}
