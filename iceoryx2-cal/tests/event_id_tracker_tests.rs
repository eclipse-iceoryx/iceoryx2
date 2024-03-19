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

#[generic_tests::define]
mod event_id_tracker {
    use std::collections::HashSet;

    use iceoryx2_bb_lock_free::mpmc::bit_set::BitSet;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_cal::event::{id_tracker::IdTracker, TriggerId};

    trait NewSut {
        fn new_sut(capacity: usize) -> Self;
    }

    impl NewSut for BitSet {
        fn new_sut(capacity: usize) -> Self {
            BitSet::new(capacity)
        }
    }
    #[test]
    fn max_trigger_id_must_be_at_least_capacity<Sut: IdTracker + NewSut>() {
        const CAPACITY: usize = 5234;

        let sut = Sut::new_sut(CAPACITY);
        assert_that!(sut.trigger_id_max().as_u64(), ge CAPACITY as u64);
    }

    #[test]
    fn add_and_acquire_works<Sut: IdTracker + NewSut>() {
        const CAPACITY: usize = 1234;

        let sut = Sut::new_sut(CAPACITY);

        assert_that!(sut.acquire(), eq None);
        for i in 0..CAPACITY {
            let id = TriggerId::new(i as u64);
            assert_that!(sut.add(id), is_ok);
            assert_that!(sut.acquire(), eq Some(id));
            assert_that!(sut.acquire(), is_none);
        }
    }

    #[test]
    fn add_until_full_and_then_acquire_works<Sut: IdTracker + NewSut>() {
        const CAPACITY: usize = 1234;

        let sut = Sut::new_sut(CAPACITY);

        for i in 0..CAPACITY {
            let id = TriggerId::new((i as u64).min(sut.trigger_id_max().as_u64()));
            assert_that!(sut.add(id), is_ok);
        }

        let mut ids = HashSet::new();
        for _ in 0..CAPACITY {
            let result = sut.acquire().unwrap();
            assert_that!(result, le sut.trigger_id_max());
            assert_that!(ids.insert(result), eq true);
        }

        assert_that!(sut.acquire(), is_none);
    }

    #[test]
    fn add_and_acquire_all_works<Sut: IdTracker + NewSut>() {
        const CAPACITY: usize = 3234;

        let sut = Sut::new_sut(CAPACITY);

        for i in 0..CAPACITY {
            let id = TriggerId::new((i as u64).min(sut.trigger_id_max().as_u64()));
            assert_that!(sut.add(id), is_ok);
        }

        let mut ids = HashSet::new();
        sut.acquire_all(|id| {
            assert_that!(id, le sut.trigger_id_max());
            assert_that!(ids.insert(id), eq true);
        });

        let mut callback_called = false;
        sut.acquire_all(|_| callback_called = true);
        assert_that!(callback_called, eq false);

        assert_that!(ids, len CAPACITY);
    }

    #[test]
    fn add_acquire_and_acquire_all_works<Sut: IdTracker + NewSut>() {
        const CAPACITY: usize = 234;

        let sut = Sut::new_sut(CAPACITY);

        for i in 0..CAPACITY {
            let id = TriggerId::new((i as u64).min(sut.trigger_id_max().as_u64()));
            assert_that!(sut.add(id), is_ok);
        }

        let mut ids = HashSet::new();
        for _ in 0..CAPACITY / 2 {
            let result = sut.acquire().unwrap();
            assert_that!(result, le sut.trigger_id_max());
            assert_that!(ids.insert(result), eq true);
        }

        sut.acquire_all(|id| {
            assert_that!(id, le sut.trigger_id_max());
            assert_that!(ids.insert(id), eq true);
        });

        assert_that!(ids, len CAPACITY);
    }

    #[instantiate_tests(<BitSet>)]
    mod bitset {}
}
