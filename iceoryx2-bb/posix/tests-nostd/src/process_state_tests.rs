// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

extern crate iceoryx2_bb_loggers;

use iceoryx2_bb_posix_tests_common::process_state_tests;
use iceoryx2_bb_testing_macros::inventory_test;

#[inventory_test]
pub fn process_state_guard_can_be_created() {
    process_state_tests::process_state_guard_can_be_created();
}

#[inventory_test]
pub fn process_state_guard_removes_file_when_dropped() {
    process_state_tests::process_state_guard_removes_file_when_dropped();
}

#[inventory_test]
pub fn process_state_guard_cannot_use_already_existing_file() {
    process_state_tests::process_state_guard_cannot_use_already_existing_file();
}

#[inventory_test]
pub fn process_state_monitor_detects_dead_state() {
    process_state_tests::process_state_monitor_detects_dead_state();
}

#[inventory_test]
pub fn process_state_monitor_detects_non_existing_state() {
    process_state_tests::process_state_monitor_detects_non_existing_state();
}

#[inventory_test]
pub fn process_state_monitor_transitions_work_starting_from_non_existing_process() {
    process_state_tests::process_state_monitor_transitions_work_starting_from_non_existing_process(
    );
}

#[inventory_test]
pub fn process_state_monitor_transitions_work_starting_from_existing_process() {
    process_state_tests::process_state_monitor_transitions_work_starting_from_existing_process();
}

#[inventory_test]
pub fn process_state_monitor_detects_initialized_state() {
    process_state_tests::process_state_monitor_detects_initialized_state();
}

#[inventory_test]
pub fn process_state_owner_lock_cannot_be_created_when_process_does_not_exist() {
    process_state_tests::process_state_owner_lock_cannot_be_created_when_process_does_not_exist();
}

#[inventory_test]
pub fn process_state_cleaner_removes_state_files_on_drop() {
    process_state_tests::process_state_cleaner_removes_state_files_on_drop();
}

#[inventory_test]
pub fn process_state_cleaner_keeps_state_files_when_abandoned() {
    process_state_tests::process_state_cleaner_keeps_state_files_when_abandoned();
}

// START: OS with IPC only lock detection
//
// the lock detection does work on some OS only in the inter process context.
// In the process local context the lock is not detected when the fcntl GETLK call is originating
// from the same thread os the fcntl SETLK call. If it is called from a different thread GETLK
// blocks despite it should be non-blocking.
#[inventory_test]
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "nto"
)))]
pub fn process_state_monitor_detects_alive_state_from_existing_process() {
    process_state_tests::process_state_monitor_detects_alive_state_from_existing_process();
}

#[inventory_test]
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "nto"
)))]
pub fn process_state_owner_lock_cannot_be_acquired_from_living_process() {
    process_state_tests::process_state_owner_lock_cannot_be_acquired_from_living_process();
}

#[inventory_test]
#[cfg(not(any(
    target_os = "linux",
    target_os = "freebsd",
    target_os = "macos",
    target_os = "nto"
)))]
pub fn process_state_owner_lock_cannot_be_acquired_twice() {
    process_state_tests::process_state_owner_lock_cannot_be_acquired_twice();
}

// END: OS with IPC only lock detection
