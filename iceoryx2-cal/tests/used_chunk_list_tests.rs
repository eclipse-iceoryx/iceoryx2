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

use iceoryx2_bb_testing::assert_that;
use iceoryx2_cal::zero_copy_connection::{used_chunk_list::FixedSizeUsedChunkList, PointerOffset};

#[test]
fn used_chunk_list_insert_pop_works() {
    let mut sut = FixedSizeUsedChunkList::<128>::new();

    assert_that!(sut.size(), eq 0);
    for i in 0..sut.capacity() {
        assert_that!(sut.insert(PointerOffset::new(i + 5)), eq true);
        assert_that!(sut.size(), eq i + 1);
    }
    assert_that!(sut.insert(PointerOffset::new(123)), eq false);
    assert_that!(sut.size(), eq sut.capacity());

    for i in 0..sut.capacity() {
        assert_that!(sut.pop(), eq Some(PointerOffset::new(sut.capacity() - i - 1 + 5)));
        assert_that!(sut.size(), eq sut.capacity() - i - 1);
    }
    assert_that!(sut.pop(), eq None);
    assert_that!(sut.size(), eq 0);
}
