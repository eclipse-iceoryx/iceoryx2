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

use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::*;
use iceoryx2_bb_posix::mutex::*;
use iceoryx2_bb_posix::system_configuration::Feature;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

const TIMEOUT: Duration = Duration::from_millis(100);

#[test]
pub fn lock_works() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new().create(123, &handle).unwrap();
    {
        let mut value = sut.lock().unwrap();
        assert_that!(*value, eq 123);
        *value = 456;
    }

    let value = sut.lock().unwrap();
    assert_that!(*value, eq 456);
}

#[test]
pub fn try_lock_works() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new().create(789, &handle).unwrap();
    {
        let mut value = sut.try_lock().unwrap().unwrap();
        assert_that!(*value, eq 789);
        *value = 321;
    }

    let value = sut.try_lock().unwrap().unwrap();
    assert_that!(*value, eq 321);
}

#[test]
pub fn try_lock_leads_to_blocked_mutex() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new().create(111, &handle).unwrap();
    let mut value = sut.try_lock().unwrap().unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut value = sut.lock().unwrap();
                assert_that!(*value, eq 444);
                *value = 555;
            })
            .expect("failed to spawn thread");

        nanosleep(Duration::from_millis(10)).expect("failed to sleep");
        let value_old = *value;
        *value = 444;
        drop(value);

        assert_that!(value_old, eq 111);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(*sut.lock().unwrap(), eq 555);
}

#[test]
pub fn timed_lock_leads_to_blocked_mutex_realtime() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .clock_type(ClockType::Realtime)
        .create(111, &handle)
        .unwrap();
    let mut value = sut.timed_lock(Duration::from_millis(100)).unwrap().unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut value = sut.lock().unwrap();
                *value = 555;
            })
            .expect("failed to spawn thread");

        nanosleep(Duration::from_millis(10)).expect("failed to sleep");
        let value_old = *value;
        *value = 444;
        drop(value);

        assert_that!(value_old, eq 111);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(*sut.lock().unwrap(), eq 555);
}

#[test]
pub fn timed_lock_leads_to_blocked_mutex_monotonic() {
    test_requires!(Feature::MonotonicClock.is_available());

    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .clock_type(ClockType::Monotonic)
        .create(111, &handle)
        .unwrap();
    let mut value = sut.timed_lock(Duration::from_millis(100)).unwrap().unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut value = sut.lock().unwrap();
                *value = 555;
            })
            .expect("failed to spawn thread");

        nanosleep(Duration::from_millis(10)).expect("failed to sleep");
        let value_old = *value;
        *value = 444;
        drop(value);

        assert_that!(value_old, eq 111);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(*sut.lock().unwrap(), eq 555);
}

#[test]
pub fn try_lock_fails_when_already_locked() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new().create(111, &handle).unwrap();
    let value = sut.lock().unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let value = sut.try_lock().unwrap();
                assert_that!(value, is_none);
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to execute thread scope");

    drop(value);
}

#[test]
pub fn timed_lock_blocks_at_least_for_timeout_realtime_clock() {
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .clock_type(ClockType::Realtime)
        .create(111, &handle)
        .unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                barrier.wait();
                let _guard = sut.lock().unwrap();
                nanosleep(TIMEOUT * 4).expect("failed to sleep");
            })
            .expect("failed to spawn thread");

        barrier.wait();
        nanosleep(TIMEOUT).expect("failed to sleep");
        let start = Time::now_with_clock(ClockType::Realtime).unwrap();
        let value = sut.timed_lock(TIMEOUT).unwrap();

        assert_that!(value, is_none);
        assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);

        Ok(())
    })
    .expect("failed to execute thread scope");
}

#[test]
pub fn timed_lock_blocks_at_least_for_timeout_monotonic_clock() {
    test_requires!(Feature::MonotonicClock.is_available());

    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .clock_type(ClockType::Monotonic)
        .create(111, &handle)
        .unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let _guard = sut.lock().unwrap();
                nanosleep(TIMEOUT * 4).expect("failed to sleep");
            })
            .expect("failed to spawn thread");

        nanosleep(TIMEOUT).expect("failed to sleep");
        let start = Time::now_with_clock(ClockType::Monotonic).unwrap();
        let value = sut.timed_lock(TIMEOUT).unwrap();

        assert_that!(value, is_none);
        assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);

        Ok(())
    })
    .expect("failed to execute thread scope");
}

#[test]
pub fn multiple_ipc_mutex_are_working() {
    let handle = MutexHandle::new();
    let sut1 = MutexBuilder::new()
        .is_interprocess_capable(true)
        .mutex_type(MutexType::Normal)
        .create(123, &handle)
        .unwrap();

    let sut2 = unsafe { Mutex::from_ipc_handle(&handle) };

    let guard1 = sut1.try_lock().unwrap();
    assert_that!(guard1, is_some);

    let guard2 = sut2.try_lock().unwrap();
    assert_that!(guard2, is_none);
    drop(guard1);

    let guard2 = sut2.try_lock().unwrap();
    assert_that!(guard2, is_some);

    let guard1 = sut1.try_lock().unwrap();
    assert_that!(guard1, is_none);
}

#[test]
pub fn recursive_mutex_can_be_locked_multiple_times_by_same_thread() {
    let handle = MutexHandle::new();
    let sut = MutexBuilder::new()
        .mutex_type(MutexType::Recursive)
        .create(5123, &handle)
        .unwrap();

    let guard1 = sut.try_lock().unwrap();
    assert_that!(guard1, is_some);
    let guard2 = sut.try_lock().unwrap();
    assert_that!(guard2, is_some);

    drop(guard1);
    drop(guard2);

    let guard1 = sut.lock();
    assert_that!(guard1, is_ok);
    let guard2 = sut.lock();
    assert_that!(guard2, is_ok);

    drop(guard1);
    drop(guard2);

    let guard1 = sut.timed_lock(TIMEOUT).unwrap();
    assert_that!(guard1, is_some);
    let guard2 = sut.timed_lock(TIMEOUT).unwrap();
    assert_that!(guard2, is_some);
}

#[test]
pub fn recursive_does_not_unlock_in_the_first_unlock_call() {
    let _watchdog = Watchdog::new();
    let handle = MutexHandle::new();
    let sut = MutexBuilder::new()
        .mutex_type(MutexType::Recursive)
        .create(5123, &handle)
        .unwrap();

    const NUMBER_OF_THREADS: u32 = 1;
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(NUMBER_OF_THREADS + 1)
        .create(&barrier_handle)
        .unwrap();

    thread_scope(|s| {
        let guard1 = sut.try_lock().unwrap();
        assert_that!(guard1, is_some);
        let guard2 = sut.try_lock().unwrap();
        assert_that!(guard2, is_some);

        drop(guard1);
        s.thread_builder()
            .spawn(|| {
                let guard = sut.try_lock().unwrap();
                assert_that!(guard, is_none);
                barrier.wait();
            })
            .expect("failed to spawn thread");

        barrier.wait();
        drop(guard2);

        s.thread_builder()
            .spawn(|| {
                let guard = sut.try_lock().unwrap();
                assert_that!(guard, is_some);
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to execute thread scope");
}

#[test]
pub fn deadlock_detection_works() {
    for clock_type in ClockType::all_supported_clocks() {
        let handle = MutexHandle::new();
        let sut = MutexBuilder::new()
            .mutex_type(MutexType::WithDeadlockDetection)
            .clock_type(*clock_type)
            .create(5123, &handle)
            .unwrap();

        let guard = sut.try_lock().unwrap();
        assert_that!(guard, is_some);
        let result = sut.try_lock().unwrap();
        assert_that!(result, is_none);

        drop(guard);

        let guard = sut.lock();
        assert_that!(guard, is_ok);
        let result = sut.lock();
        assert_that!(result, is_err);
        assert_that!(result.err().unwrap(), eq MutexLockError::DeadlockDetected);

        drop(guard);

        let guard = sut.timed_lock(TIMEOUT).unwrap();
        assert_that!(guard, is_some);
        let result = sut.timed_lock(TIMEOUT).unwrap();
        assert_that!(result, is_none);

        drop(guard);
    }
}

#[test]
pub fn recursive_mutex_blocks() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .mutex_type(MutexType::Recursive)
        .create(111, &handle)
        .unwrap();
    let mut value = sut.try_lock().unwrap().unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut value = sut.lock().unwrap();
                assert_that!(*value, eq 444);
                *value = 555;
            })
            .expect("failed to spawn thread");

        nanosleep(Duration::from_millis(10)).expect("failed to sleep");
        let old_value = *value;
        *value = 444;
        drop(value);

        assert_that!(old_value, eq 111);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(*sut.lock().unwrap(), eq 555);
}

#[test]
pub fn deadlock_detection_blocks() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .mutex_type(MutexType::WithDeadlockDetection)
        .create(111, &handle)
        .unwrap();
    let mut value = sut.try_lock().unwrap().unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut value = sut.lock().unwrap();
                assert_that!(*value, eq 444);
                *value = 555;
            })
            .expect("failed to spawn thread");

        nanosleep(Duration::from_millis(10)).expect("failed to sleep");
        let old_value = *value;
        *value = 444;
        drop(value);

        assert_that!(old_value, eq 111);

        Ok(())
    })
    .expect("failed to execute thread scope");

    assert_that!(*sut.lock().unwrap(), eq 555);
}

#[test]
#[cfg(not(target_os = "nto"))]
pub fn can_be_recovered_when_thread_died() {
    let _watchdog = Watchdog::new();
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .thread_termination_behavior(MutexThreadTerminationBehavior::ReleaseWhenLocked)
        .create(111, &handle)
        .unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let guard = sut.lock();
                assert_that!(guard, is_ok);
                core::mem::forget(guard);
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to execute thread scope");

    loop {
        let guard = sut.try_lock();

        if let Ok(guard) = &guard {
            assert_that!(guard, is_none);
        } else if let Err(MutexLockError::LockAcquiredButOwnerDied(_)) = guard {
            sut.make_consistent();
            break;
        }
    }

    let guard = sut.try_lock();
    assert_that!(guard, is_ok);
    assert_that!(guard.as_ref().unwrap(), is_some);
    drop(guard);

    let guard = sut.lock();
    assert_that!(guard, is_ok);
    drop(guard);

    let guard = sut.timed_lock(TIMEOUT);
    assert_that!(guard, is_ok);
    assert_that!(guard.as_ref().unwrap(), is_some);
    drop(guard);
}

#[test]
#[cfg(not(any(target_os = "macos", target_os = "nto")))]
pub fn mutex_in_unrecoverable_state_if_state_of_leaked_mutex_is_not_repaired() {
    let _watchdog = Watchdog::new();
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .thread_termination_behavior(MutexThreadTerminationBehavior::ReleaseWhenLocked)
        .mutex_type(MutexType::Normal)
        .create(111, &handle)
        .unwrap();

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let guard = sut.lock();
                core::mem::forget(guard);
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to execute thread scope");

    let guard = sut.lock();
    assert_that!(guard, is_err);
    match guard.as_ref().err().as_ref().unwrap() {
        MutexLockError::LockAcquiredButOwnerDied(_) => (),
        _ => assert_that!(true, eq false),
    }
    drop(guard);

    let guard = sut.lock();
    assert_that!(guard, is_err);
    assert_that!(guard.err().unwrap(), eq MutexLockError::UnrecoverableState);

    let guard = sut.try_lock();
    assert_that!(guard, is_err);
    assert_that!(guard.err().unwrap(), eq MutexLockError::UnrecoverableState);

    #[cfg(not(target_os = "linux"))]
    {
        let guard = sut.timed_lock(TIMEOUT);
        assert_that!(guard, is_err);
        assert_that!(
            guard.err().unwrap(), eq
            MutexTimedLockError::MutexLockError(MutexLockError::UnrecoverableState)
        );
    }
}
