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

#![allow(clippy::disallowed_types)]

use core::time::Duration;

use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicUsize, Ordering};
use iceoryx2_bb_lock_free::mpmc::bit_set::*;
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::requires_std;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn create_fill_and_reset_works() {
    const CAPACITY: usize = 1234;
    let sut = BitSet::new(CAPACITY);

    assert_that!(sut.capacity(), eq CAPACITY);

    for id in 0..CAPACITY {
        assert_that!(sut.set(id), eq true);
        assert_that!(sut.set(id), eq false);
    }

    let mut id_set = alloc::collections::btree_set::BTreeSet::new();
    let mut counter = 0;
    sut.reset_all(|id| {
        assert_that!(id, lt CAPACITY);
        assert_that!(id_set.insert(id), eq true);
        counter += 1;
    });

    assert_that!(counter, eq CAPACITY);

    let mut counter = 0;
    sut.reset_all(|_| {
        counter += 1;
    });

    assert_that!(counter, eq 0);
}

#[test]
pub fn fixed_size_create_fill_and_reset_works() {
    const CAPACITY: usize = 122;
    let sut = FixedSizeBitSet::<CAPACITY>::new();

    assert_that!(sut.capacity(), eq CAPACITY);

    for id in 0..CAPACITY / 2 {
        assert_that!(sut.set(id * 2), eq true);
        assert_that!(sut.set(id * 2), eq false);
    }

    let mut id_set = alloc::collections::btree_set::BTreeSet::new();
    let mut counter = 0;
    sut.reset_all(|id| {
        assert_that!(id % 2 == 0, eq true);
        assert_that!(id, lt CAPACITY);
        assert_that!(id_set.insert(id), eq true);
        counter += 1;
    });

    assert_that!(counter, eq CAPACITY / 2);

    let mut counter = 0;
    sut.reset_all(|_| {
        counter += 1;
    });

    assert_that!(counter, eq 0);
}

#[test]
pub fn set_single_bit_works() {
    const CAPACITY: usize = 124;
    let sut = BitSet::new(CAPACITY);

    assert_that!(sut.set(55), eq true);
    assert_that!(sut.set(55), eq false);

    sut.reset_all(|id| {
        assert_that!(id, eq 55);
    });

    let mut counter = 0;
    sut.reset_all(|_| {
        counter += 1;
    });

    assert_that!(counter, eq 0);
}

#[cfg_attr(debug_assertions, test)]
#[should_panic]
#[requires_std("panics")]
pub fn set_bit_outside_of_bitset_leads_to_panic() {
    const CAPACITY: usize = 1551;
    let sut = BitSet::new(CAPACITY);

    sut.set(CAPACITY);
}

#[test]
pub fn set_and_reset_next_works() {
    const CAPACITY: usize = 1551;
    let sut = BitSet::new(CAPACITY);

    assert_that!(sut.reset_next(), eq None);
    for i in 0..CAPACITY {
        assert_that!(sut.set(i), eq true);
        assert_that!(sut.reset_next(), eq Some(i));
    }
    assert_that!(sut.reset_next(), eq None);
}

#[test]
pub fn reset_next_is_fair() {
    const CAPACITY: usize = 1551;
    let sut = BitSet::new(CAPACITY);

    assert_that!(sut.set(0), eq true);
    assert_that!(sut.reset_next(), eq Some(0));

    for i in 1..CAPACITY {
        assert_that!(sut.set(i - 1), eq true);
        assert_that!(sut.set(i), eq true);
        assert_that!(sut.reset_next(), eq Some(i));
    }

    for i in 0..CAPACITY - 1 {
        assert_that!(sut.reset_next(), eq Some(i));
    }
    assert_that!(sut.reset_next(), eq None);
}

#[test]
pub fn concurrent_set_and_reset_works() {
    let _watchdog = Watchdog::new_with_timeout(Duration::from_secs(60));

    let number_of_set_threads = (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);
    let number_of_reset_threads = (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);
    const CAPACITY: usize = 10;
    const SUCCESS_LIMIT: usize = 10000;

    let sut = BitSet::new(CAPACITY);
    let start_barrier_handle = BarrierHandle::new();
    let start_barrier =
        BarrierBuilder::new((number_of_set_threads + number_of_reset_threads + 1) as u32)
            .create(&start_barrier_handle)
            .unwrap();

    let set_count = [const { AtomicUsize::new(0) }; CAPACITY];
    let reset_count = [const { AtomicUsize::new(0) }; CAPACITY];

    let keep_running = AtomicBool::new(true);
    let number_of_completed_set_threads = AtomicUsize::new(0);

    thread_scope(|s| {
        for _ in 0..number_of_set_threads {
            s.thread_builder()
                .spawn(|| {
                    let mut counter = 0usize;
                    let mut success_counter = 0;
                    let mut id_counter = [0usize; CAPACITY];

                    start_barrier.wait();
                    while success_counter < SUCCESS_LIMIT {
                        if sut.set(counter % CAPACITY) {
                            id_counter[counter % CAPACITY] += 1;
                            success_counter += 1;
                        }
                        counter += 1;
                    }

                    // Count the sets after the loop to keep contention high
                    for (idx, count) in id_counter.iter().enumerate() {
                        set_count[idx].fetch_add(*count, Ordering::Relaxed);
                    }

                    number_of_completed_set_threads.fetch_add(1, Ordering::SeqCst);
                })
                .expect("failed to spawn thread");
        }

        for _ in 0..number_of_reset_threads {
            s.thread_builder()
                .spawn(|| {
                    let mut id_counter = [0usize; CAPACITY];

                    start_barrier.wait();
                    while keep_running.load(Ordering::SeqCst) {
                        sut.reset_all(|id| {
                            id_counter[id] += 1;
                        });
                    }
                    sut.reset_all(|id| {
                        id_counter[id] += 1;
                    });

                    // Count the sets after the loop to keep contention high
                    for (idx, count) in id_counter.iter().enumerate() {
                        reset_count[idx].fetch_add(*count, Ordering::Relaxed);
                    }
                })
                .expect("failed to spawn thread");
        }

        start_barrier.wait();
        while number_of_completed_set_threads.load(Ordering::Relaxed) < number_of_set_threads {
            nanosleep(Duration::from_millis(1)).expect("sleep failed");
        }
        keep_running.store(false, Ordering::SeqCst);

        Ok(())
    })
    .expect("failed to run thread scope");

    for i in 0..CAPACITY {
        assert_that!(set_count[i].load(Ordering::Relaxed), eq reset_count[i].load(Ordering::Relaxed));
    }
}
