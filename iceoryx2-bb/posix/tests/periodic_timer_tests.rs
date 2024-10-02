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

mod periodic_timer {
    use iceoryx2_bb_posix::periodic_timer::*;
    use iceoryx2_bb_testing::assert_that;
    use std::time::Duration;

    #[test]
    fn next_iteration_works_smallest_timeout_added_first() {
        let mut sut = PeriodicTimerBuilder::new().create().unwrap();

        sut.add(Duration::from_secs(5));
        sut.add(Duration::from_secs(10));
        sut.add(Duration::from_secs(100));

        sut.start().unwrap();

        assert_that!(sut.next_iteration().unwrap(), le Duration::from_secs(5));
        assert_that!(sut.next_iteration().unwrap(), ge Duration::from_secs(1));
    }

    #[test]
    fn next_iteration_works_smallest_timeout_added_last() {
        let mut sut = PeriodicTimerBuilder::new().create().unwrap();

        sut.add(Duration::from_secs(100));
        sut.add(Duration::from_secs(10));
        sut.add(Duration::from_secs(5));

        sut.start().unwrap();

        assert_that!(sut.next_iteration().unwrap(), le Duration::from_secs(5));
        assert_that!(sut.next_iteration().unwrap(), ge Duration::from_secs(1));
    }

    #[test]
    fn removing_timeout_works() {
        let mut sut = PeriodicTimerBuilder::new().create().unwrap();

        sut.add(Duration::from_secs(1000));
        sut.add(Duration::from_secs(100));
        let idx = sut.add(Duration::from_secs(1));

        sut.start().unwrap();

        sut.remove(idx);

        assert_that!(sut.next_iteration().unwrap(), ge Duration::from_secs(10));
        assert_that!(sut.next_iteration().unwrap(), le Duration::from_secs(100));
    }

    #[test]
    fn no_missed_timers_works() {
        let mut sut = PeriodicTimerBuilder::new().create().unwrap();

        sut.add(Duration::from_secs(10));
        sut.add(Duration::from_secs(100));
        sut.add(Duration::from_secs(1000));

        sut.start().unwrap();

        let mut missed_timers = vec![];
        sut.missed_timers(|idx| missed_timers.push(idx)).unwrap();

        assert_that!(missed_timers, len 0);
    }

    #[test]
    fn one_missed_timers_works() {
        let mut sut = PeriodicTimerBuilder::new().create().unwrap();

        let idx = sut.add(Duration::from_nanos(1));
        sut.add(Duration::from_secs(100));
        sut.add(Duration::from_secs(1000));

        sut.start().unwrap();
        std::thread::sleep(Duration::from_millis(10));

        let mut missed_timers = vec![];
        sut.missed_timers(|idx| missed_timers.push(idx)).unwrap();

        assert_that!(missed_timers, len 1);
        assert_that!(missed_timers, contains idx);
    }

    #[test]
    fn many_missed_timers_works() {
        let mut sut = PeriodicTimerBuilder::new().create().unwrap();

        let idx_1 = sut.add(Duration::from_nanos(1));
        let idx_2 = sut.add(Duration::from_nanos(10));
        let idx_3 = sut.add(Duration::from_nanos(20));

        sut.start().unwrap();
        std::thread::sleep(Duration::from_millis(10));

        let mut missed_timers = vec![];
        sut.missed_timers(|idx| missed_timers.push(idx)).unwrap();

        assert_that!(missed_timers, len 3);
        assert_that!(missed_timers, contains idx_1);
        assert_that!(missed_timers, contains idx_2);
        assert_that!(missed_timers, contains idx_3);
    }
}
