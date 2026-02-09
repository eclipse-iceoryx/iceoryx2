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

use iceoryx2_bb_concurrency_tests_common::strategy_rwlock_tests;

#[test]
fn strategy_rwlock_reader_preference_try_write_lock_blocks_read_locks() {
    strategy_rwlock_tests::strategy_rwlock_reader_preference_try_write_lock_blocks_read_locks();
}

#[test]
fn strategy_rwlock_reader_preference_multiple_read_locks_block_write_lock() {
    strategy_rwlock_tests::strategy_rwlock_reader_preference_multiple_read_locks_block_write_lock();
}

#[test]
fn strategy_rwlock_reader_preference_write_lock_and_unlock_works() {
    strategy_rwlock_tests::strategy_rwlock_reader_preference_write_lock_and_unlock_works();
}

#[test]
fn strategy_rwlock_reader_preference_try_read_lock_and_unlock_works() {
    strategy_rwlock_tests::strategy_rwlock_reader_preference_try_read_lock_and_unlock_works();
}

#[test]
fn strategy_rwlock_reader_preference_read_lock_and_unlock_works() {
    strategy_rwlock_tests::strategy_rwlock_reader_preference_read_lock_and_unlock_works();
}

#[test]
fn strategy_rwlock_reader_preference_read_lock_blocks_only_write_locks() {
    strategy_rwlock_tests::strategy_rwlock_reader_preference_read_lock_blocks_only_write_locks();
}

#[test]
fn strategy_rwlock_reader_preference_write_lock_blocks_everything() {
    strategy_rwlock_tests::strategy_rwlock_reader_preference_write_lock_blocks_everything();
}

//////////////////////
/// Writer Preference
//////////////////////

#[test]
fn strategy_rwlock_writer_preference_try_write_lock_blocks_read_locks() {
    strategy_rwlock_tests::strategy_rwlock_writer_preference_try_write_lock_blocks_read_locks();
}

#[test]
fn strategy_rwlock_writer_preference_multiple_read_locks_block_write_lock() {
    strategy_rwlock_tests::strategy_rwlock_writer_preference_multiple_read_locks_block_write_lock();
}

#[test]
fn strategy_rwlock_writer_preference_write_lock_and_unlock_works() {
    strategy_rwlock_tests::strategy_rwlock_writer_preference_write_lock_and_unlock_works();
}

#[test]
fn strategy_rwlock_writer_preference_try_read_lock_and_unlock_works() {
    strategy_rwlock_tests::strategy_rwlock_writer_preference_try_read_lock_and_unlock_works();
}

#[test]
fn strategy_rwlock_writer_preference_read_lock_and_unlock_works() {
    strategy_rwlock_tests::strategy_rwlock_writer_preference_read_lock_and_unlock_works();
}

#[test]
fn strategy_rwlock_writer_preference_write_lock_blocks_everything() {
    strategy_rwlock_tests::strategy_rwlock_writer_preference_write_lock_blocks_everything();
}
