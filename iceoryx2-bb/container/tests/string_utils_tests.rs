// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_container::string::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn strnlen_returns_len_for_non_empty_string() {
    let max_len = 100;
    let some_string = "whatever you want\0";
    assert_that!(unsafe { strnlen(some_string.as_ptr().cast(), max_len) }, eq 17);
}

#[test]
fn strnlen_returns_len_for_empty_string() {
    let max_len = 100;
    let some_string = "\0";
    assert_that!(unsafe { strnlen(some_string.as_ptr().cast(), max_len) }, eq 0);
}

#[test]
fn strnlen_returns_max_len_when_string_is_longer_than_max_len() {
    let max_len = 2;
    let some_string = "nothing to sniffle here!\0";
    assert_that!(unsafe { strnlen(some_string.as_ptr().cast(), max_len) }, eq max_len);
}

#[test]
fn as_escaped_string_escapes_all_escapable_characters() {
    assert_that!(as_escaped_string(b"\x09"), eq "\\t");
    assert_that!(as_escaped_string(b"\x0d"), eq "\\r");
    assert_that!(as_escaped_string(b"\x0A"), eq "\\n");
    assert_that!(as_escaped_string(b"\x20"), eq " ");
    assert_that!(as_escaped_string(b"\x7e"), eq "~");
    assert_that!(as_escaped_string(b"\x01"), eq "\\x01");
}

#[test]
fn as_escaped_string_does_not_escape_printable_characters() {
    for c in 32u8..128u8 {
        let value = format!("{}", c as char);
        assert_that!(as_escaped_string(value.as_bytes()), eq value);
    }
}
