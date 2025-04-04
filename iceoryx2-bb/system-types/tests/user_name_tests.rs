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

use iceoryx2_bb_container::semantic_string::*;
use iceoryx2_bb_system_types::user_name::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn user_name_new_with_illegal_name_fails() {
    let sut = UserName::new(b"");
    assert_that!(sut, is_err);

    let sut = UserName::new(b"-asdf");
    assert_that!(sut, is_err);

    let sut = UserName::new(b"0asdf");
    assert_that!(sut, is_err);

    let sut = UserName::new(b"asd\0f");
    assert_that!(sut, is_err);
}

#[test]
fn user_name_new_with_legal_name_works() {
    let sut = UserName::new(b"abcdefghijklmnopqrstuvwxyz-0123");
    assert_that!(sut, is_ok);

    let sut = UserName::new(b"a456789-");
    assert_that!(sut, is_ok);

    let sut = UserName::new(b"Abc-Def");
    assert_that!(sut, is_ok);

    let sut = UserName::new(b"_fuu_bar_");
    assert_that!(sut, is_ok);
}
