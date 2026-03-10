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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::mutex_tests;

#[test]
fn mutex_lock_works() {
    mutex_tests::mutex_lock_works();
}

#[test]
fn mutex_try_lock_works() {
    mutex_tests::mutex_try_lock_works();
}

#[test]
fn mutex_try_lock_leads_to_blocked_mutex() {
    mutex_tests::mutex_try_lock_leads_to_blocked_mutex();
}

#[test]
fn mutex_timed_lock_leads_to_blocked_mutex_realtime() {
    mutex_tests::mutex_timed_lock_leads_to_blocked_mutex_realtime();
}

#[test]
fn mutex_timed_lock_leads_to_blocked_mutex_monotonic() {
    mutex_tests::mutex_timed_lock_leads_to_blocked_mutex_monotonic();
}

#[test]
fn mutex_try_lock_fails_when_already_locked() {
    mutex_tests::mutex_try_lock_fails_when_already_locked();
}

#[test]
fn mutex_timed_lock_blocks_at_least_for_timeout_realtime_clock() {
    mutex_tests::mutex_timed_lock_blocks_at_least_for_timeout_realtime_clock();
}

#[test]
fn mutex_timed_lock_blocks_at_least_for_timeout_monotonic_clock() {
    mutex_tests::mutex_timed_lock_blocks_at_least_for_timeout_monotonic_clock();
}

#[test]
fn mutex_multiple_ipc_mutex_are_working() {
    mutex_tests::mutex_multiple_ipc_mutex_are_working();
}

#[test]
fn mutex_recursive_mutex_can_be_locked_multiple_times_by_same_thread() {
    mutex_tests::mutex_recursive_mutex_can_be_locked_multiple_times_by_same_thread();
}

#[test]
fn mutex_recursive_does_not_unlock_in_the_first_unlock_call() {
    mutex_tests::mutex_recursive_does_not_unlock_in_the_first_unlock_call();
}

#[test]
fn mutex_deadlock_detection_works() {
    mutex_tests::mutex_deadlock_detection_works();
}

#[test]
fn mutex_recursive_mutex_blocks() {
    mutex_tests::mutex_recursive_mutex_blocks();
}

#[test]
fn mutex_with_deadlock_detection_blocks() {
    mutex_tests::mutex_with_deadlock_detection_blocks();
}

// This test fails on QNX due to the mutex created in the separate thread being cleaned up
// before the clean-up code is executed. Needs investigation (#978).
#[cfg(not(target_os = "nto"))]
#[test]
fn mutex_can_be_recovered_when_thread_died() {
    mutex_tests::mutex_can_be_recovered_when_thread_died();
}

// This test fails on QNX due to the mutex created in the separate thread from being cleaned up
// before the clean-up code is added. Needs investigation.
#[test]
#[cfg(not(any(target_os = "macos", target_os = "nto")))]
fn mutex_in_unrecoverable_state_if_state_of_leaked_mutex_is_not_repaired() {
    mutex_tests::mutex_in_unrecoverable_state_if_state_of_leaked_mutex_is_not_repaired();
}
