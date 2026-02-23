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

use iceoryx2_bb_elementary::alignment::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn alignment_of_invalid_values_returns_none() {
    assert_that!(Alignment::new(0), is_none);
    assert_that!(Alignment::new(3), is_none);
    assert_that!(Alignment::new(7), is_none);
    assert_that!(Alignment::new(331), is_none);
}

#[test]
fn alignment_of_valid_values_works() {
    for n in 0..16 {
        let raw_alignment = 2usize.pow(n as u32);
        let sut = Alignment::new(raw_alignment);
        assert_that!(sut, is_some);
        assert_that!(sut.unwrap().value(), eq raw_alignment);
    }
}
