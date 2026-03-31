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

use alloc::format;

use iceoryx2_bb_posix::access_mode::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_pal_posix::*;

#[test]
pub fn prot_flag_conversion_works() {
    assert_that!(AccessMode::None.as_protflag(), eq posix::PROT_NONE);
    assert_that!(AccessMode::Read.as_protflag(), eq posix::PROT_READ);
    assert_that!(AccessMode::Write.as_protflag(), eq posix::PROT_WRITE);
    assert_that!(
        AccessMode::ReadWrite.as_protflag(), eq
        posix::PROT_READ | posix::PROT_WRITE
    );
}

#[test]
pub fn o_flag_conversion_works() {
    assert_that!(AccessMode::None.as_oflag(), eq 0);
    assert_that!(AccessMode::Read.as_oflag(), eq posix::O_RDONLY);
    assert_that!(AccessMode::Write.as_oflag(), eq posix::O_WRONLY);
    assert_that!(AccessMode::ReadWrite.as_oflag(), eq posix::O_RDWR);
}

#[test]
pub fn display_works() {
    assert_that!(format!("{}", AccessMode::None), eq "AccessMode::None");
    assert_that!(format!("{}", AccessMode::Read), eq "AccessMode::Read");
    assert_that!(format!("{}", AccessMode::Write), eq "AccessMode::Write");
    assert_that!(format!("{}", AccessMode::ReadWrite), eq "AccessMode::ReadWrite");
}
