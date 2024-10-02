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
        let sut = PeriodicTimerBuilder::new().create().unwrap();

        let _guard_1 = sut.cyclic(Duration::from_secs(5)).unwrap();
        let _guard_2 = sut.cyclic(Duration::from_secs(10)).unwrap();
        let _guard_2 = sut.cyclic(Duration::from_secs(100)).unwrap();

        assert_that!(sut.duration_until_next_timeout().unwrap(), le Duration::from_secs(5));
        assert_that!(sut.duration_until_next_timeout().unwrap(), ge Duration::from_secs(1));
    }

    #[test]
    fn next_iteration_works_smallest_timeout_added_last() {
        let sut = PeriodicTimerBuilder::new().create().unwrap();

        let _guard_1 = sut.cyclic(Duration::from_secs(100)).unwrap();
        let _guard_2 = sut.cyclic(Duration::from_secs(10)).unwrap();
        let _guard_3 = sut.cyclic(Duration::from_secs(5)).unwrap();

        assert_that!(sut.duration_until_next_timeout().unwrap(), le Duration::from_secs(5));
        assert_that!(sut.duration_until_next_timeout().unwrap(), ge Duration::from_secs(1));
    }

    #[test]
    fn removing_timeout_works() {
        let sut = PeriodicTimerBuilder::new().create().unwrap();

        let _guard_1 = sut.cyclic(Duration::from_secs(1000)).unwrap();
        let _guard_2 = sut.cyclic(Duration::from_secs(100)).unwrap();
        let _guard_3 = sut.cyclic(Duration::from_secs(1)).unwrap();

        drop(_guard_3);

        assert_that!(sut.duration_until_next_timeout().unwrap(), ge Duration::from_secs(10));
        assert_that!(sut.duration_until_next_timeout().unwrap(), le Duration::from_secs(100));
    }

    #[test]
    fn no_missed_timeout_works() {
        let sut = PeriodicTimerBuilder::new().create().unwrap();

        let _guard_1 = sut.cyclic(Duration::from_secs(10)).unwrap();
        let _guard_2 = sut.cyclic(Duration::from_secs(100)).unwrap();
        let _guard_3 = sut.cyclic(Duration::from_secs(1000)).unwrap();

        let mut missed_timers = vec![];
        sut.missed_timeouts(|idx| missed_timers.push(idx)).unwrap();

        assert_that!(missed_timers, len 0);
    }

    #[test]
    fn one_missed_timeouts_works() {
        let sut = PeriodicTimerBuilder::new().create().unwrap();

        let _guard_1 = sut.cyclic(Duration::from_nanos(1)).unwrap();
        let _guard_2 = sut.cyclic(Duration::from_secs(100)).unwrap();
        let _guard_3 = sut.cyclic(Duration::from_secs(1000)).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        let mut missed_timeouts = vec![];
        sut.missed_timeouts(|idx| missed_timeouts.push(idx))
            .unwrap();

        assert_that!(missed_timeouts, len 1);
        assert_that!(missed_timeouts, contains _guard_1.index());
    }

    #[test]
    fn many_missed_timeouts_works() {
        let sut = PeriodicTimerBuilder::new().create().unwrap();

        let guard_1 = sut.cyclic(Duration::from_nanos(1)).unwrap();
        let guard_2 = sut.cyclic(Duration::from_nanos(10)).unwrap();
        let guard_3 = sut.cyclic(Duration::from_nanos(20)).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        let mut missed_timeouts = vec![];
        sut.missed_timeouts(|idx| missed_timeouts.push(idx))
            .unwrap();

        assert_that!(missed_timeouts, len 3);
        assert_that!(missed_timeouts, contains guard_1.index());
        assert_that!(missed_timeouts, contains guard_2.index());
        assert_that!(missed_timeouts, contains guard_3.index());
    }
}
