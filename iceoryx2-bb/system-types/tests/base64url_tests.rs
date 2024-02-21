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

use iceoryx2_bb_system_types::base64url::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn base64url_new_with_legal_content_works() {
    let sut = Base64Url::new(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_");
    assert_that!(sut, is_ok);

    let sut = Base64Url::new(b"a");
    assert_that!(sut, is_ok);

    let sut = Base64Url::new(b"_");
    assert_that!(sut, is_ok);

    let sut = Base64Url::new(b"123abcZXG__");
    assert_that!(sut, is_ok);
}

#[test]
fn base64url_new_with_illegal_content_fails() {
    let sut = Base64Url::new(b"");
    assert_that!(sut, is_err);

    let sut = Base64Url::new(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.");
    assert_that!(sut, is_err);

    let sut = Base64Url::new(b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLM,?/NOPQRWXYZ0123456789-_");
    assert_that!(sut, is_err);

    let sut = Base64Url::new(b"/");
    assert_that!(sut, is_err);

    let sut = Base64Url::new(b".");
    assert_that!(sut, is_err);

    let sut = Base64Url::new(b"abc=");
    assert_that!(sut, is_err);
}

#[test]
fn base64url_as_file_name_works() {
    let sut =
        Base64Url::new(b"abcdefghijklmnopqrstuvwDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_").unwrap();
    let sut_file = sut.as_file_name();
    assert_that!(sut.as_bytes(), eq sut_file.as_bytes());

    let sut = Base64Url::new(b"a").unwrap();
    let sut_file = sut.as_file_name();
    assert_that!(sut.as_bytes(), eq sut_file.as_bytes());

    let sut = Base64Url::new(b"_").unwrap();
    let sut_file = sut.as_file_name();
    assert_that!(sut.as_bytes(), eq sut_file.as_bytes());

    let sut = Base64Url::new(b"123abcZXG__").unwrap();
    let sut_file = sut.as_file_name();
    assert_that!(sut.as_bytes(), eq sut_file.as_bytes());
}
