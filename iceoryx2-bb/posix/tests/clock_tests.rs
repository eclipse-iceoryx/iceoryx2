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
use iceoryx2_bb_posix::clock::*;
use iceoryx2_bb_posix::system_configuration::Feature;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use std::time::Instant;

const TIMEOUT: Duration = Duration::from_millis(100);

#[test]
fn clock_nanosleep_sleeps_at_least_given_amount_of_time() {
    let start = Instant::now();
    assert_that!(nanosleep(TIMEOUT), is_ok);
    assert_that!(start.elapsed(), time_at_least TIMEOUT);
}

#[test]
fn clock_nanosleep_with_clock_sleeps_at_least_given_amount_of_time() {
    let start = Instant::now();
    assert_that!(nanosleep_with_clock(TIMEOUT, ClockType::Realtime), is_ok);
    assert_that!(start.elapsed(), time_at_least TIMEOUT);
}

#[test]
fn clock_timebuilder_default_values_are_set_correctly() {
    let time = TimeBuilder::new().create();
    assert_that!(time.seconds(), eq 0);
    assert_that!(time.nanoseconds(), eq 0);
    assert_that!(time.clock_type(), eq ClockType::default());
}

#[test]
fn clock_timebuilder_creates_time_correctly() {
    let time = TimeBuilder::new()
        .seconds(123)
        .nanoseconds(456)
        .clock_type(ClockType::Realtime)
        .create();
    assert_that!(time.seconds(), eq 123);
    assert_that!(time.nanoseconds(), eq 456);
    assert_that!(time.clock_type(), eq ClockType::Realtime);
}

#[test]
fn clock_time_conversion_to_duration_works() {
    let time = TimeBuilder::new().seconds(789).nanoseconds(321).create();
    let d = time.as_duration();

    assert_that!(d.as_secs(), eq time.seconds());
    assert_that!(d.subsec_nanos(), eq time.nanoseconds());
}

#[test]
fn clock_time_now_is_monotonic_with_monotonic_clock() {
    test_requires!(Feature::MonotonicClock.is_available());

    let start = Time::now_with_clock(ClockType::Monotonic).unwrap();
    assert_that!(nanosleep(TIMEOUT), is_ok);
    let start2 = Time::now_with_clock(ClockType::Monotonic).unwrap();
    assert_that!(nanosleep(TIMEOUT), is_ok);

    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT * 2);
    assert_that!(start2.elapsed().unwrap(), time_at_least TIMEOUT);
}

#[test]
fn clock_time_as_timespec_works() {
    let now = Time::now().unwrap();
    let timespec = now.as_timespec();

    assert_that!(timespec.tv_sec, eq now.as_duration().as_secs() as _);
    assert_that!(timespec.tv_nsec, eq now.as_duration().subsec_nanos() as _);
}
