// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use alloc::vec;
use alloc::vec::Vec;

use iceoryx2_bb_concurrency::atomic::{AtomicBool, Ordering};
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::{Time, nanosleep};
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorManagement;
use iceoryx2_bb_posix::group::*;
use iceoryx2_bb_posix::named_pipe::NamedPipeBuilder;
use iceoryx2_bb_posix::named_pipe::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_posix::user::*;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_pal_posix::posix::{
    POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING, POSIX_SUPPORT_NAMED_PIPE, POSIX_SUPPORT_PERMISSIONS,
    POSIX_SUPPORT_USERS_AND_GROUPS,
};

pub const TIMEOUT: core::time::Duration = core::time::Duration::from_millis(100);

struct TestFixture {
    named_pipe: FilePath,
}

impl TestFixture {
    fn new() -> TestFixture {
        create_test_directory();
        let named_pipe = generate_file_path();
        File::remove(&named_pipe).ok();
        TestFixture { named_pipe }
    }

    fn file(&self) -> &FilePath {
        &self.named_pipe
    }

    fn create_named_pipe(&self, name: &FilePath) -> NamedPipe {
        let named_pipe = NamedPipeBuilder::new(name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create();

        assert_that!(named_pipe, is_ok);
        named_pipe.unwrap()
    }

    fn open_named_pipe(&self, name: &FilePath) -> NamedPipe {
        let named_pipe = NamedPipeBuilder::new(name).open_existing(AccessMode::Read);

        assert_that!(named_pipe, is_ok);
        named_pipe.unwrap()
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        File::remove(self.file()).expect("failed to cleanup test file for named pipe");
    }
}

#[test]
pub fn opening_non_existing_named_pipe_fails() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test_non_existing_named_pipe = generate_file_path();
    let result =
        NamedPipeBuilder::new(&test_non_existing_named_pipe).open_existing(AccessMode::Read);

    assert_that!(result, is_err);
    assert_that!(result.err().unwrap(), eq NamedPipeOpenError::FileDoesNotExist);
}

#[test]
pub fn creating_existing_named_pipe_on_creation_mode_create_exclusive_suceeds() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test_named_pipe_path = generate_file_path();
    let result = NamedPipeBuilder::new(&test_named_pipe_path)
        .creation_mode(CreationMode::CreateExclusive)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn purge_and_create_non_existing_named_pipe_suceeds() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test_named_pipe_path = generate_file_path();
    let result = NamedPipeBuilder::new(&test_named_pipe_path)
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn purge_and_create_existing_named_pipe_suceeds() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test = TestFixture::new();
    test.create_named_pipe(test.file());
    let result = NamedPipeBuilder::new(test.file())
        .has_ownership(true)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn open_or_create_non_existing_named_pipe_suceeds() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test_named_pipe_path = generate_file_path();
    let result = NamedPipeBuilder::new(&test_named_pipe_path)
        .has_ownership(true)
        .creation_mode(CreationMode::OpenOrCreate)
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn open_or_create_existing_named_pipe_suceeds() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test = TestFixture::new();
    test.create_named_pipe(test.file());
    let result = NamedPipeBuilder::new(test.file())
        .has_ownership(true)
        .creation_mode(CreationMode::OpenOrCreate)
        .permission(Permission::OWNER_READ_WRITE)
        .owner(User::from_self().unwrap().uid())
        .group(Group::from_self().unwrap().gid())
        .create();

    assert_that!(result, is_ok);
}

#[test]
pub fn creating_named_pipe_applies_additional_settings() {
    test_requires!(
        POSIX_SUPPORT_NAMED_PIPE && POSIX_SUPPORT_PERMISSIONS && POSIX_SUPPORT_USERS_AND_GROUPS
    );

    let test_path = generate_file_path();
    let test_named_pipe = NamedPipeBuilder::new(&test_path)
        .has_ownership(false)
        .creation_mode(CreationMode::PurgeAndCreate)
        .permission(Permission::OWNER_READ_WRITE)
        .owner(User::from_self().unwrap().uid())
        .group(Group::from_self().unwrap().gid())
        .create();

    assert_that!(test_named_pipe, is_ok);

    let test_named_pipe = test_named_pipe.ok().unwrap();
    assert_that!(
        test_named_pipe.metadata().unwrap().permission(), eq
        Permission::OWNER_READ_WRITE
    );
}

#[test]
pub fn enable_async_io_works() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE && POSIX_SUPPORT_ADVANCED_SIGNAL_HANDLING);

    let test_named_pipe_path = generate_file_path();
    let sut = NamedPipeBuilder::new(&test_named_pipe_path)
        .creation_mode(CreationMode::CreateExclusive)
        .create()
        .unwrap();

    assert_that!(sut.set_async_io(true), is_ok);
    assert_that!(sut.is_async_io(), eq true);
}

#[test]
pub fn simple_read_write_with_single_fd_works() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test = TestFixture::new();
    let sut_named_pipe = test.create_named_pipe(test.file());

    let content: [u8; 5] = [1, 2, 3, 4, 5];
    let result = sut_named_pipe.write(content.as_slice());
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    let mut read_buffer: [u8; 5] = [0u8; 5];
    let result = sut_named_pipe.read(&mut read_buffer);
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    assert_that!(content, eq read_buffer);
}

#[test]
pub fn simple_read_write_from_with_different_fds_works() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test = TestFixture::new();
    let sut_writer = test.create_named_pipe(test.file());

    let content: [u8; 5] = [1, 2, 3, 4, 5];
    let result = sut_writer.write(content.as_slice());
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    let sut_reader = test.open_named_pipe(test.file());
    let mut read_buffer: [u8; 5] = [0u8; 5];
    let result = sut_reader.read(&mut read_buffer);
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    assert_that!(content, eq read_buffer);
}

#[test]
fn try_read_works() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let test = TestFixture::new();
    let sut_writer = test.create_named_pipe(test.file());

    let content: [u8; 5] = [1, 2, 3, 4, 5];
    let result = sut_writer.write(content.as_slice());
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    let sut_reader = test.open_named_pipe(test.file());
    let mut read_buffer: [u8; 5] = [0u8; 5];
    let result = sut_reader.try_read(&mut read_buffer);
    assert_that!(result, is_ok);
    assert_that!(content, len result.ok().unwrap() as usize);

    assert_that!(content, eq read_buffer);
}

#[test]
fn blocking_read_blocks() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let _watchdog = Watchdog::new();

    let pipe_test_path = generate_file_path();
    let received_message = AtomicBool::new(false);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let sut_reader = NamedPipeBuilder::new(&pipe_test_path)
                    .has_ownership(false)
                    .creation_mode(CreationMode::PurgeAndCreate)
                    .create()
                    .unwrap();
                barrier.wait();

                let mut receive_data: Vec<u8> = vec![0, 0, 0, 0, 0, 0];
                let result = sut_reader.blocking_read(receive_data.as_mut_slice());
                assert_that!(result, is_ok);
                assert_that!(result.unwrap(), eq send_data.len() as _);
                received_message.store(true, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        barrier.wait();
        let sut_writer = NamedPipeBuilder::new(&pipe_test_path)
            .has_ownership(false)
            .creation_mode(CreationMode::OpenOrCreate)
            .create()
            .unwrap();

        nanosleep(TIMEOUT).unwrap();
        let received_message_old = received_message.load(Ordering::Relaxed);
        sut_writer.write(send_data.as_slice()).unwrap();

        assert_that!(received_message_old, eq false);

        Ok(())
    })
    .expect("failed to spawn thread");

    assert_that!(received_message.load(Ordering::Relaxed), eq true);
}

#[test]
fn timed_read_blocks() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let _watchdog = Watchdog::new();

    let pipe_test_path = generate_file_path();
    let received_message = AtomicBool::new(false);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];

    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let sut_reader = NamedPipeBuilder::new(&pipe_test_path)
                    .has_ownership(false)
                    .creation_mode(CreationMode::PurgeAndCreate)
                    .create()
                    .unwrap();
                barrier.wait();

                let mut receive_data: Vec<u8> = vec![0, 0, 0, 0, 0, 0];
                let result = sut_reader.timed_read(receive_data.as_mut_slice(), TIMEOUT * 3);
                assert_that!(result, is_ok);
                assert_that!(result.unwrap(), eq send_data.len() as _);
                received_message.store(true, Ordering::Relaxed);
            })
            .expect("failed to spawn thread");

        barrier.wait();
        let sut_writer = NamedPipeBuilder::new(&pipe_test_path)
            .has_ownership(false)
            .creation_mode(CreationMode::OpenOrCreate)
            .create()
            .unwrap();

        nanosleep(TIMEOUT).unwrap();
        let received_message_old = received_message.load(Ordering::Relaxed);
        sut_writer.write(send_data.as_slice()).unwrap();

        assert_that!(received_message_old, eq false);

        Ok(())
    })
    .expect("failed to spawn thread");

    assert_that!(received_message.load(Ordering::Relaxed), eq true);
}

#[test]
fn timed_read_blocks_at_least_for_timeout() {
    test_requires!(POSIX_SUPPORT_NAMED_PIPE);
    let pipe_test_path = generate_file_path();

    let _sut_writer = NamedPipeBuilder::new(&pipe_test_path)
        .has_ownership(false)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_reader = NamedPipeBuilder::new(&pipe_test_path)
        .open_existing(AccessMode::Read)
        .unwrap();

    let mut read_buffer = [0u8; 8];
    let start = Time::now().expect("failed to get current time");

    assert_that!(sut_reader.timed_read(&mut read_buffer, TIMEOUT).unwrap(), eq 0);
    assert_that!(start.elapsed().expect("failed to get elapsed time"), time_at_least TIMEOUT);
}
