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

use iceoryx2_bb_posix::condition_variable::*;
use iceoryx2_bb_testing::assert_that;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

static TIMEOUT: Duration = Duration::from_millis(10);

#[test]
fn multi_condition_variable_construction_works() {
    let handle = MutexHandle::<i32>::new();
    let sut = ConditionVariableBuilder::new()
        .create_multi_condition_variable(1234, &handle)
        .unwrap();
    assert_that!(*sut.lock().unwrap(), eq 1234);
}

#[test]
fn multi_condition_variable_notify_all_modifies_value() {
    let handle = MutexHandle::<i32>::new();
    let sut = ConditionVariableBuilder::new()
        .create_multi_condition_variable(4321, &handle)
        .unwrap();
    *sut.notify_all().unwrap() = 987;
    assert_that!(*sut.lock().unwrap(), eq 987);
}

#[test]
fn multi_condition_variable_wait_while_is_signalled_by_notify_all() {
    let handle = MutexHandle::<i32>::new();
    let sut = ConditionVariableBuilder::new()
        .create_multi_condition_variable(1111, &handle)
        .unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let guard = sut.wait_while(|t| *t == 4456).unwrap();
            assert_that!(*guard, eq 4456);
        });

        thread::sleep(TIMEOUT);

        {
            let mut guard = sut.notify_all().unwrap();
            *guard = 2222;
        }

        {
            let mut guard = sut.notify_all().unwrap();
            *guard = 4456;
        }

        t1.join().unwrap();
    });
}

#[test]
fn multi_condition_variable_wait_while_is_signalled_by_modify_notify_all() {
    let handle = MutexHandle::<i32>::new();
    let sut = ConditionVariableBuilder::new()
        .create_multi_condition_variable(1111, &handle)
        .unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let guard = sut.wait_while(|t| *t == 4456).unwrap();
            assert_that!(*guard, eq 4456);
        });

        thread::sleep(TIMEOUT);

        sut.modify_notify_all(|value| *value = 2222).unwrap();
        sut.modify_notify_all(|value| *value = 4456).unwrap();

        t1.join().unwrap();
    });
}

#[test]
fn multi_condition_variable_trigger_all_signals_all() {
    let handle = MutexHandle::<i32>::new();
    let sut = ConditionVariableBuilder::new()
        .create_multi_condition_variable(0, &handle)
        .unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let mut guard = sut.wait().unwrap();
            *guard += 1;
        });

        let t2 = s.spawn(|| {
            let mut guard = sut.wait().unwrap();
            *guard += 1;
        });

        thread::sleep(TIMEOUT);

        sut.trigger_all();

        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(*sut.lock().unwrap(), eq 2);
    });
}

#[test]
fn multi_condition_variable_timed_wait_waits_at_least_given_amount_of_time() {
    let handle = MutexHandle::<i32>::new();
    let sut = ConditionVariableBuilder::new()
        .create_multi_condition_variable(1111, &handle)
        .unwrap();

    let start = Instant::now();
    sut.timed_wait(TIMEOUT).unwrap();
    assert_that!(start.elapsed(), time_at_least TIMEOUT);
}

#[test]
fn multi_condition_variable_timed_wait_while_waits_at_least_given_amount_of_time() {
    let handle = MutexHandle::<i32>::new();
    let sut = ConditionVariableBuilder::new()
        .create_multi_condition_variable(0, &handle)
        .unwrap();

    thread::scope(|s| {
        let t1 = s.spawn(|| {
            let start = Instant::now();
            sut.timed_wait_while(2 * TIMEOUT, |t| *t > 0).unwrap();
            assert_that!(start.elapsed(), time_at_least 2 * TIMEOUT);
        });

        thread::sleep(TIMEOUT);
        sut.trigger_all();

        t1.join().unwrap();
    });
}

#[test]
fn condition_variable_construction_works() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    let sut = ConditionVariableBuilder::new()
        .create_condition_variable(1234, |v| *v > 1000, &handle)
        .unwrap();
    assert_that!(sut.lock().unwrap().value, eq 1234);
}

#[test]
fn condition_variable_trigger_all_signals_all_waiters() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    thread::scope(|s| {
        let counter = Arc::new(AtomicI32::new(0));
        let sut = Arc::new(
            ConditionVariableBuilder::new()
                .create_condition_variable(500, |v| *v > 1000, &handle)
                .unwrap(),
        );

        let sut_thread1 = Arc::clone(&sut);
        let counter_thread1 = Arc::clone(&counter);
        let t1 = s.spawn(move || {
            sut_thread1.wait().unwrap();
            counter_thread1.fetch_add(1, Ordering::Relaxed);
        });

        let sut_thread2 = Arc::clone(&sut);
        let counter_thread2 = Arc::clone(&counter);
        let t2 = s.spawn(move || {
            sut_thread2.wait().unwrap();
            counter_thread2.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        sut.trigger_all();

        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn condition_variable_trigger_one_signals_one_waiter() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    thread::scope(|s| {
        let counter = Arc::new(AtomicI32::new(0));
        let sut = Arc::new(
            ConditionVariableBuilder::new()
                .create_condition_variable(500, |v| *v > 1000, &handle)
                .unwrap(),
        );

        let sut_thread1 = Arc::clone(&sut);
        let counter_thread1 = Arc::clone(&counter);
        let t1 = s.spawn(move || {
            sut_thread1.wait().unwrap();
            counter_thread1.fetch_add(1, Ordering::Relaxed);
        });

        let sut_thread2 = Arc::clone(&sut);
        let counter_thread2 = Arc::clone(&counter);
        let t2 = s.spawn(move || {
            sut_thread2.wait().unwrap();
            counter_thread2.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(TIMEOUT);
        let counter_old_1 = counter.load(Ordering::Relaxed);
        sut.trigger_one();
        thread::sleep(TIMEOUT);
        let counter_old_2 = counter.load(Ordering::Relaxed);
        sut.trigger_one();

        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(counter_old_1, eq 0);
        assert_that!(counter_old_2, eq 1);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn condition_variable_notify_all_signals_all_waiters() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    thread::scope(|s| {
        let counter = Arc::new(AtomicI32::new(0));
        let sut = Arc::new(
            ConditionVariableBuilder::new()
                .create_condition_variable(500, |v| *v > 1000, &handle)
                .unwrap(),
        );

        let sut_thread1 = Arc::clone(&sut);
        let counter_thread1 = Arc::clone(&counter);
        let t1 = s.spawn(move || {
            sut_thread1.wait_while().unwrap();
            counter_thread1.fetch_add(1, Ordering::Relaxed);
        });

        let sut_thread2 = Arc::clone(&sut);
        let counter_thread2 = Arc::clone(&counter);
        let t2 = s.spawn(move || {
            sut_thread2.wait_while().unwrap();
            counter_thread2.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(TIMEOUT);
        let counter_old_1 = counter.load(Ordering::Relaxed);
        let mut guard = sut.notify_all().unwrap();
        *guard = 750;
        drop(guard);

        thread::sleep(TIMEOUT);
        let counter_old_2 = counter.load(Ordering::Relaxed);
        let mut guard = sut.notify_all().unwrap();
        *guard = 1500;
        drop(guard);

        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(counter_old_1, eq 0);
        assert_that!(counter_old_2, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn condition_variable_notify_one_signals_one_waiter() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    thread::scope(|s| {
        let counter = Arc::new(AtomicI32::new(0));
        let sut = Arc::new(
            ConditionVariableBuilder::new()
                .create_condition_variable(500, |v| *v > 1000, &handle)
                .unwrap(),
        );

        let sut_thread1 = Arc::clone(&sut);
        let counter_thread1 = Arc::clone(&counter);
        let t1 = s.spawn(move || {
            sut_thread1.wait_while().unwrap();
            counter_thread1.fetch_add(1, Ordering::Relaxed);
        });

        let sut_thread2 = Arc::clone(&sut);
        let counter_thread2 = Arc::clone(&counter);
        let t2 = s.spawn(move || {
            sut_thread2.wait_while().unwrap();
            counter_thread2.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(TIMEOUT);
        let counter_old_1 = counter.load(Ordering::Relaxed);
        let mut guard = sut.notify_one().unwrap();
        *guard = 1750;
        drop(guard);

        thread::sleep(TIMEOUT);
        let counter_old_2 = counter.load(Ordering::Relaxed);
        let mut guard = sut.notify_one().unwrap();
        *guard = 1500;
        drop(guard);

        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(counter_old_1, eq 0);
        assert_that!(counter_old_2, eq 1);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn condition_variable_modify_notify_all_signals_all_waiters() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    thread::scope(|s| {
        let counter = Arc::new(AtomicI32::new(0));
        let sut = Arc::new(
            ConditionVariableBuilder::new()
                .create_condition_variable(500, |v| *v > 1000, &handle)
                .unwrap(),
        );

        let sut_thread1 = Arc::clone(&sut);
        let counter_thread1 = Arc::clone(&counter);
        let t1 = s.spawn(move || {
            sut_thread1.timed_wait_while(TIMEOUT * 10).unwrap();
            counter_thread1.fetch_add(1, Ordering::Relaxed);
        });

        let sut_thread2 = Arc::clone(&sut);
        let counter_thread2 = Arc::clone(&counter);
        let t2 = s.spawn(move || {
            sut_thread2.timed_wait_while(TIMEOUT * 10).unwrap();
            counter_thread2.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(TIMEOUT);
        let counter_old_1 = counter.load(Ordering::Relaxed);
        sut.modify_notify_all(|value| *value = 2213).unwrap();
        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(counter_old_1, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn condition_variable_modify_notify_one_signals_one_waiter() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    thread::scope(|s| {
        let counter = Arc::new(AtomicI32::new(0));
        let sut = Arc::new(
            ConditionVariableBuilder::new()
                .create_condition_variable(500, |v| *v > 1000, &handle)
                .unwrap(),
        );

        let sut_thread1 = Arc::clone(&sut);
        let counter_thread1 = Arc::clone(&counter);
        let t1 = s.spawn(move || {
            sut_thread1.timed_wait_while(TIMEOUT * 10).unwrap();
            counter_thread1.fetch_add(1, Ordering::Relaxed);
        });

        let sut_thread2 = Arc::clone(&sut);
        let counter_thread2 = Arc::clone(&counter);
        let t2 = s.spawn(move || {
            sut_thread2.timed_wait_while(TIMEOUT * 10).unwrap();
            counter_thread2.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(TIMEOUT);
        let counter_old_1 = counter.load(Ordering::Relaxed);
        sut.modify_notify_one(|value| *value = 2213).unwrap();

        thread::sleep(TIMEOUT);
        let counter_old_2 = counter.load(Ordering::Relaxed);
        sut.modify_notify_one(|value| *value = 2213).unwrap();

        t1.join().unwrap();
        t2.join().unwrap();

        assert_that!(counter_old_1, eq 0);
        assert_that!(counter_old_2, eq 1);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn condition_variable_timed_wait_waits_at_least_given_amount_of_time() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    let sut = Arc::new(
        ConditionVariableBuilder::new()
            .create_condition_variable(1111, |value| *value > 20000, &handle)
            .unwrap(),
    );

    let start = Instant::now();
    sut.timed_wait(TIMEOUT).unwrap();
    assert_that!(start.elapsed(), time_at_least TIMEOUT);
}

#[test]
fn condition_variable_timed_wait_while_waits_at_least_given_amount_of_time() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    let sut = Arc::new(
        ConditionVariableBuilder::new()
            .create_condition_variable(1111, |value| *value > 20000, &handle)
            .unwrap(),
    );

    let start = Instant::now();
    sut.timed_wait_while(TIMEOUT).unwrap();
    assert_that!(start.elapsed(), time_at_least TIMEOUT);
}

#[test]
fn condition_variable_timed_wait_while_does_not_wait_when_predicate_is_fulfilled() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    let sut = Arc::new(
        ConditionVariableBuilder::new()
            .create_condition_variable(111, |value| *value > 20000, &handle)
            .unwrap(),
    );
    sut.lock().unwrap().value = 9999999;

    let start = Instant::now();
    assert_that!(sut.timed_wait_while(TIMEOUT).unwrap(), is_some);
    assert_that!(start.elapsed(), lt TIMEOUT);
}

#[test]
fn condition_variable_wait_while_does_not_wait_when_predicate_is_fulfilled() {
    let handle = MutexHandle::<ConditionVariableData<i32>>::new();
    let sut = Arc::new(
        ConditionVariableBuilder::new()
            .create_condition_variable(111, |value| *value > 20000, &handle)
            .unwrap(),
    );
    sut.lock().unwrap().value = 9999999;

    assert_that!(sut.wait_while(), is_ok);
}
