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

use iceoryx2_bb_elementary::cyclic_tagger::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn create_tag_works() {
    let sut = CyclicTagger::new();
    let sut_tag = sut.create_tag();

    assert_that!(sut_tag.was_tagged_by(&sut), eq true);
}

#[test]
fn create_untagged_tag_works() {
    let sut = CyclicTagger::new();
    let sut_tag = sut.create_untagged_tag();

    assert_that!(sut_tag.was_tagged_by(&sut), eq false);
}

#[test]
fn tagging_after_new_cyclic_works() {
    let sut = CyclicTagger::new();
    let sut_tag_1 = sut.create_tag();
    let sut_tag_2 = sut.create_tag();

    sut.next_cycle();

    sut.tag(&sut_tag_1);

    assert_that!(sut_tag_1.was_tagged_by(&sut), eq true);
    assert_that!(sut_tag_2.was_tagged_by(&sut), eq false);
}
