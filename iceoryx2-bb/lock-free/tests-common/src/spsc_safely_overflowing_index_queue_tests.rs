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

#![allow(clippy::disallowed_types)]

use alloc::vec;
use alloc::vec::Vec;
use iceoryx2_bb_posix::mutex::{MutexBuilder, MutexHandle};

use iceoryx2_bb_lock_free::spsc::safely_overflowing_index_queue::*;
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn push_works_until_full() {
    const CAPACITY: usize = 128;
    let sut = FixedSizeSafelyOverflowingIndexQueue::<CAPACITY>::new();

    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut, len 0);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_empty);

    let mut sut_producer = sut.acquire_producer().unwrap();

    for i in 0..CAPACITY {
        assert_that!(sut, len i);
        assert_that!(sut_producer.push(i as u64), is_none);
    }
    let oldest = sut_producer.push(1234);
    assert_that!(oldest, is_some);
    assert_that!(oldest.unwrap(), eq 0);

    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut, len CAPACITY);
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut, is_not_empty);
}

#[test]
pub fn pop_works_until_empty() {
    const CAPACITY: usize = 128;
    let sut = FixedSizeSafelyOverflowingIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    for i in 0..CAPACITY {
        assert_that!(sut_producer.push(i as u64), is_none);
    }

    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut, is_not_empty);
    assert_that!(sut, len CAPACITY);

    let mut sut_consumer = sut.acquire_consumer().unwrap();
    for i in 0..CAPACITY {
        assert_that!(sut, len CAPACITY - i);
        let result = sut_consumer.pop();
        assert_that!(result, is_some);
        assert_that!(result.unwrap(), eq i as u64);
    }
    assert_that!(sut_consumer.pop(), is_none);

    assert_that!(sut, len 0);
    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_empty);
}

#[test]
pub fn push_pop_alteration_works() {
    const CAPACITY: usize = 128;
    let sut = FixedSizeSafelyOverflowingIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    for i in 0..CAPACITY - 1 {
        assert_that!(sut_producer.push(i as u64), is_none);
        assert_that!(sut_producer.push(i as u64), is_none);

        assert_that!(sut_consumer.pop(), eq Some(i as u64 / 2))
    }
}

#[test]
pub fn get_consumer_twice_fails() {
    let sut = FixedSizeSafelyOverflowingIndexQueue::<1024>::new();
    let _consumer = sut.acquire_consumer().unwrap();
    assert_that!(sut.acquire_consumer(), is_none);
}

#[test]
pub fn get_consumer_after_release_succeeds() {
    let sut = FixedSizeSafelyOverflowingIndexQueue::<1024>::new();
    {
        let _consumer = sut.acquire_consumer();
    }
    assert_that!(sut.acquire_consumer(), is_some);
}

#[test]
pub fn get_producer_twice_fails() {
    let sut = FixedSizeSafelyOverflowingIndexQueue::<1024>::new();
    let _producer = sut.acquire_producer().unwrap();
    assert_that!(sut.acquire_producer(), is_none);
}

#[test]
pub fn get_producer_after_release_succeeds() {
    let sut = FixedSizeSafelyOverflowingIndexQueue::<1024>::new();
    {
        let _producer = sut.acquire_producer();
    }
    assert_that!(sut.acquire_producer(), is_some);
}

#[test]
pub fn push_pop_works_concurrently() {
    const LIMIT: u64 = 1000000;
    const CAPACITY: usize = 1024;

    let sut = FixedSizeSafelyOverflowingIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2)
        .is_interprocess_capable(false)
        .create(&handle)
        .unwrap();

    let pushed_indexes_handle = MutexHandle::<Vec<u64>>::new();
    let pushed_indexes = MutexBuilder::new()
        .create(Vec::new(), &pushed_indexes_handle)
        .expect("Failed to create mutex");
    let popped_indexes_handle = MutexHandle::<Vec<u64>>::new();
    let popped_indexes = MutexBuilder::new()
        .create(Vec::new(), &popped_indexes_handle)
        .expect("Failed to create mutex");

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut counter: u64 = 0;
                let mut guard = pushed_indexes.lock().expect("failed to acquire mutex");

                barrier.wait();
                while counter <= LIMIT {
                    if let Some(idx) = sut_producer.push(counter) {
                        guard.push(idx);
                    }
                    counter += 1;
                }
            })
            .expect("failed to spawn thread");

        s.thread_builder()
            .spawn(|| {
                let mut guard = popped_indexes.lock().expect("failed to acquire mutex");

                barrier.wait();
                loop {
                    if let Some(idx) = sut_consumer.pop() {
                        guard.push(idx);
                        if idx == LIMIT {
                            return;
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to run thread scope");

    let mut element_counter = vec![0; LIMIT as usize + 1];

    let guard = &*pushed_indexes.lock().expect("failed to acquire mutex");
    for idx in guard {
        element_counter[*idx as usize] += 1;
    }
    let guard = &*popped_indexes.lock().expect("failed to acquire mutex");
    for idx in guard {
        element_counter[*idx as usize] += 1;
    }

    for element in element_counter {
        assert_that!(element, eq 1);
    }
}

#[test]
pub fn push_pop_works_concurrently_with_full_queue() {
    const LIMIT: u64 = 1000000;
    const CAPACITY: usize = 1024;
    const NUMBER_OF_THREADS: u32 = 2;

    let sut = FixedSizeSafelyOverflowingIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(NUMBER_OF_THREADS)
        .is_interprocess_capable(false)
        .create(&handle)
        .unwrap();

    let pushed_indexes_handle = MutexHandle::<Vec<u64>>::new();
    let pushed_indexes = MutexBuilder::new()
        .create(Vec::new(), &pushed_indexes_handle)
        .expect("Failed to create mutex");
    let popped_indexes_handle = MutexHandle::<Vec<u64>>::new();
    let popped_indexes = MutexBuilder::new()
        .create(Vec::new(), &popped_indexes_handle)
        .expect("Failed to create mutex");

    for i in 0..CAPACITY {
        assert_that!(sut_producer.push(i as u64), is_none);
    }

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut counter: u64 = 1024;
                let mut guard = pushed_indexes.lock().expect("failed to acquire mutex");

                barrier.wait();
                while counter <= LIMIT {
                    if let Some(idx) = sut_producer.push(counter) {
                        guard.push(idx);
                    }
                    counter += 1;
                }
            })
            .expect("failed to spawn thread");

        s.thread_builder()
            .spawn(|| {
                let mut guard = popped_indexes.lock().expect("failed to acquire mutex");

                barrier.wait();
                loop {
                    if let Some(idx) = sut_consumer.pop() {
                        guard.push(idx);
                        if idx == LIMIT {
                            return;
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to run thread scope");

    let mut element_counter = vec![0; LIMIT as usize + 1];

    let guard = &*pushed_indexes.lock().expect("failed to acquire mutex");
    for idx in guard {
        element_counter[*idx as usize] += 1;
    }
    let guard = &*popped_indexes.lock().expect("failed to acquire mutex");
    for idx in guard {
        element_counter[*idx as usize] += 1;
    }

    for element in element_counter {
        assert_that!(element, eq 1);
    }
}
