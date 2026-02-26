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

#![allow(clippy::disallowed_types)]

extern crate alloc;

use iceoryx2_bb_concurrency_tests_common::spin_lock_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn spin_lock_try_lock_locks() {
    spin_lock_tests::try_lock_locks();
}

#[inventory_test]
fn spin_lock_lock_guard_unlocks_when_dropped() {
    spin_lock_tests::lock_guard_unlocks_when_dropped();
}

#[inventory_test]
fn spin_lock_lock_guard_behaves_like_reference() {
    spin_lock_tests::lock_guard_behaves_like_reference();
}

#[inventory_test]
fn spin_lock_blocking_lock_locks_exclusively() {
    spin_lock_tests::blocking_lock_locks_exclusively();
}

#[inventory_test]
fn spin_lock_try_lock_locks_exclusively() {
    spin_lock_tests::try_lock_locks_exclusively();
}

#[inventory_test]
fn spin_lock_lock_is_hold_until_guard_drops() {
    spin_lock_tests::lock_is_hold_until_guard_drops();
}

#[inventory_test]
fn spin_lock_drop_called_for_underlying_value() {
    spin_lock_tests::drop_called_for_underlying_value();
}
