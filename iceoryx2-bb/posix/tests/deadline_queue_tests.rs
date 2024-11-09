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

mod deadline_queue {
    use iceoryx2_bb_elementary::CallbackProgression;
    use iceoryx2_bb_posix::deadline_queue::*;
    use iceoryx2_bb_testing::assert_that;
    use std::time::Duration;

    #[test]
    fn attach_detach_works() {
        const NUMBER_OF_ATTACHMENTS: usize = 16;
        let sut = DeadlineQueueBuilder::new().create().unwrap();
        let mut guards = vec![];

        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.len(), eq 0);
        for n in 0..NUMBER_OF_ATTACHMENTS {
            guards.push(sut.add_deadline_interval(Duration::from_secs((n + 1) as u64)));
            assert_that!(sut.is_empty(), eq false);
            assert_that!(sut.len(), eq n + 1);
        }

        guards.clear();
        assert_that!(sut.is_empty(), eq true);
        assert_that!(sut.len(), eq 0);
    }

    #[test]
    fn next_iteration_works_smallest_deadline_added_first() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let _guard_1 = sut.add_deadline_interval(Duration::from_secs(5)).unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_secs(10)).unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_secs(100)).unwrap();

        assert_that!(sut.duration_until_next_deadline().unwrap(), le Duration::from_secs(5));
        assert_that!(sut.duration_until_next_deadline().unwrap(), ge Duration::from_secs(1));
    }

    #[test]
    fn next_iteration_works_smallest_deadline_added_last() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let _guard_1 = sut.add_deadline_interval(Duration::from_secs(100)).unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_secs(10)).unwrap();
        let _guard_3 = sut.add_deadline_interval(Duration::from_secs(5)).unwrap();

        assert_that!(sut.duration_until_next_deadline().unwrap(), le Duration::from_secs(5));
        assert_that!(sut.duration_until_next_deadline().unwrap(), ge Duration::from_secs(1));
    }

    #[test]
    fn removing_deadline_works() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let _guard_1 = sut
            .add_deadline_interval(Duration::from_secs(1000))
            .unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_secs(100)).unwrap();
        let _guard_3 = sut.add_deadline_interval(Duration::from_secs(1)).unwrap();

        drop(_guard_3);

        assert_that!(sut.duration_until_next_deadline().unwrap(), ge Duration::from_secs(10));
        assert_that!(sut.duration_until_next_deadline().unwrap(), le Duration::from_secs(100));
    }

    #[test]
    fn no_missed_deadline_works() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let _guard_1 = sut.add_deadline_interval(Duration::from_secs(10)).unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_secs(100)).unwrap();
        let _guard_3 = sut
            .add_deadline_interval(Duration::from_secs(1000))
            .unwrap();

        let mut missed_deadline_queues = vec![];
        sut.missed_deadlines(|idx| {
            missed_deadline_queues.push(idx);
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(missed_deadline_queues, len 0);
    }

    #[test]
    fn one_missed_deadlines_works() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let _guard_1 = sut.add_deadline_interval(Duration::from_nanos(1)).unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_secs(100)).unwrap();
        let _guard_3 = sut
            .add_deadline_interval(Duration::from_secs(1000))
            .unwrap();

        std::thread::sleep(Duration::from_millis(10));

        let mut missed_deadlines = vec![];
        sut.missed_deadlines(|idx| {
            missed_deadlines.push(idx);
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(missed_deadlines, len 1);
        assert_that!(missed_deadlines, contains _guard_1.index());
    }

    #[test]
    fn many_missed_deadlines_works() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let guard_1 = sut.add_deadline_interval(Duration::from_nanos(1)).unwrap();
        let guard_2 = sut.add_deadline_interval(Duration::from_nanos(10)).unwrap();
        let guard_3 = sut.add_deadline_interval(Duration::from_nanos(20)).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        let mut missed_deadlines = vec![];
        sut.missed_deadlines(|idx| {
            missed_deadlines.push(idx);
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(missed_deadlines, len 3);
        assert_that!(missed_deadlines, contains guard_1.index());
        assert_that!(missed_deadlines, contains guard_2.index());
        assert_that!(missed_deadlines, contains guard_3.index());
    }

    #[test]
    fn missed_deadline_iteration_stops_when_requested() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let _guard_1 = sut.add_deadline_interval(Duration::from_nanos(1)).unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_nanos(10)).unwrap();
        let _guard_3 = sut.add_deadline_interval(Duration::from_nanos(20)).unwrap();

        std::thread::sleep(Duration::from_millis(10));

        let mut missed_deadlines = vec![];
        sut.missed_deadlines(|idx| {
            missed_deadlines.push(idx);
            CallbackProgression::Stop
        })
        .unwrap();

        assert_that!(missed_deadlines, len 1);
    }

    #[test]
    fn duration_until_next_deadline_is_zero_if_deadline_is_already_missed() {
        let sut = DeadlineQueueBuilder::new().create().unwrap();

        let guard_1 = sut
            .add_deadline_interval(Duration::from_millis(100))
            .unwrap();
        let _guard_2 = sut.add_deadline_interval(Duration::from_secs(1)).unwrap();

        std::thread::sleep(Duration::from_millis(110));

        let next_deadline = sut.duration_until_next_deadline().unwrap();
        assert_that!(next_deadline, eq Duration::ZERO);

        let mut missed_deadline_counter = 0;
        let mut deadline_idx = None;
        sut.missed_deadlines(|idx| {
            missed_deadline_counter += 1;
            deadline_idx = Some(idx);
            CallbackProgression::Continue
        })
        .unwrap();

        assert_that!(missed_deadline_counter, eq 1);
        assert_that!(deadline_idx, eq Some(guard_1.index()));
    }
}
