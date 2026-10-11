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

use core::time::Duration;
use iceoryx2_bb_si_units::time_span::TimeSpan;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn relocatable_duration_roundtrip_conversion() {
    let secs = 123;
    let nsecs = 456;
    let duration = Duration::from_secs(secs) + Duration::from_nanos(nsecs);
    let sut: TimeSpan = duration.into();
    let duration_2: Duration = sut.into();

    assert_that!(duration, eq duration_2);
    assert_that!(sut.as_secs(), eq duration.as_secs());
    assert_that!(sut.subsec_nanos(), eq duration.subsec_nanos());
}

#[test]
pub fn relocatable_duration_max_value() {
    let duration = Duration::MAX;
    let sut: TimeSpan = duration.into();

    assert_that!(sut.as_secs(), eq sut.as_secs());
    assert_that!(sut.subsec_nanos(), eq sut.subsec_nanos());
}
