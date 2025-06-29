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

use iceoryx2_bb_lock_free::spsc::index_queue::*;
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_testing::assert_that;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn spsc_index_queue_push_works_until_full() {
    const CAPACITY: usize = 128;
    let sut = FixedSizeIndexQueue::<CAPACITY>::new();

    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut, len 0);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_empty);

    let mut sut_producer = sut.acquire_producer().unwrap();

    for i in 0..CAPACITY {
        assert_that!(sut, len i);
        assert_that!(sut_producer.push(i as u64), eq true);
    }
    assert_that!(sut_producer.push(1234), eq false);

    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut, len CAPACITY);
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut, is_not_empty);
}

#[test]
fn spsc_index_queue_pop_works_until_empty() {
    const CAPACITY: usize = 128;
    let sut = FixedSizeIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    for i in 0..CAPACITY {
        assert_that!(sut_producer.push(i as u64), eq true);
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
fn spsc_index_queue_push_pop_alteration_works() {
    const CAPACITY: usize = 128;
    let sut = FixedSizeIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    for i in 0..CAPACITY - 1 {
        assert_that!(sut_producer.push(i as u64), eq true);
        assert_that!(sut_producer.push(i as u64), eq true);

        assert_that!(sut_consumer.pop(), eq Some(i as u64 / 2))
    }
}

#[test]
fn spsc_index_queue_get_consumer_twice_fails() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    let _consumer = sut.acquire_consumer().unwrap();
    assert_that!(sut.acquire_consumer(), is_none);
}

#[test]
fn spsc_index_queue_get_consumer_after_release_succeeds() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    {
        let _consumer = sut.acquire_consumer();
    }
    assert_that!(sut.acquire_consumer(), is_some);
}

#[test]
fn spsc_index_queue_get_producer_twice_fails() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    let _producer = sut.acquire_producer().unwrap();
    assert_that!(sut.acquire_producer(), is_none);
}

#[test]
fn spsc_index_queue_get_producer_after_release_succeeds() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    {
        let _producer = sut.acquire_producer();
    }
    assert_that!(sut.acquire_producer(), is_some);
}

#[test]
fn spsc_index_queue_push_pop_works_concurrently() {
    const LIMIT: usize = 1000000;
    const CAPACITY: usize = 1024;

    let sut = FixedSizeIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    let storage = Arc::new(Mutex::<Vec<usize>>::new(vec![]));
    let storage_pop = Arc::clone(&storage);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2)
        .is_interprocess_capable(false)
        .create(&handle)
        .unwrap();

    thread::scope(|s| {
        s.spawn(|| {
            let mut counter: usize = 0;
            barrier.wait();
            while counter <= LIMIT {
                if sut_producer.push(counter as u64) {
                    counter += 1;
                }
            }
        });

        s.spawn(|| {
            let mut guard = storage_pop.lock().unwrap();
            barrier.wait();
            loop {
                if let Some(v) = sut_consumer.pop() {
                    guard.push(v as usize);
                    if v as usize == LIMIT {
                        return;
                    }
                }
            }
        });
    });

    let guard = storage.lock().unwrap();
    for i in 0..LIMIT {
        assert_that!(guard[i], eq i);
    }
}
