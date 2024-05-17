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

use iceoryx2_bb_posix::creation_mode::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_pal_posix::*;

#[test]
fn creation_mode_o_flag_conversion_works() {
    assert_that!(
        CreationMode::CreateExclusive.as_oflag(), eq
        posix::O_CREAT | posix::O_EXCL
    );
    assert_that!(
        CreationMode::PurgeAndCreate.as_oflag(), eq
        posix::O_CREAT | posix::O_EXCL
    );
    assert_that!(CreationMode::OpenOrCreate.as_oflag(), eq posix::O_CREAT);
}

#[test]
fn creation_mode_display_works() {
    assert_that!(
        format!("{}", CreationMode::PurgeAndCreate),
        eq "CreationMode::PurgeAndCreate"
    );
    assert_that!(
        format!("{}", CreationMode::CreateExclusive),
        eq "CreationMode::CreateExclusive"
    );
    assert_that!(
        format!("{}", CreationMode::OpenOrCreate),
        eq "CreationMode::OpenOrCreate"
    );
}
