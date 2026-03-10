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

use iceoryx2_bb_posix_tests_common::semaphore_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn semaphore_named_semaphore_initializes_correctly() {
    semaphore_tests::semaphore_named_semaphore_initializes_correctly();
}

#[inventory_test]
fn semaphore_named_semaphore_opens_correctly() {
    semaphore_tests::semaphore_named_semaphore_opens_correctly();
}

#[inventory_test]
fn semaphore_unnamed_semaphore_initializes_correctly() {
    semaphore_tests::semaphore_unnamed_semaphore_initializes_correctly();
}

#[inventory_test]
fn semaphore_named_semaphore_post_and_try_wait_work() {
    semaphore_tests::semaphore_named_semaphore_post_and_try_wait_work();
}

#[inventory_test]
fn semaphore_unnamed_semaphore_post_and_try_wait_work() {
    semaphore_tests::semaphore_unnamed_semaphore_post_and_try_wait_work();
}

#[inventory_test]
fn semaphore_named_semaphore_post_and_wait_work() {
    semaphore_tests::semaphore_named_semaphore_post_and_wait_work();
}

#[inventory_test]
fn semaphore_unnamed_semaphore_post_and_wait_work() {
    semaphore_tests::semaphore_unnamed_semaphore_post_and_wait_work();
}

#[inventory_test]
fn semaphore_named_semaphore_post_and_timed_wait_work() {
    semaphore_tests::semaphore_named_semaphore_post_and_timed_wait_work();
}

#[inventory_test]
fn semaphore_unnamed_semaphore_post_and_timed_wait_work() {
    semaphore_tests::semaphore_unnamed_semaphore_post_and_timed_wait_work();
}

#[inventory_test]
fn semaphore_named_semaphore_wait_blocks() {
    semaphore_tests::semaphore_named_semaphore_wait_blocks();
}

#[inventory_test]
fn semaphore_unnamed_semaphore_wait_blocks() {
    semaphore_tests::semaphore_unnamed_semaphore_wait_blocks();
}

#[inventory_test]
fn semaphore_named_semaphore_timed_wait_blocks() {
    semaphore_tests::semaphore_named_semaphore_timed_wait_blocks();
}

#[inventory_test]
fn semaphore_unnamed_semaphore_timed_wait_blocks() {
    semaphore_tests::semaphore_unnamed_semaphore_timed_wait_blocks();
}

#[inventory_test]
fn semaphore_named_semaphore_timed_wait_waits_at_least_timeout() {
    semaphore_tests::semaphore_named_semaphore_timed_wait_waits_at_least_timeout();
}

#[inventory_test]
fn semaphore_unnamed_semaphore_timed_wait_waits_at_least_timeout() {
    semaphore_tests::semaphore_unnamed_semaphore_timed_wait_waits_at_least_timeout();
}

#[inventory_test]
fn unnamed_semaphore_multiple_ipc_semaphores_are_working() {
    semaphore_tests::unnamed_semaphore_multiple_ipc_semaphores_are_working();
}

#[inventory_test]
fn unnamed_semaphore_acquire_uninitialized_ipc_handle_failes() {
    semaphore_tests::unnamed_semaphore_acquire_uninitialized_ipc_handle_failes();
}

#[inventory_test]
fn unnamed_semaphore_acquiring_non_ipc_capable_handle_fails() {
    semaphore_tests::unnamed_semaphore_acquiring_non_ipc_capable_handle_fails();
}
