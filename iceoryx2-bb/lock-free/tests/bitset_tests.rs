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

use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::{collections::HashSet, sync::Barrier};

use iceoryx2_bb_lock_free::mpmc::bit_set::*;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};

#[test]
fn bit_set_create_fill_and_reset_works() {
    const CAPACITY: usize = 1234;
    let sut = BitSet::new(CAPACITY);

    assert_that!(sut.capacity(), eq CAPACITY);

    for id in 0..CAPACITY {
        assert_that!(sut.set(id), eq true);
        assert_that!(sut.set(id), eq false);
    }

    let mut id_set = HashSet::new();
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
fn fixed_size_bit_set_create_fill_and_reset_works() {
    const CAPACITY: usize = 122;
    let sut = FixedSizeBitSet::<CAPACITY>::new();

    assert_that!(sut.capacity(), eq CAPACITY);

    for id in 0..CAPACITY / 2 {
        assert_that!(sut.set(id * 2), eq true);
        assert_that!(sut.set(id * 2), eq false);
    }

    let mut id_set = HashSet::new();
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
fn bit_set_set_single_bit_works() {
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

#[test]
#[should_panic]
#[cfg(debug_assertions)]
fn bit_set_set_bit_outside_of_bitset_leads_to_panic() {
    const CAPACITY: usize = 1551;
    let sut = BitSet::new(CAPACITY);

    sut.set(CAPACITY);
}

#[test]
fn bit_set_set_and_reset_next_works() {
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
fn bit_set_reset_next_is_fair() {
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
fn bit_set_concurrent_set_and_reset_works() {
    let _watchdog = Watchdog::new_with_timeout(Duration::from_secs(60));

    let number_of_set_threads = (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);
    let number_of_reset_threads = (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);
    const CAPACITY: usize = 10;
    const SUCCESS_LIMIT: usize = 10000;

    let sut = BitSet::new(CAPACITY);
    let barrier = Barrier::new(number_of_set_threads + number_of_reset_threads + 1);
    let keep_running = AtomicBool::new(true);

    std::thread::scope(|s| {
        let mut set_threads = vec![];
        for _ in 0..number_of_set_threads {
            set_threads.push(s.spawn(|| -> Vec<usize> {
                let mut counter = 0usize;
                let mut success_counter = 0;
                let mut id_counter = vec![0usize; CAPACITY];

                barrier.wait();
                while success_counter < SUCCESS_LIMIT {
                    if sut.set(counter % CAPACITY) {
                        id_counter[counter % CAPACITY] += 1;
                        success_counter += 1;
                    }
                    counter += 1;
                }

                id_counter
            }));
        }

        let mut reset_threads = vec![];
        for _ in 0..number_of_reset_threads {
            reset_threads.push(s.spawn(|| -> Vec<usize> {
                let mut id_counter = vec![0usize; CAPACITY];

                barrier.wait();
                while keep_running.load(Ordering::Relaxed) {
                    sut.reset_all(|id| {
                        id_counter[id] += 1;
                    });
                }

                sut.reset_all(|id| {
                    id_counter[id] += 1;
                });

                id_counter
            }));
        }

        barrier.wait();

        let mut total_set_count = vec![0usize; CAPACITY];
        for t in set_threads {
            let id_count = t.join().unwrap();
            for (n, count) in id_count.iter().enumerate() {
                total_set_count[n] += count;
            }
        }

        keep_running.store(false, Ordering::Relaxed);

        let mut total_reset_count = vec![0usize; CAPACITY];
        for t in reset_threads {
            let id_count = t.join().unwrap();
            for (n, count) in id_count.iter().enumerate() {
                total_reset_count[n] += count;
            }
        }

        assert_that!(total_set_count, eq total_reset_count);
    });
}
