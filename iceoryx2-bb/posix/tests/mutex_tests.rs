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

use iceoryx2_bb_posix::clock::*;
use iceoryx2_bb_posix::mutex::*;
use iceoryx2_bb_posix::system_configuration::Feature;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing::watchdog::Watchdog;

use core::time::Duration;

extern crate alloc;
use alloc::sync::Arc;

use std::sync::Barrier;
use std::thread;

const TIMEOUT: Duration = Duration::from_millis(100);

#[test]
fn mutex_lock_works() {
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
fn mutex_try_lock_works() {
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
fn mutex_try_lock_leads_to_blocked_mutex() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new().create(111, &handle).unwrap();
    let mut value = sut.try_lock().unwrap().unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut value = sut.lock().unwrap();
            assert_that!(*value, eq 444);
            *value = 555;
        });

        thread::sleep(core::time::Duration::from_millis(10));
        let value_old = *value;
        *value = 444;
        drop(value);

        t1.join().unwrap();
        assert_that!(value_old, eq 111);
        assert_that!(*sut.lock().unwrap(), eq 555);
    });
}

#[test]
fn mutex_timed_lock_leads_to_blocked_mutex_realtime() {
    let handle = MutexHandle::<i32>::new();
    let sut = Arc::new(
        MutexBuilder::new()
            .clock_type(ClockType::Realtime)
            .create(111, &handle)
            .unwrap(),
    );
    let mut value = sut.timed_lock(Duration::from_millis(100)).unwrap().unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut value = sut.lock().unwrap();
            *value = 555;
        });

        thread::sleep(core::time::Duration::from_millis(10));
        let value_old = *value;
        *value = 444;
        drop(value);

        t1.join().unwrap();
        assert_that!(value_old, eq 111);
        assert_that!(*sut.lock().unwrap(), eq 555);
    });
}

#[test]
fn mutex_timed_lock_leads_to_blocked_mutex_monotonic() {
    test_requires!(Feature::MonotonicClock.is_available());

    let handle = MutexHandle::<i32>::new();
    let sut = Arc::new(
        MutexBuilder::new()
            .clock_type(ClockType::Monotonic)
            .create(111, &handle)
            .unwrap(),
    );
    let mut value = sut.timed_lock(Duration::from_millis(100)).unwrap().unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut value = sut.lock().unwrap();
            *value = 555;
        });

        thread::sleep(core::time::Duration::from_millis(10));
        let value_old = *value;
        *value = 444;
        drop(value);

        t1.join().unwrap();
        assert_that!(value_old, eq 111);
        assert_that!(*sut.lock().unwrap(), eq 555);
    });
}

#[test]
fn mutex_try_lock_fails_when_already_locked() {
    let handle = MutexHandle::<i32>::new();
    let sut = Arc::new(MutexBuilder::new().create(111, &handle).unwrap());
    let value = sut.lock().unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let value = sut.try_lock().unwrap();
            assert_that!(value, is_none);
        });

        t1.join().unwrap();
        drop(value);
    });
}

#[test]
fn mutex_timed_lock_blocks_at_least_for_timeout_realtime_clock() {
    let barrier = Barrier::new(2);
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .clock_type(ClockType::Realtime)
        .create(111, &handle)
        .unwrap();

    thread::scope(|s| {
        s.spawn(|| {
            barrier.wait();
            let _guard = sut.lock().unwrap();
            nanosleep(TIMEOUT * 4).unwrap();
        });

        barrier.wait();
        nanosleep(TIMEOUT).unwrap();
        let start = Time::now_with_clock(ClockType::Realtime).unwrap();
        let value = sut.timed_lock(TIMEOUT).unwrap();

        assert_that!(value, is_none);
        assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
    });
}

#[test]
fn mutex_timed_lock_blocks_at_least_for_timeout_monotonic_clock() {
    test_requires!(Feature::MonotonicClock.is_available());

    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .clock_type(ClockType::Monotonic)
        .create(111, &handle)
        .unwrap();

    thread::scope(|s| {
        s.spawn(|| {
            let _guard = sut.lock().unwrap();
            nanosleep(TIMEOUT * 4).unwrap();
        });

        nanosleep(TIMEOUT).unwrap();
        let start = Time::now_with_clock(ClockType::Monotonic).unwrap();
        let value = sut.timed_lock(TIMEOUT).unwrap();

        assert_that!(value, is_none);
        assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
    });
}

#[test]
fn mutex_multiple_ipc_mutex_are_working() {
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
fn mutex_recursive_mutex_can_be_locked_multiple_times_by_same_thread() {
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
fn mutex_recursive_does_not_unlock_in_the_first_unlock_call() {
    let _watchdog = Watchdog::new();
    let handle = MutexHandle::new();
    let sut = MutexBuilder::new()
        .mutex_type(MutexType::Recursive)
        .create(5123, &handle)
        .unwrap();

    let barrier = Barrier::new(2);
    std::thread::scope(|s| {
        let guard1 = sut.try_lock().unwrap();
        assert_that!(guard1, is_some);
        let guard2 = sut.try_lock().unwrap();
        assert_that!(guard2, is_some);

        drop(guard1);
        s.spawn(|| {
            let guard = sut.try_lock().unwrap();
            assert_that!(guard, is_none);
            barrier.wait();
        });

        barrier.wait();
        drop(guard2);

        s.spawn(|| {
            let guard = sut.try_lock().unwrap();
            assert_that!(guard, is_some);
        });
    });
}

#[test]
fn mutex_deadlock_detection_works() {
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
fn mutex_recursive_mutex_blocks() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .mutex_type(MutexType::Recursive)
        .create(111, &handle)
        .unwrap();
    let mut value = sut.try_lock().unwrap().unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut value = sut.lock().unwrap();
            assert_that!(*value, eq 444);
            *value = 555;
        });

        thread::sleep(core::time::Duration::from_millis(10));
        let old_value = *value;
        *value = 444;
        drop(value);

        t1.join().unwrap();
        assert_that!(old_value, eq 111);
        assert_that!(*sut.lock().unwrap(), eq 555);
    });
}

#[test]
fn mutex_with_deadlock_detection_blocks() {
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .mutex_type(MutexType::WithDeadlockDetection)
        .create(111, &handle)
        .unwrap();
    let mut value = sut.try_lock().unwrap().unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut value = sut.lock().unwrap();
            assert_that!(*value, eq 444);
            *value = 555;
        });

        thread::sleep(core::time::Duration::from_millis(10));
        let old_value = *value;
        *value = 444;
        drop(value);

        t1.join().unwrap();
        assert_that!(old_value, eq 111);
        assert_that!(*sut.lock().unwrap(), eq 555);
    });
}

// This test fails on QNX due to the mutex created in the separate thread being cleaned up
// before the clean-up code is executed. Needs investigation (#978).
#[cfg(not(target_os = "nto"))]
#[test]
fn mutex_can_be_recovered_when_thread_died() {
    let _watchdog = Watchdog::new();
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .thread_termination_behavior(MutexThreadTerminationBehavior::ReleaseWhenLocked)
        .create(111, &handle)
        .unwrap();

    thread::scope(|s| {
        s.spawn(|| {
            let guard = sut.lock();
            assert_that!(guard, is_ok);
            core::mem::forget(guard);
        });
    });

    loop {
        let guard = sut.try_lock();

        if guard.is_ok() {
            assert_that!(guard.as_ref().unwrap(), is_none);
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

// This test fails on QNX due to the mutex created in the separate thread from being cleaned up
// before the clean-up code is added. Needs investigation.
#[test]
#[cfg(not(any(target_os = "macos", target_os = "nto")))]
fn mutex_in_unrecoverable_state_if_state_of_leaked_mutex_is_not_repaired() {
    iceoryx2_bb_log::set_log_level(iceoryx2_bb_log::LogLevel::Trace);
    let _watchdog = Watchdog::new();
    let handle = MutexHandle::<i32>::new();
    let sut = MutexBuilder::new()
        .thread_termination_behavior(MutexThreadTerminationBehavior::ReleaseWhenLocked)
        .mutex_type(MutexType::Normal)
        .create(111, &handle)
        .unwrap();

    thread::scope(|s| {
        let t = s.spawn(|| {
            let guard = sut.lock();
            core::mem::forget(guard);
        });
        let _ = t.join();
    });

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
