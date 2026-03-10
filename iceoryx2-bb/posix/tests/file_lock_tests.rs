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

use iceoryx2_bb_posix_tests_common::file_lock_tests;

#[test]
fn file_lock_unlocked_by_default() {
    file_lock_tests::file_lock_unlocked_by_default();
}

#[test]
fn file_lock_write_lock_blocks_other_write_locks() {
    file_lock_tests::file_lock_write_lock_blocks_other_write_locks();
}

#[test]
fn file_lock_write_try_lock_denies_other_try_locks() {
    file_lock_tests::file_lock_write_try_lock_denies_other_try_locks();
}

#[test]
fn file_lock_read_lock_allows_other_read_locks() {
    file_lock_tests::file_lock_read_lock_allows_other_read_locks();
}

#[test]
fn file_lock_read_try_lock_allows_other_read_try_locks() {
    file_lock_tests::file_lock_read_try_lock_allows_other_read_try_locks();
}

#[test]
fn file_lock_one_read_blocks_write() {
    file_lock_tests::file_lock_one_read_blocks_write();
}

#[test]
fn file_lock_multiple_readers_blocks_write() {
    file_lock_tests::file_lock_multiple_readers_blocks_write();
}

#[test]
fn file_lock_write_lock_blocks() {
    file_lock_tests::file_lock_write_lock_blocks();
}

#[test]
fn file_lock_read_lock_blocks_write_locks() {
    file_lock_tests::file_lock_read_lock_blocks_write_locks();
}

#[test]
fn file_lock_read_try_lock_does_not_block() {
    file_lock_tests::file_lock_read_try_lock_does_not_block();
}

#[test]
fn file_lock_write_try_lock_does_not_block() {
    file_lock_tests::file_lock_write_try_lock_does_not_block();
}

#[test]
fn file_lock_read_write_works() {
    file_lock_tests::file_lock_read_write_works();
}

#[test]
fn file_lock_try_lock_fails_when_locked() {
    file_lock_tests::file_lock_try_lock_fails_when_locked();
}
