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

#![allow(clippy::disallowed_types)]

use core::time::Duration;

use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use iceoryx2_bb_posix::clock::Time;
use iceoryx2_bb_posix::file_descriptor::FileDescriptorBased;
use iceoryx2_bb_posix::file_descriptor_set::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::testing::generate_file_path;
use iceoryx2_bb_posix::unix_datagram_socket::*;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing_macros::test;
use iceoryx2_pal_posix::posix;

static TIMEOUT: Duration = Duration::from_millis(10);

#[test]
pub fn timed_wait_blocks_at_least_timeout() {
    create_test_directory();
    let socket_name = generate_file_path();

    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let fd_set = FileDescriptorSet::new();
    let _guard = fd_set.add(&sut_receiver).unwrap();

    assert_that!(fd_set.contains(&sut_receiver), eq true);
    assert_that!(fd_set.contains(&sut_sender),  eq false);

    let start = Time::now().expect("failed to get current time");

    let mut result = vec![];
    fd_set
        .timed_wait(TIMEOUT, FileEvent::Read, |fd| {
            result.push(unsafe { fd.native_handle() })
        })
        .unwrap();

    assert_that!(start.elapsed().expect("failed to get elapsed time"), time_at_least TIMEOUT);
    assert_that!(result, len 0);
}

#[test]
pub fn add_and_remove_works() {
    let fd_set = FileDescriptorSet::new();
    let mut sockets = vec![];
    let number_of_fds: usize = core::cmp::min(128, posix::FD_SETSIZE);

    create_test_directory();
    for _ in 0..number_of_fds {
        let socket_name = generate_file_path();
        sockets.push(
            UnixDatagramReceiverBuilder::new(&socket_name)
                .creation_mode(CreationMode::PurgeAndCreate)
                .create()
                .unwrap(),
        );
    }

    let mut counter = 0;
    let mut guards = vec![];
    for fd in &sockets {
        counter += 1;
        assert_that!(fd_set.contains(fd), eq false);
        let guard = fd_set.add(fd);
        assert_that!(guard, is_ok);
        guards.push(guard);
        assert_that!(fd_set.contains(fd), eq true);
        assert_that!(fd_set.len(), eq counter);
    }

    let mut counter = 0;
    for fd in sockets.iter().rev() {
        counter += 1;
        assert_that!(fd_set.contains(fd), eq true);
        guards.pop();
        assert_that!(fd_set.contains(fd), eq false);
        assert_that!(fd_set.len(), eq number_of_fds - counter);
    }
}

#[test]
pub fn add_same_fd_twice_fails() {
    let fd_set = FileDescriptorSet::new();

    create_test_directory();
    let socket_name = generate_file_path();
    let socket = UnixDatagramReceiverBuilder::new(&socket_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let _guard = fd_set.add(&socket).unwrap();

    let result = fd_set.add(&socket);
    assert_that!(result.err(), eq Some(FileDescriptorSetAddError::AlreadyAttached));
}

#[test]
pub fn timed_wait_works() {
    create_test_directory();
    let socket_name = generate_file_path();

    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let fd_set = FileDescriptorSet::new();
    let _guard = fd_set.add(&sut_receiver).unwrap();
    let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];
    sut_sender.blocking_send(send_data.as_slice()).unwrap();

    let mut result = vec![];
    let number_of_notifications = fd_set
        .timed_wait(TIMEOUT, FileEvent::Read, |fd| {
            result.push(unsafe { fd.native_handle() })
        })
        .unwrap();

    assert_that!(number_of_notifications, eq 1);
    assert_that!(result, len 1);
    assert_that!(result[0], eq unsafe{sut_receiver.file_descriptor().native_handle()});
}

#[test]
pub fn blocking_wait_immediately_returns_notifications() {
    create_test_directory();
    let socket_name = generate_file_path();

    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let fd_set = FileDescriptorSet::new();
    let _guard = fd_set.add(&sut_receiver).unwrap();
    let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];
    sut_sender.blocking_send(send_data.as_slice()).unwrap();

    let mut result = vec![];
    let number_of_notifications = fd_set
        .blocking_wait(FileEvent::Read, |fd| {
            result.push(unsafe { fd.native_handle() })
        })
        .unwrap();

    assert_that!(number_of_notifications, eq 1);
    assert_that!(result, len 1);
    assert_that!(result[0], eq unsafe{sut_receiver.file_descriptor().native_handle()});
}

#[test]
pub fn file_descriptor_guard_has_access_to_underlying_fd() {
    create_test_directory();
    let socket_name = generate_file_path();

    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let fd_set = FileDescriptorSet::new();
    let guard = fd_set.add(&sut_receiver).unwrap();

    unsafe {
        assert_that!(guard.file_descriptor().native_handle(), eq sut_receiver.file_descriptor().native_handle())
    }
}

#[test]
pub fn file_descriptor_debug_works() {
    let sut = FileDescriptorSet::new();
    assert_that!(format!("{sut:?}").starts_with("FileDescriptorSet"), eq true);
}

#[test]
pub fn file_descriptor_triggering_many_returns_correct_number_of_notifications() {
    let fd_set = FileDescriptorSet::new();
    let mut sockets = vec![];
    let mut senders = vec![];
    let number_of_fds: usize = core::cmp::min(32, posix::FD_SETSIZE);

    create_test_directory();
    for _ in 0..number_of_fds {
        let socket_name = generate_file_path();
        sockets.push(
            UnixDatagramReceiverBuilder::new(&socket_name)
                .creation_mode(CreationMode::PurgeAndCreate)
                .create()
                .unwrap(),
        );

        senders.push(
            UnixDatagramSenderBuilder::new(&socket_name)
                .create()
                .unwrap(),
        );
    }

    let mut guards = vec![];
    for fd in &sockets {
        guards.push(fd_set.add(fd));
    }

    for sender in senders {
        assert_that!(sender.try_send(b"abc"), eq Ok(true));
    }

    let mut counter = 0;
    let number_of_notifications = fd_set
        .timed_wait(TIMEOUT, FileEvent::Read, |_| {
            counter += 1;
        })
        .unwrap();

    assert_that!(counter, eq number_of_fds);
    assert_that!(number_of_notifications, eq number_of_fds);
}
