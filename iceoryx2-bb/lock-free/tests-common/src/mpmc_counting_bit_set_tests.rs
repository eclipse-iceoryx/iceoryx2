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

use core::time::Duration;

use iceoryx2_bb_concurrency::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use iceoryx2_bb_lock_free::mpmc::counting_bit_set::{CountingBitSet, FixedSizeCountingBitSet};
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

const SUT_CAPACITY: usize = 32;

type FixedSizeSut = FixedSizeCountingBitSet<SUT_CAPACITY>;

#[test]
pub fn set_every_bit_individually_works() {
    let sut = FixedSizeSut::new();

    for i in 0..SUT_CAPACITY {
        sut.set(i);
        let mut callback_counter = 0;
        sut.reset_all(|state| {
            callback_counter += 1;
            assert_that!(state.bit(), eq i);
            assert_that!(state.count(), eq 1);
        });
        assert_that!(callback_counter, eq 1);
    }
}

#[test]
pub fn set_every_bit_multiple_times_works() {
    const NUMBER_OF_SETS: usize = 5;
    let sut = FixedSizeSut::new();

    for i in 0..SUT_CAPACITY {
        for _ in 0..NUMBER_OF_SETS {
            sut.set(i);
        }
        let mut callback_counter = 0;
        sut.reset_all(|state| {
            callback_counter += 1;
            assert_that!(state.bit(), eq i);
            assert_that!(state.count(), eq NUMBER_OF_SETS as u64);
        });
        assert_that!(callback_counter, eq 1);
    }
}

#[test]
pub fn set_all_bits_at_once() {
    let sut = FixedSizeSut::new();

    for n in 0..SUT_CAPACITY {
        for _ in 0..n {
            sut.set(n);
        }
    }

    let mut callback_counter = 0;
    sut.reset_all(|state| {
        callback_counter += 1;
        assert_that!(state.bit(), eq state.count() as usize);
    });
    assert_that!(callback_counter, eq SUT_CAPACITY - 1);
}

#[test]
pub fn reset_sets_all_counters_to_zero() {
    let sut = FixedSizeSut::new();

    for n in 0..SUT_CAPACITY {
        sut.set(n);
    }

    let mut callback_counter = 0;
    sut.reset_all(|_| {
        callback_counter += 1;
    });
    assert_that!(callback_counter, eq SUT_CAPACITY);

    callback_counter = 0;
    sut.reset_all(|_| {
        callback_counter += 1;
    });
    assert_that!(callback_counter, eq 0);
}

#[test]
pub fn heap_based_counting_bitset_works() {
    let sut = CountingBitSet::new(SUT_CAPACITY);

    for i in 0..SUT_CAPACITY {
        sut.set(i);
        let mut callback_counter = 0;
        sut.reset_all(|state| {
            callback_counter += 1;
            assert_that!(state.bit(), eq i);
            assert_that!(state.count(), eq 1);
        });
        assert_that!(callback_counter, eq 1);
    }
}

#[test]
pub fn concurrent_set_and_reset_works() {
    let _watchdog = Watchdog::new();

    let number_of_set_threads = (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);
    let number_of_reset_threads = (SystemInfo::NumberOfCpuCores.value() / 2).clamp(2, usize::MAX);
    const CAPACITY: usize = 10;
    const ITERATIONS: usize = 100000;

    let sut = CountingBitSet::new(CAPACITY);
    let start_barrier_handle = BarrierHandle::new();
    let start_barrier =
        BarrierBuilder::new((number_of_set_threads + number_of_reset_threads + 1) as u32)
            .create(&start_barrier_handle)
            .unwrap();

    let set_count = [const { AtomicU64::new(0) }; CAPACITY];
    let reset_count = [const { AtomicU64::new(0) }; CAPACITY];

    let keep_running = AtomicBool::new(true);
    let number_of_completed_set_threads = AtomicUsize::new(0);

    thread_scope(|s| {
        for _ in 0..number_of_set_threads {
            s.thread_builder()
                .spawn(|| {
                    let mut counter = 0;
                    let mut id_counter = [0u64; CAPACITY];

                    start_barrier.wait();
                    while counter < ITERATIONS {
                        sut.set(counter % CAPACITY);
                        id_counter[counter % CAPACITY] += 1;
                        counter += 1;
                    }

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
                    let mut id_counter = [0u64; CAPACITY];

                    start_barrier.wait();
                    while keep_running.load(Ordering::SeqCst) {
                        sut.reset_all(|state| {
                            id_counter[state.bit()] += state.count();
                        });
                    }
                    sut.reset_all(|state| {
                        id_counter[state.bit()] += state.count();
                    });

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
