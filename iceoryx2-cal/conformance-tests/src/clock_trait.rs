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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod clock_trait {
    use core::time::Duration;
    use iceoryx2_bb_posix::testing::generate_file_path;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::clock::{Clock, ClockBuilder, ScheduleHandle, Timeout};
    use iceoryx2_cal::named_concept::NamedConceptBuilder;
    use iceoryx2_cal::testing::*;

    const TIMEOUT: Duration = Duration::from_millis(10);

    #[conformance_test]
    pub fn blocking_sleep_sleeps_for_at_least_provided_duration<Sut: Clock>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::TimeController>();

        let sut = Sut::ClockBuilder::new(&name)
            .config(&config)
            .acquire()
            .unwrap();

        let start = sut.now().unwrap();
        assert_that!(sut.blocking_sleep(Timeout::By(TIMEOUT)), is_ok);
        let end = sut.now().unwrap();
        let sleep_duration = end - start;

        assert_that!(sleep_duration, time_at_least TIMEOUT);
    }

    #[conformance_test]
    pub fn blocking_sleep_sleeps_until_at_least_provided_time_point<Sut: Clock>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::TimeController>();

        let sut = Sut::ClockBuilder::new(&name)
            .config(&config)
            .acquire()
            .unwrap();

        let start = sut.now().unwrap();
        assert_that!(sut.blocking_sleep(Timeout::At(start + TIMEOUT)), is_ok);
        let end = sut.now().unwrap();
        let sleep_duration = end - start;

        assert_that!(sleep_duration, time_at_least TIMEOUT);
    }

    #[conformance_test]
    pub fn without_timecontroller_support_schedule_always_returns_translated_timeout<Sut: Clock>() {
        test_requires!(!Sut::HAS_TIME_CONTROLLER_SUPPORT);

        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::TimeController>();

        let sut = Sut::ClockBuilder::new(&name)
            .config(&config)
            .acquire()
            .unwrap();

        if let (_, ScheduleHandle::Trigger(_)) = sut.schedule(Timeout::By(TIMEOUT)) {
            assert_that!(shall_never_be_reached);
        }
    }

    #[conformance_test]
    pub fn with_timecontroller_support_schedule_always_returns_trigger<Sut: Clock>() {
        test_requires!(Sut::HAS_TIME_CONTROLLER_SUPPORT);

        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::TimeController>();

        let sut = Sut::ClockBuilder::new(&name)
            .config(&config)
            .acquire()
            .unwrap();

        if let (_, ScheduleHandle::TranslatedTimeout(_)) = sut.schedule(Timeout::By(TIMEOUT)) {
            assert_that!(shall_never_be_reached);
        }
    }
}
