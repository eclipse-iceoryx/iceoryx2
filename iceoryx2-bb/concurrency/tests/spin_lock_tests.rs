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

use core::alloc::Layout;
use iceoryx2_bb_concurrency::spin_lock::SpinLock;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_bb_testing::lifetime_tracker::LifetimeTracker;
use iceoryx2_bb_testing::{assert_that, watchdog::Watchdog};
use std::alloc::{alloc, dealloc};
use std::sync::Barrier;
use std::thread;

#[test]
fn try_lock_locks() {
    let lk = SpinLock::new(0);
    let guard = lk.try_lock();
    assert_that!(guard, is_some);
    assert_that!(lk.try_lock(), is_none);
}

#[test]
fn lock_guard_unlocks_when_dropped() {
    let lk = SpinLock::new(0);
    let guard = lk.try_lock();
    assert_that!(guard, is_some);
    assert_that!(lk.try_lock(), is_none);
    drop(guard);
    assert_that!(lk.try_lock(), is_some);
}

#[test]
fn lock_guard_behaves_like_reference() {
    let lk = SpinLock::new(0);
    let mut guard = lk.try_lock().unwrap();
    *guard = 1984;
    assert_that!(*guard, eq 1984);
}

#[test]
fn blocking_lock_locks_exclusively() {
    const NUMBER_OF_THREADS: usize = 2;

    let _watchdog = Watchdog::new();
    let barrier = Barrier::new(NUMBER_OF_THREADS);
    let lk = SpinLock::new(0);
    thread::scope(|s| {
        for _ in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait();
                let mut guard = lk.blocking_lock();
                *guard += 1;
            });
        }
    });
    let guard = lk.blocking_lock();
    assert_that!(*guard, eq NUMBER_OF_THREADS);
}

#[test]
fn try_lock_locks_exclusively() {
    const NUMBER_OF_THREADS: usize = 2;

    let _watchdog = Watchdog::new();
    let barrier = Barrier::new(NUMBER_OF_THREADS);
    let lk = SpinLock::new(0);
    thread::scope(|s| {
        for _n in 0..NUMBER_OF_THREADS {
            s.spawn(|| {
                barrier.wait();
                let guard = lk.try_lock();
                if guard.is_some() {
                    *guard.unwrap() += 1;
                }
            });
        }
    });
    let guard = lk.blocking_lock();
    assert_that!(*guard, gt 0);
    assert_that!(*guard, le NUMBER_OF_THREADS);
}

#[test]
fn lock_is_hold_until_guard_drops() {
    let _watchdog = Watchdog::new();
    let lk = SpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| lk.blocking_lock().push(1990));
        s.spawn(|| {
            let mut guard = lk.blocking_lock();
            guard.push(4);
            guard.push(6);
        });
    });
    let guard = lk.blocking_lock();
    assert_that!(guard.as_slice(), any_of [[1990, 4, 6], [4, 6, 1990]]);
}

#[test]
fn placement_default_works() {
    let layout = Layout::new::<SpinLock<Option<u8>>>();
    let raw_memory = unsafe { alloc(layout) } as *mut SpinLock<Option<u8>>;
    unsafe { SpinLock::placement_default(raw_memory) };

    let lk = unsafe { &mut *raw_memory }.try_lock();
    assert_that!(lk, is_some);
    let mut guard = lk.unwrap();
    assert_that!(guard, is_none);
    *guard = Some(34);

    drop(guard);
    unsafe { core::ptr::drop_in_place(raw_memory) };
    unsafe { dealloc(raw_memory.cast(), layout) };
}

#[test]
fn drop_called_for_underlying_value() {
    let state = LifetimeTracker::start_tracking();
    let lk = SpinLock::<LifetimeTracker>::new(LifetimeTracker::default());
    assert_that!(state.number_of_living_instances(), eq 1);
    drop(lk);
    assert_that!(state.number_of_living_instances(), eq 0);
}
