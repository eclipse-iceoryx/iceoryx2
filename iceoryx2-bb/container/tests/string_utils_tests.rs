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

use iceoryx2_bb_container::string::*;
use iceoryx2_bb_testing::assert_that;

#[test]
fn strnlen_returns_max_without_null_terminator() {
    let max_len = 4;
    let some_string = "hello world";
    assert_that!(unsafe { strnlen(some_string.as_ptr().cast(), max_len) }, eq max_len);
}

#[test]
fn as_escaped_string_works() {
    assert_that!(as_escaped_string(b"\\t"), eq "\\t");
    assert_that!(as_escaped_string(b"\\r"), eq "\\r");
    assert_that!(as_escaped_string(b"\\n"), eq "\\n");
    assert_that!(as_escaped_string(b"\x20"), eq " ");
    assert_that!(as_escaped_string(b"\x7e"), eq "~");
    assert_that!(as_escaped_string(b"\x01"), eq "\\x01");
}
