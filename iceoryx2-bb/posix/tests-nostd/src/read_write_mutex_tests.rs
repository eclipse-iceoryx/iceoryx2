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

use iceoryx2_bb_posix_tests_common::read_write_mutex_tests;
use iceoryx2_bb_testing_nostd_macros::inventory_test;

#[inventory_test]
fn read_write_mutex_lock_works() {
    read_write_mutex_tests::read_write_mutex_lock_works();
}

#[inventory_test]
fn read_write_mutex_try_lock_works() {
    read_write_mutex_tests::read_write_mutex_try_lock_works();
}

#[inventory_test]
fn read_write_mutex_write_lock_blocks_read_and_write_locks() {
    read_write_mutex_tests::read_write_mutex_write_lock_blocks_read_and_write_locks();
}

#[inventory_test]
fn read_write_mutex_read_lock_blocks_only_write_locks() {
    read_write_mutex_tests::read_write_mutex_read_lock_blocks_only_write_locks();
}

#[inventory_test]
fn read_write_mutex_try_lock_fails_when_lock_was_acquired() {
    read_write_mutex_tests::read_write_mutex_try_lock_fails_when_lock_was_acquired();
}

#[inventory_test]
fn read_write_mutex_multiple_ipc_mutex_are_working() {
    read_write_mutex_tests::read_write_mutex_multiple_ipc_mutex_are_working();
}
