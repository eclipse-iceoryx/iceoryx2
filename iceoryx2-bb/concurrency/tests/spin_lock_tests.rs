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

extern crate alloc;
use alloc::alloc::Layout;
use alloc::alloc::{alloc, dealloc};

use iceoryx2_bb_concurrency::spin_lock::SpinLock;
use iceoryx2_bb_concurrency_tests_common::spin_lock_tests;
use iceoryx2_bb_elementary_traits::placement_default::PlacementDefault;
use iceoryx2_pal_testing::assert_that;

#[test]
fn spin_lock_try_lock_locks() {
    spin_lock_tests::try_lock_locks();
}

#[test]
fn spin_lock_lock_guard_unlocks_when_dropped() {
    spin_lock_tests::lock_guard_unlocks_when_dropped();
}

#[test]
fn spin_lock_lock_guard_behaves_like_reference() {
    spin_lock_tests::lock_guard_behaves_like_reference();
}

#[test]
fn spin_lock_blocking_lock_locks_exclusively() {
    spin_lock_tests::blocking_lock_locks_exclusively();
}

#[test]
fn spin_lock_try_lock_locks_exclusively() {
    spin_lock_tests::try_lock_locks_exclusively();
}

#[test]
fn spin_lock_lock_is_hold_until_guard_drops() {
    spin_lock_tests::lock_is_hold_until_guard_drops();
}

#[test]
fn spin_lock_drop_called_for_underlying_value() {
    spin_lock_tests::drop_called_for_underlying_value();
}

#[test]
fn spin_lock_placement_default_works() {
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
