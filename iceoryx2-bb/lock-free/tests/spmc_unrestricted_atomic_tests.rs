// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

use core::sync::atomic::{AtomicBool, Ordering};
use std::{sync::Mutex, thread};

use iceoryx2_bb_lock_free::spmc::unrestricted_atomic::*;
use iceoryx2_bb_posix::{barrier::*, system_configuration::SystemInfo};
use iceoryx2_bb_testing::assert_that;

const NUMBER_OF_RUNS: usize = 100000;
const DATA_SIZE: usize = 1024;

static TEST_LOCK: Mutex<bool> = Mutex::new(false);

fn verify(value: u8, rhs: &[u8; DATA_SIZE]) -> bool {
    for i in 0..DATA_SIZE {
        if value != rhs[i] {
            return false;
        }
    }

    true
}

fn verify_no_data_race(rhs: &[u8; DATA_SIZE]) -> bool {
    let value = rhs[0];
    for i in 0..DATA_SIZE {
        if value != rhs[i] {
            return false;
        }
    }

    true
}

#[test]
fn spmc_unrestricted_atomic_acquire_multiple_producer_fails() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let sut = UnrestrictedAtomic::<[u8; DATA_SIZE]>::new([0xff; DATA_SIZE]);

    let p1 = sut.acquire_producer();
    assert_that!(p1, is_some);
    let p2 = sut.acquire_producer();
    assert_that!(p2, is_none);

    drop(p1);

    let p3 = sut.acquire_producer();
    assert_that!(p3, is_some);
}

#[test]
fn spmc_unrestricted_atomic_load_store_works() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let sut = UnrestrictedAtomic::<[u8; DATA_SIZE]>::new([0xff; DATA_SIZE]);
    assert_that!(verify(0xff, &sut.load()), eq true);

    for i in 0..NUMBER_OF_RUNS {
        let idx = i % 255;
        sut.acquire_producer()
            .unwrap()
            .store([(idx) as u8; DATA_SIZE]);
        assert_that!(verify((idx) as u8, &sut.load()), eq true);
    }
}

#[test]
fn spmc_unrestricted_atomic_load_store_works_concurrently() {
    let _test_lock = TEST_LOCK.lock().unwrap();
    let number_of_threads = SystemInfo::NumberOfCpuCores.value();
    let store_finished = AtomicBool::new(false);
    let sut = UnrestrictedAtomic::<[u8; DATA_SIZE]>::new([0xff; DATA_SIZE]);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(number_of_threads as u32 + 1)
        .create(&handle)
        .unwrap();

    thread::scope(|s| {
        for _ in 0..number_of_threads {
            s.spawn(|| {
                barrier.wait();

                while !store_finished.load(Ordering::Relaxed) {
                    assert_that!(verify_no_data_race(&sut.load()), eq true);
                }
            });
        }

        s.spawn(|| {
            barrier.wait();
            let producer = sut.acquire_producer().unwrap();

            for i in 0..NUMBER_OF_RUNS {
                producer.store([(i % 255) as u8; DATA_SIZE]);
            }

            store_finished.store(true, Ordering::Relaxed);
        });
    });
}
