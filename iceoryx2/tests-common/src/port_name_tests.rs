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

use alloc::format;

use iceoryx2::prelude::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
fn creating_fails() {
    let value = "ö la palöma blanca";
    let sut = PortName::new(value);

    assert_that!(sut.is_err(), eq true);
    assert_that!(sut.err(), eq Some(SemanticStringError::InvalidContent));
}

#[test]
fn creating_empty_works() {
    let value = "";
    let sut = PortName::new_empty();

    assert_that!(sut, eq value);
    assert_that!(&sut, eq value);
}

#[test]
fn creating_works() {
    let value = "oe la paloema blanca";
    let sut = PortName::new(value).unwrap();

    assert_that!(sut, eq value);
    assert_that!(&sut, eq value);
}

#[test]
fn display_works() {
    let value = "eim just a boerd in se sky";
    let sut = PortName::new(value).unwrap();

    assert_that!(format!("{}", sut), eq value);
}

#[test]
fn try_into_works() {
    let value = "oever se maundaen I fly";
    let sut: PortName = value.try_into().unwrap();

    assert_that!(sut, eq value);
    assert_that!(&sut, eq value);
}
