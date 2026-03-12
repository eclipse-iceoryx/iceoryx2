// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use alloc::vec::Vec;

use iceoryx2_bb_concurrency::spin_lock::SpinLock;
use iceoryx2_bb_posix::barrier::{BarrierBuilder, BarrierHandle, Handle};
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
use iceoryx2_bb_testing::watchdog::Watchdog;

pub fn try_lock_locks() {
    let lk = SpinLock::new(0);
    let guard = lk.try_lock();
    assert_that!(guard, is_some);
    assert_that!(lk.try_lock(), is_none);
}

pub fn lock_guard_unlocks_when_dropped() {
    let lk = SpinLock::new(0);
    let guard = lk.try_lock();
    assert_that!(guard, is_some);
    assert_that!(lk.try_lock(), is_none);
    drop(guard);
    assert_that!(lk.try_lock(), is_some);
}

pub fn lock_guard_behaves_like_reference() {
    let lk = SpinLock::new(0);
    let mut guard = lk.try_lock().unwrap();
    *guard = 1984;
    assert_that!(*guard, eq 1984);
}

pub fn blocking_lock_locks_exclusively() {
    const NUMBER_OF_THREADS: u32 = 2;

    let _watchdog = Watchdog::new();
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(NUMBER_OF_THREADS)
        .create(&barrier_handle)
        .unwrap();

    let lk = SpinLock::new(0);
    thread_scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    let mut guard = lk.blocking_lock();
                    *guard += 1;
                })
                .expect("failed to spawn thread");
        }

        Ok(())
    })
    .expect("failed to spawn thread");

    let guard = lk.blocking_lock();
    assert_that!(*guard, eq NUMBER_OF_THREADS);
}

pub fn try_lock_locks_exclusively() {
    const NUMBER_OF_THREADS: u32 = 2;

    let _watchdog = Watchdog::new();
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(NUMBER_OF_THREADS)
        .create(&barrier_handle)
        .unwrap();

    let lk = SpinLock::new(0);
    thread_scope(|s| {
        for _n in 0..NUMBER_OF_THREADS {
            s.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    let guard = lk.try_lock();
                    if let Some(mut guard) = guard {
                        *guard += 1;
                    }
                })
                .expect("failed to spawn thread");
        }
        Ok(())
    })
    .expect("failed to spawn thread");

    let guard = lk.blocking_lock();
    assert_that!(*guard, gt 0);
    assert_that!(*guard, le NUMBER_OF_THREADS);
}

pub fn lock_is_hold_until_guard_drops() {
    let _watchdog = Watchdog::new();
    let lk = SpinLock::new(Vec::new());
    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| lk.blocking_lock().push(1990))
            .expect("failed to spawn thread");
        s.thread_builder()
            .spawn(|| {
                let mut guard = lk.blocking_lock();
                guard.push(4);
                guard.push(6);
            })
            .expect("failed to spawn thread");

        Ok(())
    })
    .expect("failed to spawn thread");

    let guard = lk.blocking_lock();
    assert_that!(guard.as_slice(), any_of [[1990, 4, 6], [4, 6, 1990]]);
}

pub fn drop_called_for_underlying_value() {
    let state = LifetimeTracker::start_tracking();
    let lk = SpinLock::<LifetimeTracker>::new(LifetimeTracker::default());
    assert_that!(state.number_of_living_instances(), eq 1);
    drop(lk);
    assert_that!(state.number_of_living_instances(), eq 0);
}
