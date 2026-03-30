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
use iceoryx2_bb_posix::clock::*;
use iceoryx2_bb_posix::semaphore::*;
use iceoryx2_bb_posix::system_configuration::Feature;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing::watchdog::Watchdog;
#[cfg(debug_assertions)]
use iceoryx2_bb_testing_macros::requires_std;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_NAMED_SEMAPHORE;

pub struct NamedSemaphoreTest {
    monotonic_named_sut1: NamedSemaphore,
    monotonic_named_sut2: NamedSemaphore,

    realtime_named_sut1: NamedSemaphore,
    realtime_named_sut2: NamedSemaphore,
}

const TIMEOUT: Duration = Duration::from_millis(25);

impl NamedSemaphoreTest {
    fn new() -> Self {
        let monotonic_name = generate_file_path().file_name();
        let realtime_name = generate_file_path().file_name();

        Self {
            monotonic_named_sut1: NamedSemaphoreBuilder::new(&monotonic_name)
                .clock_type(if Feature::MonotonicClock.is_available() {
                    ClockType::Monotonic
                } else {
                    ClockType::default()
                })
                .creation_mode(CreationMode::PurgeAndCreate)
                .initial_value(0)
                .permission(Permission::OWNER_ALL)
                .create()
                .unwrap(),
            monotonic_named_sut2: NamedSemaphoreBuilder::new(&monotonic_name)
                .clock_type(if Feature::MonotonicClock.is_available() {
                    ClockType::Monotonic
                } else {
                    ClockType::default()
                })
                .open_existing()
                .unwrap(),
            realtime_named_sut1: NamedSemaphoreBuilder::new(&realtime_name)
                .clock_type(ClockType::Realtime)
                .creation_mode(CreationMode::PurgeAndCreate)
                .initial_value(0)
                .permission(Permission::OWNER_ALL)
                .create()
                .unwrap(),
            realtime_named_sut2: NamedSemaphoreBuilder::new(&realtime_name)
                .clock_type(ClockType::Realtime)
                .open_existing()
                .unwrap(),
        }
    }
}

struct UnnamedSemaphoreTest<'a> {
    monotonic_unnamed_sut: UnnamedSemaphore<'a>,
    realtime_unnamed_sut: UnnamedSemaphore<'a>,
}

impl<'a> UnnamedSemaphoreTest<'a> {
    fn new(handle1: &'a UnnamedSemaphoreHandle, handle2: &'a UnnamedSemaphoreHandle) -> Self {
        Self {
            monotonic_unnamed_sut: UnnamedSemaphoreBuilder::new()
                .initial_value(0)
                .clock_type(if Feature::MonotonicClock.is_available() {
                    ClockType::Monotonic
                } else {
                    ClockType::default()
                })
                .create(handle1)
                .unwrap(),
            realtime_unnamed_sut: UnnamedSemaphoreBuilder::new()
                .initial_value(0)
                .clock_type(ClockType::Realtime)
                .create(handle2)
                .unwrap(),
        }
    }
}

#[test]
pub fn named_semaphore_initializes_correctly() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);

    let initial_value = 4;
    let sem_name = generate_file_path().file_name();
    let sut = NamedSemaphoreBuilder::new(&sem_name)
        .clock_type(ClockType::Realtime)
        .creation_mode(CreationMode::PurgeAndCreate)
        .initial_value(initial_value)
        .permission(Permission::OWNER_ALL)
        .create()
        .unwrap();

    assert_that!(*sut.name(), eq sem_name);
    assert_that!(sut.clock_type(), eq ClockType::Realtime);

    for _i in 0..initial_value {
        assert_that!(sut.try_wait().unwrap(), eq true);
    }
    assert_that!(sut.try_wait().unwrap(), eq false);
}

#[test]
pub fn named_semaphore_opens_correctly() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);

    let initial_value = 7;
    let sem_name = generate_file_path().file_name();
    let _creator = NamedSemaphoreBuilder::new(&sem_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .initial_value(initial_value)
        .permission(Permission::OWNER_ALL)
        .create()
        .unwrap();

    let sut = NamedSemaphoreBuilder::new(&sem_name)
        .clock_type(ClockType::Monotonic)
        .open_existing()
        .unwrap();

    assert_that!(sut.clock_type(), eq ClockType::Monotonic);

    for _i in 0..initial_value {
        assert_that!(sut.try_wait().unwrap(), eq true);
    }
    assert_that!(sut.try_wait().unwrap(), eq false);
}

#[test]
pub fn unnamed_semaphore_initializes_correctly() {
    let sut_handle = UnnamedSemaphoreHandle::new();

    let initial_value = 5;
    let sut = UnnamedSemaphoreBuilder::new()
        .is_interprocess_capable(false)
        .clock_type(ClockType::default())
        .initial_value(initial_value)
        .create(&sut_handle)
        .expect("");

    assert_that!(sut.is_interprocess_capable(), eq false);
    assert_that!(sut.clock_type(), eq ClockType::default());

    for _i in 0..initial_value {
        assert_that!(sut.try_wait().unwrap(), eq true);
    }
    assert_that!(sut.try_wait().unwrap(), eq false);
}

pub fn post_and_try_wait_work<T: SemaphoreInterface>(initial_value: u64, sut: &T) {
    for _i in 0..initial_value {
        sut.post().unwrap();
    }

    for _i in 0..initial_value {
        assert_that!(sut.try_wait().unwrap(), eq true);
    }
    assert_that!(sut.try_wait().unwrap(), eq false);
    assert_that!(sut.try_wait().unwrap(), eq false);
}

#[test]
pub fn named_semaphore_post_and_try_wait_work() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);

    let test = NamedSemaphoreTest::new();

    post_and_try_wait_work(78, &test.monotonic_named_sut1);
    post_and_try_wait_work(12, &test.monotonic_named_sut2);

    post_and_try_wait_work(98, &test.realtime_named_sut1);
    post_and_try_wait_work(92, &test.realtime_named_sut2);
}

#[test]
pub fn unnamed_semaphore_post_and_try_wait_work() {
    let handle1 = UnnamedSemaphoreHandle::new();
    let handle2 = UnnamedSemaphoreHandle::new();

    let test = UnnamedSemaphoreTest::new(&handle1, &handle2);

    post_and_try_wait_work(14, &test.monotonic_unnamed_sut);
    post_and_try_wait_work(14, &test.realtime_unnamed_sut);
}

pub fn post_and_wait_work<T: SemaphoreInterface>(initial_value: u64, sut: &T) {
    for _i in 0..initial_value {
        sut.post().unwrap();
    }

    for _i in 0..initial_value {
        assert_that!(sut.blocking_wait(), is_ok);
    }
}

#[test]
pub fn named_semaphore_post_and_wait_work() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);

    let test = NamedSemaphoreTest::new();

    post_and_wait_work(78, &test.monotonic_named_sut1);
    post_and_wait_work(12, &test.monotonic_named_sut2);

    post_and_wait_work(98, &test.realtime_named_sut1);
    post_and_wait_work(92, &test.realtime_named_sut2);
}

#[test]
pub fn unnamed_semaphore_post_and_wait_work() {
    let handle1 = UnnamedSemaphoreHandle::new();
    let handle2 = UnnamedSemaphoreHandle::new();

    let test = UnnamedSemaphoreTest::new(&handle1, &handle2);

    post_and_wait_work(14, &test.monotonic_unnamed_sut);
    post_and_wait_work(19, &test.realtime_unnamed_sut);
}

pub fn post_and_timed_wait_work<T: SemaphoreInterface>(initial_value: u64, sut: &T) {
    for _i in 0..initial_value {
        sut.post().unwrap();
    }

    for _i in 0..initial_value {
        assert_that!(sut.timed_wait(TIMEOUT).unwrap(), eq true);
    }

    assert_that!(sut.timed_wait(TIMEOUT).unwrap(), eq false);
    assert_that!(sut.timed_wait(TIMEOUT).unwrap(), eq false);
}

#[test]
pub fn named_semaphore_post_and_timed_wait_work() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);

    let test = NamedSemaphoreTest::new();

    post_and_timed_wait_work(78, &test.monotonic_named_sut1);
    post_and_timed_wait_work(12, &test.monotonic_named_sut2);

    post_and_timed_wait_work(98, &test.realtime_named_sut1);
    post_and_timed_wait_work(92, &test.realtime_named_sut2);
}

#[test]
pub fn unnamed_semaphore_post_and_timed_wait_work() {
    let handle1 = UnnamedSemaphoreHandle::new();
    let handle2 = UnnamedSemaphoreHandle::new();

    let test = UnnamedSemaphoreTest::new(&handle1, &handle2);

    post_and_timed_wait_work(14, &test.monotonic_unnamed_sut);
    post_and_timed_wait_work(19, &test.realtime_unnamed_sut);
}

pub fn wait_blocks<T: SemaphoreInterface + Send + Sync>(sut1: &T, sut2: &T) {
    let _watchdog = Watchdog::new();
    let counter = AtomicUsize::new(0);
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                barrier.wait();
                sut1.blocking_wait().unwrap();
                counter.fetch_add(1, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        barrier.wait();
        nanosleep(TIMEOUT).expect("failed to sleep");
        let counter_old = counter.load(Ordering::Relaxed);
        sut2.post().unwrap();

        assert_that!(counter_old, eq 0);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(counter.load(Ordering::Relaxed), eq 1);
}

#[test]
pub fn named_semaphore_wait_blocks() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);
    let test = NamedSemaphoreTest::new();

    wait_blocks(&test.monotonic_named_sut1, &test.monotonic_named_sut2);
    wait_blocks(&test.realtime_named_sut1, &test.realtime_named_sut2);
}

#[test]
pub fn unnamed_semaphore_wait_blocks() {
    let handle1 = UnnamedSemaphoreHandle::new();
    let handle2 = UnnamedSemaphoreHandle::new();

    let test = UnnamedSemaphoreTest::new(&handle1, &handle2);

    wait_blocks(&test.monotonic_unnamed_sut, &test.monotonic_unnamed_sut);
    wait_blocks(&test.realtime_unnamed_sut, &test.realtime_unnamed_sut);
}

pub fn timed_wait_blocks<T: SemaphoreInterface + Send + Sync>(sut1: &T, sut2: &T) {
    let counter = AtomicUsize::new(0);

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                sut1.timed_wait(TIMEOUT * 10).unwrap();
                counter.fetch_add(1, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        nanosleep(TIMEOUT).expect("failed to sleep");
        let counter_old = counter.load(Ordering::Relaxed);
        sut2.post().unwrap();

        assert_that!(counter_old, eq 0);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(counter.load(Ordering::Relaxed), eq 1);
}

#[test]
pub fn named_semaphore_timed_wait_blocks() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);

    let test = NamedSemaphoreTest::new();

    timed_wait_blocks(&test.monotonic_named_sut1, &test.monotonic_named_sut2);
    timed_wait_blocks(&test.realtime_named_sut1, &test.realtime_named_sut2);
}

#[test]
pub fn unnamed_semaphore_timed_wait_blocks() {
    let handle1 = UnnamedSemaphoreHandle::new();
    let handle2 = UnnamedSemaphoreHandle::new();

    let test = UnnamedSemaphoreTest::new(&handle1, &handle2);

    timed_wait_blocks(&test.monotonic_unnamed_sut, &test.monotonic_unnamed_sut);
    timed_wait_blocks(&test.realtime_unnamed_sut, &test.realtime_unnamed_sut);
}

pub fn timed_wait_waits_at_least_timeout<T: SemaphoreInterface>(sut: &T) {
    let now = Time::now_with_clock(sut.get_clock_type()).unwrap();
    sut.timed_wait(TIMEOUT).unwrap();
    assert_that!(now.elapsed().unwrap(), time_at_least TIMEOUT);
}

#[test]
pub fn named_semaphore_timed_wait_waits_at_least_timeout() {
    test_requires!(POSIX_SUPPORT_NAMED_SEMAPHORE);

    let test = NamedSemaphoreTest::new();

    timed_wait_waits_at_least_timeout(&test.monotonic_named_sut1);
    timed_wait_waits_at_least_timeout(&test.monotonic_named_sut2);

    timed_wait_waits_at_least_timeout(&test.realtime_named_sut1);
    timed_wait_waits_at_least_timeout(&test.realtime_named_sut2);
}

#[test]
pub fn unnamed_semaphore_timed_wait_waits_at_least_timeout() {
    let handle1 = UnnamedSemaphoreHandle::new();
    let handle2 = UnnamedSemaphoreHandle::new();

    let test = UnnamedSemaphoreTest::new(&handle1, &handle2);

    timed_wait_waits_at_least_timeout(&test.monotonic_unnamed_sut);
    timed_wait_waits_at_least_timeout(&test.realtime_unnamed_sut);
}

#[test]
pub fn unnamed_semaphore_multiple_ipc_semaphores_are_working() {
    let handle = UnnamedSemaphoreHandle::new();
    let sut1 = UnnamedSemaphoreBuilder::new()
        .is_interprocess_capable(true)
        .create(&handle)
        .unwrap();

    let sut2 = unsafe { UnnamedSemaphore::from_ipc_handle(&handle) };

    assert_that!(sut1.post(), is_ok);
    assert_that!(sut2.try_wait().unwrap(), eq true);
    assert_that!(!sut2.try_wait().unwrap(), eq true);

    assert_that!(sut2.post(), is_ok);
    assert_that!(sut1.try_wait().unwrap(), eq true);
    assert_that!(!sut1.try_wait().unwrap(), eq true);
}

#[test]
#[requires_std("panics")]
#[should_panic]
#[cfg(debug_assertions)]
pub fn unnamed_semaphore_acquire_uninitialized_ipc_handle_failes() {
    let handle = UnnamedSemaphoreHandle::new();

    unsafe { UnnamedSemaphore::from_ipc_handle(&handle) };
}

#[test]
#[requires_std("panics")]
#[should_panic]
#[cfg(debug_assertions)]
pub fn unnamed_semaphore_acquiring_non_ipc_capable_handle_fails() {
    let handle = UnnamedSemaphoreHandle::new();
    let _sut1 = UnnamedSemaphoreBuilder::new()
        .is_interprocess_capable(false)
        .create(&handle);

    unsafe { UnnamedSemaphore::from_ipc_handle(&handle) };
}
