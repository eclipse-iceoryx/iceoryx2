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

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::config;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_lock::*;
use iceoryx2_bb_posix::process::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_FILE_LOCK;

use core::sync::atomic::{AtomicU64, Ordering};
use std::thread;

fn generate_file_name() -> FilePath {
    let mut file = FileName::new(b"file_lock_tests_").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    FilePath::from_path_and_file(&config::test_directory(), &file).unwrap()
}

struct TestFixture<'a> {
    file_name: FilePath,
    sut: FileLock<'a, File>,
}

impl<'a> TestFixture<'a> {
    fn new(handle: &'a ReadWriteMutexHandle<File>) -> TestFixture<'a> {
        create_test_directory();
        let file_name = generate_file_name();
        let file = FileBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .permission(Permission::OWNER_ALL)
            .create()
            .expect("");

        TestFixture {
            file_name,
            sut: FileLockBuilder::new().create(file, handle).expect(""),
        }
    }
}

impl<'a> Drop for TestFixture<'a> {
    fn drop(&mut self) {
        File::remove(&self.file_name).expect("");
    }
}

#[test]
fn file_lock_unlocked_by_default() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Unlock);
    assert_that!(result.pid_of_owner().value(), eq 0);
}

#[test]
fn file_lock_write_lock_blocks_other_write_locks() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let guard = test.sut.write_lock().unwrap();

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Write);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    drop(guard);

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Unlock);
    assert_that!(result.pid_of_owner().value(), eq 0);
}

#[test]
fn file_lock_write_try_lock_denies_other_try_locks() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let guard = test.sut.write_try_lock();

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Write);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    assert_that!(test.sut.write_try_lock().unwrap(), is_none);

    drop(guard);

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Unlock);
    assert_that!(result.pid_of_owner().value(), eq 0);

    assert_that!(test.sut.write_try_lock().unwrap(), is_some);
}

#[test]
fn file_lock_read_lock_allows_other_read_locks() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let guard = test.sut.read_lock().unwrap();

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Read);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    let guard2 = test.sut.read_lock().unwrap();
    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Read);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    drop(guard);

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Read);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    drop(guard2);

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Unlock);
    assert_that!(result.pid_of_owner().value(), eq 0);
}

#[test]
fn file_lock_read_try_lock_allows_other_read_try_locks() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let guard = test.sut.read_try_lock().unwrap();

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Read);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    let guard2 = test.sut.read_try_lock().unwrap();
    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Read);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    drop(guard);

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Read);
    assert_that!(result.pid_of_owner(), eq Process::from_self().id());

    drop(guard2);

    let result = test.sut.get_lock_state().unwrap();
    assert_that!(result.lock_type(), eq LockType::Unlock);
    assert_that!(result.pid_of_owner().value(), eq 0);
}

#[test]
fn file_lock_one_read_blocks_write() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let guard = test.sut.read_lock().unwrap();

    assert_that!(test.sut.write_try_lock().unwrap(), is_none);
    drop(guard);
    assert_that!(test.sut.write_try_lock().unwrap(), is_some);
}

#[test]
fn file_lock_multiple_readers_blocks_write() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let guard = test.sut.read_lock().unwrap();
    let guard2 = test.sut.read_lock().unwrap();

    assert_that!(test.sut.write_try_lock().unwrap(), is_none);
    drop(guard2);
    assert_that!(test.sut.write_try_lock().unwrap(), is_none);
    drop(guard);
    assert_that!(test.sut.write_try_lock().unwrap(), is_some);
}

#[test]
fn file_lock_write_lock_blocks() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let counter = AtomicU64::new(0);
    thread::scope(|s| {
        let guard = test.sut.write_lock().expect("");

        s.spawn(|| {
            test.sut.read_lock().expect("");
            counter.fetch_add(1, Ordering::Relaxed);
        });

        s.spawn(|| {
            test.sut.write_lock().expect("");
            counter.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(core::time::Duration::from_millis(10));
        let counter_old = counter.load(Ordering::Relaxed);
        drop(guard);
        thread::sleep(core::time::Duration::from_millis(10));

        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 2);
    });
}

#[test]
fn file_lock_read_lock_blocks_write_locks() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let counter = AtomicU64::new(0);
    thread::scope(|s| {
        let guard = test.sut.read_lock().expect("");

        s.spawn(|| {
            test.sut.read_lock().expect("");
            counter.fetch_add(1, Ordering::Relaxed);
        });

        s.spawn(|| {
            test.sut.write_lock().expect("");
            counter.fetch_add(2, Ordering::Relaxed);
        });

        thread::sleep(core::time::Duration::from_millis(10));
        let counter_old = counter.load(Ordering::Relaxed);
        drop(guard);
        thread::sleep(core::time::Duration::from_millis(10));

        assert_that!(counter_old, eq 1);
        assert_that!(counter.load(Ordering::Relaxed), eq 3);
    });
}

#[test]
fn file_lock_read_try_lock_does_not_block() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let counter = AtomicU64::new(0);

    thread::scope(|s| {
        let _guard = test.sut.write_lock().expect("");

        s.spawn(|| {
            test.sut.read_try_lock().expect("");
            counter.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(core::time::Duration::from_millis(10));
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
    });
}

#[test]
fn file_lock_write_try_lock_does_not_block() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let counter = AtomicU64::new(0);

    thread::scope(|s| {
        let _guard = test.sut.write_lock().expect("");

        s.spawn(|| {
            test.sut.write_try_lock().expect("");
            counter.fetch_add(1, Ordering::Relaxed);
        });

        thread::sleep(core::time::Duration::from_millis(10));
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
    });
}

#[test]
fn file_lock_read_write_works() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let mut guard = test.sut.write_lock().expect("");

    assert_that!(guard.write(b"hello").unwrap(), eq 5);
    drop(guard);

    let guard = test.sut.write_lock().expect("");
    assert_that!(guard.seek(0), is_ok);

    let mut content = vec![];
    assert_that!(guard.read_to_vector(&mut content).unwrap(), eq 5);
    assert_that!(content, eq b"hello");

    drop(guard);

    let guard = test.sut.read_lock().expect("");
    assert_that!(guard.seek(0), is_ok);

    let mut content = vec![];
    assert_that!(guard.read_to_vector(&mut content).unwrap(), eq 5);
    assert_that!(content, eq b"hello");
}

#[test]
fn file_lock_try_lock_fails_when_locked() {
    test_requires!(POSIX_SUPPORT_FILE_LOCK);

    let handle = ReadWriteMutexHandle::new();
    let test = TestFixture::new(&handle);
    let _guard = test.sut.write_lock().expect("");

    assert_that!(test.sut.read_try_lock().unwrap(), is_none);
    assert_that!(test.sut.write_try_lock().unwrap(), is_none);
}
