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
use core::time::Duration;
use iceoryx2_bb_si_units::{TimePoint, TimeSpan};
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn new_creates_zero_timepoint() {
    let sut = TimePoint::new();
    assert_that!(sut.as_secs(), eq 0);
    assert_that!(sut.as_nanos(), eq 0);
}

#[test]
pub fn add_works() {
    let lhs = TimePoint::from_millis(600);
    let rhs = TimeSpan::from_millis(700);
    let sut = lhs + rhs;

    assert_that!(sut.as_secs(), eq 1);
    assert_that!(sut.as_millis(), eq 1_300);
    assert_that!(sut.as_micros(), eq 1_300_000);
    assert_that!(sut.as_nanos(), eq 1_300_000_000);
    assert_that!(sut.subsec_nanos(), eq 300_000_000);
    assert_that!(sut.as_secs_f32() - 1.3, le 0.01);
    assert_that!(sut.as_secs_f64() - 1.3, le 0.01);
}

#[test]
pub fn add_with_duration_works() {
    let lhs = TimePoint::from_millis(1600);
    let rhs = Duration::from_millis(1700);
    let sut = lhs + rhs;

    assert_that!(sut.as_secs(), eq 3);
    assert_that!(sut.as_millis(), eq 3_300);
    assert_that!(sut.as_micros(), eq 3_300_000);
    assert_that!(sut.as_nanos(), eq 3_300_000_000);
    assert_that!(sut.subsec_nanos(), eq 300_000_000);
    assert_that!(sut.as_secs_f32() - 3.3, le 0.01);
    assert_that!(sut.as_secs_f64() - 3.3, le 0.01);
}

#[test]
pub fn add_assign_works() {
    let mut sut = TimePoint::from_millis(700);
    sut += TimeSpan::from_millis(800);

    assert_that!(sut.as_secs(), eq 1);
    assert_that!(sut.as_millis(), eq 1_500);
    assert_that!(sut.as_micros(), eq 1_500_000);
    assert_that!(sut.as_nanos(), eq 1_500_000_000);
    assert_that!(sut.subsec_nanos(), eq 500_000_000);
    assert_that!(sut.as_secs_f32() - 1.5, le 0.01);
    assert_that!(sut.as_secs_f64() - 1.5, le 0.01);
}

#[test]
pub fn add_assign_with_duration_works() {
    let mut sut = TimePoint::from_millis(2700);
    sut += Duration::from_millis(1800);

    assert_that!(sut.as_secs(), eq 4);
    assert_that!(sut.as_millis(), eq 4_500);
    assert_that!(sut.as_micros(), eq 4_500_000);
    assert_that!(sut.as_nanos(), eq 4_500_000_000);
    assert_that!(sut.subsec_nanos(), eq 500_000_000);
    assert_that!(sut.as_secs_f32() - 4.5, le 0.01);
    assert_that!(sut.as_secs_f64() - 4.5, le 0.01);
}

#[test]
pub fn sub_timepoint_works() {
    let sut = TimePoint::from_millis(2100);
    let sut = sut - TimePoint::from_millis(900);

    assert_that!(sut.as_secs(), eq 1);
    assert_that!(sut.as_millis(), eq 1_200);
    assert_that!(sut.as_micros(), eq 1_200_000);
    assert_that!(sut.as_nanos(), eq 1_200_000_000);
    assert_that!(sut.subsec_nanos(), eq 200_000_000);
    assert_that!(sut.as_secs_f32() - 1.2, le 0.01);
    assert_that!(sut.as_secs_f64() - 1.2, le 0.01);
}

#[test]
pub fn sub_timespan_works() {
    let sut = TimePoint::from_millis(2100);
    let sut = sut - TimeSpan::from_millis(800);

    assert_that!(sut.as_secs(), eq 1);
    assert_that!(sut.as_millis(), eq 1_300);
    assert_that!(sut.as_micros(), eq 1_300_000);
    assert_that!(sut.as_nanos(), eq 1_300_000_000);
    assert_that!(sut.subsec_nanos(), eq 300_000_000);
    assert_that!(sut.as_secs_f32() - 1.3, le 0.01);
    assert_that!(sut.as_secs_f64() - 1.3, le 0.01);
}

#[test]
pub fn sub_duration_works() {
    let sut = TimePoint::from_millis(4100);
    let sut = sut - Duration::from_millis(3800);

    assert_that!(sut.as_secs(), eq 0);
    assert_that!(sut.as_millis(), eq 300);
    assert_that!(sut.as_micros(), eq 300_000);
    assert_that!(sut.as_nanos(), eq 300_000_000);
    assert_that!(sut.subsec_nanos(), eq 300_000_000);
    assert_that!(sut.as_secs_f32() - 0.3, le 0.01);
    assert_that!(sut.as_secs_f64() - 0.3, le 0.01);
}

#[test]
pub fn sub_assign_works() {
    let mut sut = TimePoint::from_millis(2500);
    sut -= TimeSpan::from_millis(1600);

    assert_that!(sut.as_secs(), eq 0);
    assert_that!(sut.as_millis(), eq 900);
    assert_that!(sut.as_micros(), eq 900_000);
    assert_that!(sut.as_nanos(), eq 900_000_000);
    assert_that!(sut.subsec_nanos(), eq 900_000_000);
    assert_that!(sut.as_secs_f32() - 0.9, le 0.01);
    assert_that!(sut.as_secs_f64() - 0.9, le 0.01);
}

#[test]
pub fn sub_assign_duration_works() {
    let mut sut = TimePoint::from_millis(5500);
    sut -= Duration::from_millis(1700);

    assert_that!(sut.as_secs(), eq 3);
    assert_that!(sut.as_millis(), eq 3_800);
    assert_that!(sut.as_micros(), eq 3_800_000);
    assert_that!(sut.as_nanos(), eq 3_800_000_000);
    assert_that!(sut.subsec_nanos(), eq 800_000_000);
    assert_that!(sut.as_secs_f32() - 3.8, le 0.01);
    assert_that!(sut.as_secs_f64() - 3.8, le 0.01);
}

#[test]
pub fn from_secs_works() {
    let sut = TimePoint::from_secs(12);

    assert_that!(sut.as_secs(), eq 12);
    assert_that!(sut.as_millis(), eq 12_000);
    assert_that!(sut.as_micros(), eq 12_000_000);
    assert_that!(sut.as_nanos(), eq 12_000_000_000);
    assert_that!(sut.subsec_nanos(), eq 000_000_000);
    assert_that!(sut.as_secs_f32() - 12.0, le 0.01);
    assert_that!(sut.as_secs_f64() - 12.0, le 0.01);
}

#[test]
pub fn from_secs_f32_works() {
    let sut = TimePoint::from_secs_f32(12.34);

    assert_that!(sut.as_secs(), eq 12);
    assert_that!(sut.as_millis(), eq 12_340);
    assert_that!(sut.as_micros(), eq 12_340_000);
    assert_that!(sut.as_secs_f32() - 12.34, le 0.01);
    assert_that!(sut.as_secs_f64() - 12.34, le 0.01);
}

#[test]
pub fn from_secs_f64_works() {
    let sut = TimePoint::from_secs_f64(122.345);

    assert_that!(sut.as_secs(), eq 122);
    assert_that!(sut.as_millis(), eq 122_345);
    assert_that!(sut.as_secs_f32() - 123.345, le 0.01);
    assert_that!(sut.as_secs_f64() - 123.345, le 0.01);
}

#[test]
pub fn from_millis_works() {
    let sut = TimePoint::from_millis(54321);

    assert_that!(sut.as_secs(), eq 54);
    assert_that!(sut.as_millis(), eq 54_321);
    assert_that!(sut.as_micros(), eq 54_321_000);
    assert_that!(sut.as_nanos(), eq 54_321_000_000);
    assert_that!(sut.subsec_nanos(), eq 321_000_000);
    assert_that!(sut.as_secs_f32() - 54.321, le 0.01);
    assert_that!(sut.as_secs_f64() - 54.321, le 0.01);
}

#[test]
pub fn from_micros_works() {
    let sut = TimePoint::from_micros(56321987);

    assert_that!(sut.as_secs(), eq 56);
    assert_that!(sut.as_millis(), eq 56_321);
    assert_that!(sut.as_micros(), eq 56_321_987);
    assert_that!(sut.as_nanos(), eq 56_321_987_000);
    assert_that!(sut.subsec_nanos(), eq 321_987_000);
    assert_that!(sut.as_secs_f32() - 56.321, le 0.01);
    assert_that!(sut.as_secs_f64() - 56.321, le 0.01);
}

#[test]
pub fn from_nanos_works() {
    let sut = TimePoint::from_nanos(57321987444);

    assert_that!(sut.as_secs(), eq 57);
    assert_that!(sut.as_millis(), eq 57_321);
    assert_that!(sut.as_micros(), eq 57_321_987);
    assert_that!(sut.as_nanos(), eq 57_321_987_444);
    assert_that!(sut.subsec_nanos(), eq 321_987_444);
    assert_that!(sut.as_secs_f32() - 57.321, le 0.01);
    assert_that!(sut.as_secs_f64() - 57.321, le 0.01);
}

#[test]
pub fn time_span_between_works() {
    let sut = TimePoint::from_secs(456);
    let span = sut.abs_diff(TimePoint::from_secs(123));

    assert_that!(span.as_secs(), eq 333);

    let span = sut.abs_diff(TimePoint::from_secs(789));
    assert_that!(span.as_secs(), eq 333);
}

#[test]
pub fn display_works() {
    assert_that!(format!("{}", TimePoint::from_secs(12)), eq "12 s");
    assert_that!(format!("{}", TimePoint::from_millis(1234)), eq "1.234 s");
    assert_that!(format!("{}", TimePoint::from_millis(34)), eq "34 ms");
    assert_that!(format!("{}", TimePoint::from_micros(3456)), eq "3.456 ms");
    assert_that!(format!("{}", TimePoint::from_micros(45)), eq "45 us");
    assert_that!(format!("{}", TimePoint::from_nanos(4567)), eq "4.567 us");
    assert_that!(format!("{}", TimePoint::from_nanos(567)), eq "567 ns");
}

#[test]
pub fn create_from_nanos_with_overflow_saturates_at_max() {
    let sut = TimePoint::from_nanos(u128::MAX);
    assert_that!(sut, eq TimePoint::MAX);
}

#[test]
pub fn create_from_micros_with_overflow_saturates_at_max() {
    let sut = TimePoint::from_micros(u128::MAX);
    assert_that!(sut, eq TimePoint::MAX);
}

#[test]
pub fn create_from_millis_with_overflow_saturates_at_max() {
    let sut = TimePoint::from_millis(u128::MAX);
    assert_that!(sut, eq TimePoint::MAX);
}

#[test]
pub fn add_that_would_lead_to_overflow_saturates_at_max() {
    let sut = TimePoint::from_secs(123) + TimeSpan::MAX;
    assert_that!(sut, eq TimePoint::MAX);
}

#[test]
pub fn sub_that_would_lead_to_underflow_saturates_at_zero() {
    let sut = TimePoint::from_secs(123) - TimeSpan::MAX;
    assert_that!(sut, eq TimePoint::ZERO);
}
