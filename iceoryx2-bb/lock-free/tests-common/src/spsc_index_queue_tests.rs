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

use iceoryx2_bb_lock_free::spsc::index_queue::*;
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;

#[test]
pub fn push_works_until_full() {
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
pub fn pop_works_until_empty() {
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
pub fn push_pop_alteration_works() {
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
pub fn get_consumer_twice_fails() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    let _consumer = sut.acquire_consumer().unwrap();
    assert_that!(sut.acquire_consumer(), is_none);
}

#[test]
pub fn get_consumer_after_release_succeeds() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    {
        let _consumer = sut.acquire_consumer();
    }
    assert_that!(sut.acquire_consumer(), is_some);
}

#[test]
pub fn get_producer_twice_fails() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    let _producer = sut.acquire_producer().unwrap();
    assert_that!(sut.acquire_producer(), is_none);
}

#[test]
pub fn get_producer_after_release_succeeds() {
    let sut = FixedSizeIndexQueue::<1024>::new();
    {
        let _producer = sut.acquire_producer();
    }
    assert_that!(sut.acquire_producer(), is_some);
}

#[test]
pub fn push_pop_works_concurrently() {
    const LIMIT: usize = 1000000;
    const CAPACITY: usize = 1024;

    let sut = FixedSizeIndexQueue::<CAPACITY>::new();
    let mut sut_producer = sut.acquire_producer().unwrap();
    let mut sut_consumer = sut.acquire_consumer().unwrap();

    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2)
        .is_interprocess_capable(false)
        .create(&handle)
        .unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut counter: usize = 0;
                barrier.wait();
                while counter <= LIMIT {
                    if sut_producer.push(counter as u64) {
                        counter += 1;
                    }
                }
            })
            .expect("failed to spawn thread");

        s.thread_builder()
            .spawn(|| {
                let mut expected: usize = 0;
                barrier.wait();
                loop {
                    if let Some(value) = sut_consumer.pop() {
                        assert_that!(value as usize, eq expected);
                        expected += 1;
                        if value as usize == LIMIT {
                            return;
                        }
                    }
                }
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to run thread scope");
}
