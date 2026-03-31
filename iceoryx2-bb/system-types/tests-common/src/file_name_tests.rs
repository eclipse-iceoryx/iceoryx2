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

#![allow(clippy::disallowed_types)]

use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_system_types::file_name::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn new_with_illegal_name_fails() {
    let sut = FileName::new(b"");
    assert_that!(sut, is_err);

    let sut = FileName::new(b".");
    assert_that!(sut, is_err);

    let sut = FileName::new(b"..");
    assert_that!(sut, is_err);
}

#[test]
pub fn new_name_with_slash_is_illegal() {
    let sut = FileName::new(b"hell/.txt");
    assert_that!(sut, is_err);
}

#[test]
pub fn pop_fails_when_it_results_in_illegal_name() {
    let sut = FileName::new(b"..f");
    assert_that!(sut, is_ok);
    let mut sut = sut.unwrap();

    assert_that!(sut.pop(), is_err);

    assert_that!(sut, len 3);
    assert_that!(sut.as_bytes(), eq b"..f");
}

#[test]
pub fn remove_fails_when_it_results_in_illegal_name() {
    let sut = FileName::new(b".f.");
    assert_that!(sut, is_ok);
    let mut sut = sut.unwrap();

    assert_that!(sut.remove(1), is_err);

    assert_that!(sut, len 3);
    assert_that!(sut.as_bytes(), eq b".f.");
}

#[test]
pub fn remove_range_fails_when_it_results_in_illegal_name() {
    let sut = FileName::new(b".fuu");
    assert_that!(sut, is_ok);
    let mut sut = sut.unwrap();

    assert_that!(sut.remove_range(1, 3), is_err);

    assert_that!(sut, len 4);
    assert_that!(sut.as_bytes(), eq b".fuu");
}

#[test]
pub fn retain_fails_when_it_results_in_illegal_name() {
    let sut = FileName::new(b".fuu");
    assert_that!(sut, is_ok);
    let mut sut = sut.unwrap();

    let retain_result = sut.retain(|c| c != b'.');
    assert_that!(retain_result, is_err);

    assert_that!(sut, len 4);
    assert_that!(sut.as_bytes(), eq b".fuu");
}
