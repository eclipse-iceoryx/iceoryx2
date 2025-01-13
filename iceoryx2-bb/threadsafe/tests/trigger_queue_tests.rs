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

use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;
use std::sync::Barrier;
use std::thread;

use iceoryx2_bb_posix::clock::{nanosleep, Time};
use iceoryx2_bb_posix::mutex::MutexHandle;
use iceoryx2_bb_posix::semaphore::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_threadsafe::trigger_queue::*;

const TIMEOUT: Duration = Duration::from_millis(100);
const SUT_CAPACITY: usize = 128;
type Sut<'a> = TriggerQueue<'a, usize, SUT_CAPACITY>;

#[test]
fn trigger_queue_new_queue_is_empty() {
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    assert_that!(!sut.is_full(), eq true);
    assert_that!(sut, is_empty);
    assert_that!(sut.capacity(), eq SUT_CAPACITY);
    assert_that!(sut, len 0);
    assert_that!(sut.try_pop(), eq None);
}

#[test]
fn trigger_queue_try_push_pop_works() {
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    for i in 0..SUT_CAPACITY {
        assert_that!(sut.try_push(i), eq true);
        assert_that!(sut, is_not_empty);
        assert_that!(sut, len i + 1);
    }
    assert_that!(sut.is_full(), eq true);
    assert_that!(!sut.try_push(0), eq true);

    for i in 0..SUT_CAPACITY {
        let value = sut.try_pop();
        assert_that!(value, is_some);
        assert_that!(value.unwrap(), eq i);
        assert_that!(!sut.is_full(), eq true);
    }
    assert_that!(sut, is_empty);
    let value = sut.try_pop();
    assert_that!(value, is_none);
}

#[test]
fn trigger_queue_timed_push_pop_works() {
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    for i in 0..SUT_CAPACITY {
        assert_that!(sut.timed_push(i, TIMEOUT), eq true);
        assert_that!(sut, is_not_empty);
        assert_that!(sut, len i + 1);
    }
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut.try_push(0), eq false);

    for i in 0..SUT_CAPACITY {
        let value = sut.timed_pop(TIMEOUT);
        assert_that!(value, is_some);
        assert_that!(value.unwrap(), eq i);
        assert_that!(!sut.is_full(), eq true);
    }
    assert_that!(sut, is_empty);
    let value = sut.try_pop();
    assert_that!(value, is_none);
}

#[test]
fn trigger_queue_blocking_push_pop_works() {
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    for i in 0..SUT_CAPACITY {
        sut.blocking_push(i);
        assert_that!(sut, is_not_empty);
        assert_that!(sut, len i + 1);
    }
    assert_that!(sut.is_full(), eq true);
    assert_that!(sut.try_push(0), eq false);

    for i in 0..SUT_CAPACITY {
        let value = sut.blocking_pop();
        assert_that!(value, eq i);
        assert_that!(sut.is_full(), eq false);
    }
    assert_that!(sut, is_empty);
    let value = sut.try_pop();
    assert_that!(value, is_none);
}

#[test]
fn trigger_queue_timed_push_blocks_at_least_until_timeout_has_passed() {
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    for _ in 0..SUT_CAPACITY {
        sut.try_push(0);
    }

    let start = Time::now().unwrap();
    assert_that!(sut.timed_push(0, TIMEOUT), eq false);
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
}

#[test]
fn trigger_queue_timed_pop_blocks_at_least_until_timeout_has_passed() {
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    let start = Time::now().unwrap();
    assert_that!(sut.timed_pop(TIMEOUT), is_none);
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
}

#[test]
fn trigger_queue_blocking_push_blocks_until_there_is_space_again() {
    let _watchdog = Watchdog::new();
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    let counter = AtomicU64::new(0);
    for _ in 0..SUT_CAPACITY {
        sut.blocking_push(0);
    }

    thread::scope(|s| {
        s.spawn(|| {
            sut.blocking_push(0);
            counter.store(1, Ordering::SeqCst);
        });

        nanosleep(TIMEOUT).unwrap();
        let counter_old = counter.load(Ordering::SeqCst);
        sut.blocking_pop();

        assert_that!(counter_old, eq 0);

        // if the thread is not unblocked the counter stays zero until the watchdog intervenes
        while counter.load(Ordering::SeqCst) == 0 {}
    });
}

#[test]
fn trigger_queue_blocking_pop_blocks_until_there_is_something_pushed() {
    let _watchdog = Watchdog::new();
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);

    let counter = AtomicU64::new(0);

    thread::scope(|s| {
        s.spawn(|| {
            sut.blocking_pop();
            counter.store(1, Ordering::SeqCst);
        });

        nanosleep(TIMEOUT).unwrap();
        let counter_old = counter.load(Ordering::SeqCst);
        sut.blocking_push(0);

        assert_that!(counter_old, eq 0);

        // if the thread is not unblocked the counter stays zero until the watchdog intervenes
        while counter.load(Ordering::SeqCst) == 0 {}
    });
}

#[test]
fn trigger_queue_one_pop_notifies_exactly_one_blocking_push() {
    let _watchdog = Watchdog::new();
    const NUMBER_OF_THREADS: u64 = 2;
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);
    let barrier = Barrier::new(NUMBER_OF_THREADS as usize + 1);

    let counter = AtomicU64::new(0);
    for _ in 0..SUT_CAPACITY {
        sut.blocking_push(0);
    }

    thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait();
                sut.blocking_push(0);
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        barrier.wait();
        for i in 0..NUMBER_OF_THREADS {
            nanosleep(TIMEOUT).unwrap();
            assert_that!(|| counter.load(Ordering::SeqCst), block_until i);
            sut.blocking_pop();
        }
    });
}

#[test]
fn trigger_queue_one_pop_notifies_exactly_one_timed_push() {
    const NUMBER_OF_THREADS: u64 = 2;

    let _watchdog = Watchdog::new();
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();
    let barrier = Barrier::new(NUMBER_OF_THREADS as usize + 1);

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);
    let counter = AtomicU64::new(0);
    for _ in 0..SUT_CAPACITY {
        sut.blocking_push(0);
    }

    thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait();
                assert_that!(sut.timed_push(0, TIMEOUT * 1000), eq true);
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        barrier.wait();
        for i in 0..NUMBER_OF_THREADS {
            nanosleep(TIMEOUT).unwrap();
            assert_that!(|| counter.load(Ordering::SeqCst), block_until i);
            sut.blocking_pop();
        }
    });
}

#[test]
fn trigger_queue_one_push_notifies_exactly_one_blocking_pop() {
    let _watchdog = Watchdog::new();
    const NUMBER_OF_THREADS: u64 = 2;
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);
    let counter = AtomicU64::new(0);
    let barrier = Barrier::new(NUMBER_OF_THREADS as usize + 1);

    thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait();
                sut.blocking_pop();
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        barrier.wait();

        for i in 0..NUMBER_OF_THREADS {
            nanosleep(TIMEOUT).unwrap();
            assert_that!(|| counter.load(Ordering::SeqCst), block_until i);
            sut.blocking_push(0);
        }
    });
}

#[test]
fn trigger_queue_one_push_notifies_exactly_one_timed_pop() {
    const NUMBER_OF_THREADS: u64 = 2;
    let mtx_handle = MutexHandle::new();
    let free_handle = UnnamedSemaphoreHandle::new();
    let used_handle = UnnamedSemaphoreHandle::new();

    let sut = Sut::new(&mtx_handle, &free_handle, &used_handle);
    let counter = AtomicU64::new(0);
    let barrier = Barrier::new(NUMBER_OF_THREADS as usize + 1);

    thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait();
                sut.timed_pop(TIMEOUT * 1000);
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        barrier.wait();

        for i in 0..NUMBER_OF_THREADS {
            nanosleep(TIMEOUT).unwrap();
            assert_that!(|| counter.load(Ordering::SeqCst), block_until i);
            sut.blocking_push(0);
        }
    });
}
