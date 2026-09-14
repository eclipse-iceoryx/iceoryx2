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

use iceoryx2_bb_elementary::generation::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn a_tracked_generation_is_unchanged_since_itself_only() {
    assert_that!(Generation::At(3).unchanged_since(Generation::At(3)), eq true);
    assert_that!(Generation::At(3).unchanged_since(Generation::At(2)), eq false);
    assert_that!(Generation::At(3).unchanged_since(Generation::Untracked), eq false);
}

#[test]
pub fn an_untracked_generation_is_never_unchanged() {
    assert_that!(Generation::Untracked.unchanged_since(Generation::Untracked), eq false);
    assert_that!(Generation::Untracked.unchanged_since(Generation::At(3)), eq false);
}

#[test]
pub fn a_counter_advances_its_generation() {
    let sut = GenerationCounter::new();
    let before = sut.current();

    sut.advance();

    assert_that!(sut.current().unchanged_since(before), eq false);
    assert_that!(sut.current().unchanged_since(sut.current()), eq true);
}
