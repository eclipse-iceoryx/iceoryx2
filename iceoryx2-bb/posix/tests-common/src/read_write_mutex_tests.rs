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

use core::time::Duration;
use iceoryx2_bb_concurrency::atomic::{AtomicUsize, Ordering};
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::{nanosleep, Time};
use iceoryx2_bb_posix::read_write_mutex::*;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

const TIMEOUT: Duration = Duration::from_millis(50);

#[test]
pub fn lock_works() {
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
pub fn try_lock_works() {
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
pub fn write_lock_blocks_read_and_write_locks() {
    const NUMBER_OF_THREADS: u32 = 2;

    let _watchdog = Watchdog::new();
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(781, &handle).unwrap();
    let counter = AtomicUsize::new(0);
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(NUMBER_OF_THREADS + 1)
        .create(&barrier_handle)
        .unwrap();

    thread_scope(|s| {
        let _guard = sut.write_blocking_lock().unwrap();

        s.thread_builder()
            .spawn(|| {
                barrier.wait();
                let _guard = sut.write_blocking_lock().unwrap();
                counter.fetch_add(1, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        s.thread_builder()
            .spawn(|| {
                barrier.wait();
                let _guard = sut.read_blocking_lock().unwrap();
                counter.fetch_add(1, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        barrier.wait();
        nanosleep(TIMEOUT).expect("failed to sleep");
        let counter_old = counter.load(Ordering::Relaxed);
        drop(_guard);

        assert_that!(counter_old, eq 0);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(counter.load(Ordering::Relaxed), eq NUMBER_OF_THREADS as usize);
}

#[test]
pub fn read_lock_blocks_only_write_locks() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(781, &handle).unwrap();
    let counter = AtomicUsize::new(5);

    thread_scope(|s| {
        let _guard = sut.read_blocking_lock().unwrap();
        let _guard2 = sut.read_blocking_lock().unwrap();

        s.thread_builder()
            .spawn(|| {
                let _guard = sut.write_blocking_lock().unwrap();
                counter.fetch_add(1, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        nanosleep(TIMEOUT * 4).expect("failed to sleep");
        let counter_old = counter.load(Ordering::Relaxed);
        assert_that!(counter_old, eq 5);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(counter.load(Ordering::Relaxed), eq 6);
}

#[test]
pub fn try_lock_fails_when_lock_was_acquired() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut = ReadWriteMutexBuilder::new().create(781, &handle).unwrap();
    let _guard = sut.write_blocking_lock().unwrap();

    assert_that!(sut.read_try_lock().unwrap(), is_none);
    drop(_guard);

    let _guard = sut.read_blocking_lock().unwrap();
    assert_that!(sut.write_try_lock().unwrap(), is_none);
}

#[test]
pub fn multiple_ipc_mutex_are_working() {
    let handle = ReadWriteMutexHandle::<i32>::new();
    let sut1 = ReadWriteMutexBuilder::new()
        .is_interprocess_capable(true)
        .create(781, &handle)
        .unwrap();

    let sut2 = unsafe { ReadWriteMutex::from_ipc_handle(&handle) };

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut guard = sut2.write_blocking_lock().unwrap();
                *guard = 99501;
                nanosleep(TIMEOUT * 4).expect("failed to sleep");
            })
            .expect("failed to spawn thread");

        nanosleep(TIMEOUT).expect("failed to sleep");
        let start = Time::now().unwrap();
        sut1.write_blocking_lock().unwrap();
        assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(*sut2.read_blocking_lock().unwrap(), eq 99501);
}
