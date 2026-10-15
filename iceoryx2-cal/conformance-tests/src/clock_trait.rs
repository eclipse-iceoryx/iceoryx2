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
    use alloc::vec;
    use core::time::Duration;
    use iceoryx2_bb_posix::testing::generate_file_path;
    use iceoryx2_bb_testing::{assert_that, test_requires};
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_cal::clock::{Clock, ClockBuilder, ScheduleHandle, Timeout};
    use iceoryx2_cal::event::Event;
    use iceoryx2_cal::static_storage::process_local::NamedConceptConfiguration;
    use iceoryx2_cal::testing::*;

    const TIMEOUT: Duration = Duration::from_millis(10);
    const SHORT_TIMEOUT: Duration = Duration::from_millis(1);

    #[conformance_test]
    pub fn blocking_sleep_sleeps_for_at_least_provided_duration<E: Event, Sut: Clock<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::Configuration>();

        let sut = Sut::Builder::new(&name)
            .config(&config)
            .open_clock()
            .unwrap();

        let start = sut.now().unwrap();
        assert_that!(sut.blocking_sleep(Timeout::By(TIMEOUT)), is_ok);
        let end = sut.now().unwrap();
        let sleep_duration = end - start;

        assert_that!(sleep_duration, time_at_least TIMEOUT);
    }

    #[conformance_test]
    pub fn blocking_sleep_sleeps_until_at_least_provided_time_point<E: Event, Sut: Clock<E>>() {
        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::Configuration>();

        let sut = Sut::Builder::new(&name)
            .config(&config)
            .open_clock()
            .unwrap();

        let start = sut.now().unwrap();
        assert_that!(sut.blocking_sleep(Timeout::At(start + TIMEOUT)), is_ok);
        let end = sut.now().unwrap();
        let sleep_duration = end - start;

        assert_that!(sleep_duration, time_at_least TIMEOUT);
    }

    #[conformance_test]
    pub fn clock_is_monotonic<E: Event, Sut: Clock<E>>() {
        const ITERATIONS: usize = 20;

        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::Configuration>();

        let sut = Sut::Builder::new(&name)
            .config(&config)
            .open_clock()
            .unwrap();

        let mut time_points = vec![];
        for _ in 0..ITERATIONS {
            time_points.push(sut.now().unwrap());
            time_points.push(sut.now().unwrap());
            time_points.push(sut.now().unwrap());

            sut.blocking_sleep(Timeout::By(SHORT_TIMEOUT)).unwrap();
            time_points.push(sut.now().unwrap());
        }

        for time_points in time_points.as_chunks::<2>().0 {
            assert_that!(time_points[0], le time_points[1]);
        }
    }

    #[conformance_test]
    pub fn default_suffix_is_used_for_default_config<E: Event, Sut: Clock<E>>() {
        let config = <Sut::Configuration as Default>::default();

        assert_that!(*config.get_suffix(), eq Sut::default_suffix());
    }

    #[conformance_test]
    pub fn without_timecontroller_support_schedule_always_returns_translated_timeout<
        E: Event,
        Sut: Clock<E>,
    >() {
        test_requires!(!Sut::HAS_TIME_CONTROLLER_SUPPORT);

        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::Configuration>();

        let sut = Sut::Builder::new(&name)
            .config(&config)
            .open_clock()
            .unwrap();

        if let ScheduleHandle::GuardAndListener(_) = sut.schedule(Timeout::By(TIMEOUT)).unwrap() {
            assert_that!(shall_never_be_reached);
        }
    }

    #[conformance_test]
    pub fn with_timecontroller_support_schedule_always_returns_trigger<E: Event, Sut: Clock<E>>() {
        test_requires!(Sut::HAS_TIME_CONTROLLER_SUPPORT);

        let name = generate_file_path().file_name();
        let config = generate_isolated_config::<Sut::Configuration>();

        let sut = Sut::Builder::new(&name)
            .config(&config)
            .open_clock()
            .unwrap();

        if let ScheduleHandle::TranslatedTimeout(_) = sut.schedule(Timeout::By(TIMEOUT)).unwrap() {
            assert_that!(shall_never_be_reached);
        }
    }
}
