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

use iceoryx2_bb_system_types::port::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn set_works() {
    assert_that!(Port::new(54321).as_u16(), eq 54321);
}

#[test]
pub fn is_unspecified_works() {
    assert_that!(UNSPECIFIED.is_unspecified(), eq true);
    assert_that!(UNSPECIFIED, eq Port::new(0));
    assert_that!(Port::new(143).is_unspecified(), eq false);
}

#[test]
pub fn is_system_works() {
    assert_that!(Port::new(1).is_system(), eq true);
    assert_that!(Port::new(1023).is_system(), eq true);
    assert_that!(UNSPECIFIED.is_system(), eq false);
    assert_that!(Port::new(1493).is_system(), eq false);
}

#[test]
pub fn is_registered_works() {
    assert_that!(Port::new(1024).is_registered(), eq true);
    assert_that!(Port::new(49151).is_registered(), eq true);
    assert_that!(UNSPECIFIED.is_registered(), eq false);
    assert_that!(Port::new(51493).is_registered(), eq false);
}

#[test]
pub fn is_dynamic_works() {
    assert_that!(Port::new(49152).is_dynamic(), eq true);
    assert_that!(Port::new(65535).is_dynamic(), eq true);
    assert_that!(UNSPECIFIED.is_dynamic(), eq false);
    assert_that!(Port::new(5193).is_dynamic(), eq false);
}

#[test]
pub fn try_from_str_work() {
    assert_that!(Port::try_from("1234"), eq Ok(Port::new(1234)));
}

#[test]
pub fn try_from_str_with_invalid_integer_fails() {
    assert_that!(Port::try_from("12huh"), is_err);
}
