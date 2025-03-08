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

use iceoryx2_bb_elementary::visitor::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn create_visited_marker_works() {
    let sut = Visitor::new();
    let sut_marker = sut.create_visited_marker();

    assert_that!(sut_marker.was_visited_by(&sut), eq true);
}

#[test]
fn create_unvisited_marker_works() {
    let sut = Visitor::new();
    let sut_marker = sut.create_unvisited_marker();

    assert_that!(sut_marker.was_visited_by(&sut), eq false);
}

#[test]
fn visiting_after_new_cycle_works() {
    let sut = Visitor::new();
    let sut_marker_1 = sut.create_visited_marker();
    let sut_marker_2 = sut.create_visited_marker();

    sut.next_cycle();

    sut.visit(&sut_marker_1);

    assert_that!(sut_marker_1.was_visited_by(&sut), eq true);
    assert_that!(sut_marker_2.was_visited_by(&sut), eq false);
}
