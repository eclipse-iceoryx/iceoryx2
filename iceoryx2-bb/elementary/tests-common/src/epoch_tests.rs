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

use iceoryx2_bb_elementary::epoch::Epoch;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn the_next_epoch_is_later() {
    let sut = Epoch::default();

    assert_that!(sut.next() > sut, eq true);
    assert_that!(sut.next().next() > sut.next(), eq true);
}
