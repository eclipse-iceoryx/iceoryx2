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

use core::sync::atomic::{AtomicUsize, Ordering};
use core::time::Duration;
use iceoryx2_bb_posix::clock::*;
use iceoryx2_bb_posix::read_write_mutex::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use std::sync::Barrier;
use std::thread;

const TIMEOUT: Duration = Duration::from_millis(50);

#[test]
fn read_write_mutex_lock_works() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(456, &handle).unwrap();
    {
        let mut value = sut.write_blocking_lock().unwrap();
        assert_that!(*value, eq 456);
        *value = 123;
    }

    let value = sut.read_blocking_lock().unwrap();
    assert_that!(*value, eq 123);
}

#[test]
fn read_write_mutex_try_lock_works() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(7890, &handle).unwrap();
    {
        let mut value = sut.write_try_lock().unwrap();
        assert_that!(**value.as_mut().unwrap(), eq 7890);
        *value.unwrap() = 551;
    }

    let value = sut.read_try_lock().unwrap();
    assert_that!(*value.unwrap(), eq 551);
}

#[test]
fn read_write_mutex_write_lock_blocks_read_and_write_locks() {
    let _watchdog = Watchdog::new();
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(781, &handle).unwrap();
    let counter = AtomicUsize::new(0);
    let barrier = Barrier::new(3);

    thread::scope(|s| {
        let _guard = sut.write_blocking_lock().unwrap();

        let t1 = s.spawn(|| {
            barrier.wait();
            let _guard = sut.write_blocking_lock().unwrap();
            counter.fetch_add(1, Ordering::Relaxed);
        });

        let t2 = s.spawn(|| {
            barrier.wait();
            let _guard = sut.read_blocking_lock().unwrap();
            counter.fetch_add(1, Ordering::Relaxed);
        });

        barrier.wait();
        nanosleep(TIMEOUT).unwrap();
        let counter_old = counter.load(Ordering::Relaxed);
        drop(_guard);

        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn read_write_mutex_read_lock_blocks_only_write_locks() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(781, &handle).unwrap();
    let counter = AtomicUsize::new(5);

    thread::scope(|s| {
        let _guard = sut.read_blocking_lock().unwrap();
        let _guard2 = sut.read_blocking_lock().unwrap();

        let t1 = s.spawn(|| {
            let _guard = sut.write_blocking_lock().unwrap();
            counter.fetch_add(1, Ordering::Relaxed);
        });

        nanosleep(TIMEOUT * 4).unwrap();
        let counter_old = counter.load(Ordering::Relaxed);
        drop(_guard);
        drop(_guard2);
        assert_that!(t1.join(), is_ok);
        assert_that!(counter_old, eq 5);
        assert_that!(counter.load(Ordering::Relaxed), eq 6);
    });
}

#[test]
fn read_write_mutex_try_lock_fails_when_lock_was_acquired() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(781, &handle).unwrap();
    let _guard = sut.write_blocking_lock().unwrap();

    assert_that!(sut.read_try_lock().unwrap(), is_none);
    drop(_guard);

    let _guard = sut.read_blocking_lock().unwrap();
    assert_that!(sut.write_try_lock().unwrap(), is_none);
}

#[test]
fn read_write_mutex_multiple_ipc_mutex_are_working() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut1 = ReadWriteMutexBuilder::new()
        .is_interprocess_capable(true)
        .create(781, &handle)
        .unwrap();

    let sut2 = unsafe { ReadWriteMutex::from_ipc_handle(&handle) };

    thread::scope(|s| {
        s.spawn(|| {
            let mut guard = sut2.write_blocking_lock().unwrap();
            *guard = 99501;
            nanosleep(TIMEOUT * 4).unwrap();
        });

        nanosleep(TIMEOUT).unwrap();
        let start = Time::now().unwrap();
        sut1.write_blocking_lock().unwrap();
        assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
    });

    assert_that!(*sut2.read_blocking_lock().unwrap(), eq 99501);
}
