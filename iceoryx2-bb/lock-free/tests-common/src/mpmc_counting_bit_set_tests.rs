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

use iceoryx2_bb_lock_free::mpmc::counting_bit_set::FixedSizeCountingBitSet;
use iceoryx2_bb_testing::assert_that;

const SUT_CAPACITY: usize = 1024;

type FixedSizeSut = FixedSizeCountingBitSet<SUT_CAPACITY>;

#[test]
pub fn create_fill_and_reset_works() {
    let sut = FixedSizeSut::new();

    for i in 0..SUT_CAPACITY {
        sut.set(i);
        sut.reset_all(|state| {
            assert_that!(state.bit(), eq i);
            assert_that!(state.count(), eq 1);
        });
    }
}
