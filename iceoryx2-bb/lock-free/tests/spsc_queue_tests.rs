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

use iceoryx2_bb_lock_free::spsc::queue::*;
use iceoryx2_bb_testing::assert_that;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn spsc_queue_push_works_until_full() {
    const CAPACITY: usize = 128;
    let sut = Queue::<i64, CAPACITY>::new();

    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut, len 0);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_empty);

    let mut sut_producer = sut.acquire_producer().unwrap();

    for i in 0..CAPACITY {
        assert_that!(sut, len i);
        assert_that!(sut_producer.push(&(i as i64)), eq true);
    }
    assert_that!(sut_producer.push(&1234), eq false);

    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut, len CAPACITY);
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut, is_not_empty);
}

#[test]
fn spsc_queue_pop_works_until_empty() {
    const CAPACITY: usize = 128;
    let sut = Queue::<i64, CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    for i in 0..CAPACITY {
        assert_that!(sut_producer.push(&(i as i64)), eq true);
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
        assert_that!(result.unwrap(), eq i as i64);
    }
    assert_that!(sut_consumer.pop(), is_none);

    assert_that!(sut, len 0);
    assert_that!(sut.capacity(), eq CAPACITY);
    assert_that!(sut.is_full(), eq false);
    assert_that!(sut, is_empty);
}

#[test]
fn spsc_queue_push_pop_alteration_works() {
    const CAPACITY: usize = 128;
    let sut = Queue::<i64, CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    for i in 0..CAPACITY - 1 {
        assert_that!(sut_producer.push(&(i as i64)), eq true);
        assert_that!(sut_producer.push(&(i as i64)), eq true);

        assert_that!(sut_consumer.pop(), eq Some(i as i64 / 2))
    }
}

#[test]
fn spsc_queue_get_consumer_twice_fails() {
    let sut = Queue::<i64, 1024>::new();
    let _consumer = sut.acquire_consumer().unwrap();
    assert_that!(sut.acquire_consumer(), is_none);
}

#[test]
fn spsc_queue_get_consumer_after_release_succeeds() {
    let sut = Queue::<i64, 1024>::new();
    {
        let _consumer = sut.acquire_consumer();
    }
    assert_that!(sut.acquire_consumer(), is_some);
}

#[test]
fn spsc_queue_get_producer_twice_fails() {
    let sut = Queue::<i64, 1024>::new();
    let _producer = sut.acquire_producer().unwrap();
    assert_that!(sut.acquire_producer(), is_none);
}

#[test]
fn spsc_queue_get_producer_after_release_succeeds() {
    let sut = Queue::<i64, 1024>::new();
    {
        let _producer = sut.acquire_producer();
    }
    assert_that!(sut.acquire_producer(), is_some);
}

#[test]
fn spsc_queue_push_pop_works_concurrently() {
    const LIMIT: i64 = 10000;
    const CAPACITY: usize = 1024;

    let sut = Queue::<i64, CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    let storage = Arc::new(Mutex::<Vec<i64>>::new(vec![]));
    let storage_pop = Arc::clone(&storage);

    thread::scope(|s| {
        s.spawn(|| {
            let mut counter: i64 = 0;
            while counter <= LIMIT {
                if sut_producer.push(&counter) {
                    counter += 1;
                }
            }
        });

        s.spawn(|| {
            let mut guard = storage_pop.lock().unwrap();
            loop {
                if let Some(v) = sut_consumer.pop() {
                    guard.push(v);
                    if v == LIMIT {
                        return;
                    }
                }
            }
        });
    });

    let guard = storage.lock().unwrap();
    for i in 0..LIMIT {
        assert_that!(guard[i as usize], eq i);
    }
}
