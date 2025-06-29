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

use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::config::*;
use iceoryx2_bb_posix::creation_mode::*;
use iceoryx2_bb_posix::file::*;
use iceoryx2_bb_posix::file_descriptor::*;
use iceoryx2_bb_posix::socket_ancillary::*;
use iceoryx2_bb_posix::testing::create_test_directory;
use iceoryx2_bb_posix::unique_system_id::UniqueSystemId;
use iceoryx2_bb_posix::unix_datagram_socket::*;
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_system_types::file_path::FilePath;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::test_requires;
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA;
use std::thread;
use std::time::Instant;

const TIMEOUT: Duration = Duration::from_millis(100);

fn generate_socket_name() -> FilePath {
    let mut file = FileName::new(b"unix_datagram_socket_tests").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    FilePath::from_path_and_file(&test_directory(), &file).unwrap()
}

fn generate_file_name() -> FilePath {
    let mut file = FileName::new(b"unix_datagram_socket_file_tests").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();

    FilePath::from_path_and_file(&test_directory(), &file).unwrap()
}

struct TestFixture {
    files: Vec<FilePath>,
}

impl TestFixture {
    fn new() -> TestFixture {
        TestFixture { files: vec![] }
    }

    fn create_file_with_content(&mut self, content: &mut String) -> File {
        let file_name = generate_file_name();
        let mut file = FileBuilder::new(&file_name)
            .creation_mode(CreationMode::PurgeAndCreate)
            .create()
            .unwrap();
        file.write(unsafe { content.as_mut_vec() }.as_slice())
            .unwrap();
        self.files.push(file_name);
        file
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        for file in &self.files {
            File::remove(file).expect("failed to cleanup test file");
        }
    }
}

#[test]
fn unix_datagram_socket_send_receive_works() {
    create_test_directory();
    let socket_name = generate_socket_name();
    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .permission(Permission::OWNER_ALL)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];
    sut_sender.blocking_send(send_data.as_slice()).unwrap();

    let mut receive_data: Vec<u8> = vec![0; 6];
    sut_receiver
        .blocking_receive(receive_data.as_mut_slice())
        .unwrap();

    assert_that!(send_data, eq receive_data);
}

#[test]
fn unix_datagram_socket_adjust_buffer_size_works() {
    create_test_directory();
    let socket_name = generate_socket_name();
    let mut sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .permission(Permission::OWNER_ALL)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let mut sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    assert_that!(sut_receiver.set_receive_buffer_min_size(4096), is_ok);
    assert_that!(sut_receiver.get_receive_buffer_size().unwrap(), ge 4096);

    assert_that!(sut_sender.set_send_buffer_min_size(4096), is_ok);
    assert_that!(sut_sender.get_send_buffer_size().unwrap(), ge 4096);
}

#[test]
fn unix_datagram_socket_non_blocking_mode_returns_zero_when_nothing_was_received() {
    create_test_directory();
    let socket_name = generate_socket_name();
    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .permission(Permission::OWNER_ALL)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let _sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let mut receive_data: Vec<u8> = vec![0, 0, 0, 0];
    let result = sut_receiver.try_receive(receive_data.as_mut_slice());

    assert_that!(result, eq Ok(0));
}

#[test]
fn unix_datagram_socket_blocking_mode_blocks() {
    create_test_directory();
    let socket_name = generate_socket_name();
    let received_message = AtomicBool::new(false);
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let send_data: Vec<u8> = vec![1u8, 3u8, 3u8, 7u8, 13u8, 37u8];

    thread::scope(|s| {
        let t = s.spawn(|| {
            let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
                .permission(Permission::OWNER_ALL)
                .creation_mode(CreationMode::PurgeAndCreate)
                .create()
                .unwrap();
            barrier.wait();

            let mut receive_data: Vec<u8> = vec![0, 0, 0, 0, 0, 0];
            let result = sut_receiver.blocking_receive(receive_data.as_mut_slice());
            assert_that!(result, is_ok);
            assert_that!(result.unwrap(), eq send_data.len() as _);
            received_message.store(true, Ordering::Relaxed);
        });

        barrier.wait();
        let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
            .create()
            .unwrap();

        thread::sleep(TIMEOUT);
        let received_message_old = received_message.load(Ordering::Relaxed);
        sut_sender.blocking_send(send_data.as_slice()).unwrap();
        t.join().ok();

        assert_that!(received_message_old, eq false);
        assert_that!(received_message.load(Ordering::Relaxed), eq true);
    });
}

#[test]
fn unix_datagram_socket_timeout_blocks_at_least() {
    create_test_directory();
    let socket_name = generate_socket_name();
    let handle = BarrierHandle::new();
    let handle_2 = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let barrier_2 = BarrierBuilder::new(2).create(&handle_2).unwrap();

    thread::scope(|s| {
        let t = s.spawn(|| {
            let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
                .permission(Permission::OWNER_ALL)
                .creation_mode(CreationMode::PurgeAndCreate)
                .create()
                .unwrap();
            barrier.wait();
            barrier_2.wait();

            let mut receive_data: Vec<u8> = vec![0, 0, 0, 0, 0, 0];
            sut_receiver
                .timed_receive(receive_data.as_mut_slice(), TIMEOUT)
                .ok();
        });

        barrier.wait();
        let start = Instant::now();
        barrier_2.wait();

        t.join().ok();

        assert_that!(start.elapsed(), time_at_least TIMEOUT);
    });
}

// TODO iox2-#320
#[ignore]
#[test]
fn unix_datagram_socket_sending_receiving_with_single_fd_works() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let mut test = TestFixture::new();

    create_test_directory();
    let socket_name = generate_socket_name();
    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .permission(Permission::OWNER_ALL)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let mut file_send_content = "itsy bitsy teeny schlurp".to_string();
    let file_sender = test.create_file_with_content(&mut file_send_content);
    let mut msg = SocketAncillary::new();
    assert_that!(msg.add_fd(file_sender.file_descriptor().clone()), eq true);

    sut_sender.try_send_msg(&mut msg).unwrap();

    let mut received_msg = SocketAncillary::new();
    sut_receiver.try_receive_msg(&mut received_msg).unwrap();
    let mut fd_vec = received_msg.extract_fds();
    assert_that!(fd_vec, len 1);

    let mut file_receiver = File::from_file_descriptor(fd_vec.remove(0));
    let mut file_recv_content = String::new();
    file_receiver
        .read_to_string(&mut file_recv_content)
        .unwrap();

    assert_that!(file_recv_content, eq file_send_content);

    file_recv_content = "back to base".to_string();
    file_receiver.truncate(0).unwrap();
    file_receiver
        .write(unsafe { file_recv_content.as_mut_vec() }.as_slice())
        .unwrap();
    file_send_content.clear();
    file_sender.read_to_string(&mut file_send_content).unwrap();

    assert_that!(file_recv_content, eq file_send_content);
}

#[ignore]
#[test]
fn unix_datagram_socket_sending_receiving_credentials_works() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    create_test_directory();
    let socket_name = generate_socket_name();
    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .permission(Permission::OWNER_ALL)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    let send_credentials = SocketCred::new();

    let mut msg = SocketAncillary::new();
    msg.set_creds(&send_credentials);

    sut_sender.blocking_send_msg(&mut msg).unwrap();

    let mut received_msg = SocketAncillary::new();
    sut_receiver.try_receive_msg(&mut received_msg).unwrap();
    let recv_credentials = received_msg.get_creds();
    assert_that!(recv_credentials, eq Some(send_credentials));
}

// TODO iox2-#320
#[ignore]
#[test]
fn unix_datagram_socket_sending_receiving_with_max_supported_fd_and_credentials_works() {
    test_requires!(POSIX_SUPPORT_UNIX_DATAGRAM_SOCKETS_ANCILLARY_DATA);

    let mut test = TestFixture::new();

    create_test_directory();
    let socket_name = generate_socket_name();
    let sut_receiver = UnixDatagramReceiverBuilder::new(&socket_name)
        .permission(Permission::OWNER_ALL)
        .creation_mode(CreationMode::PurgeAndCreate)
        .create()
        .unwrap();

    let sut_sender = UnixDatagramSenderBuilder::new(&socket_name)
        .create()
        .unwrap();

    const NUMBER_OF_FILES: usize = MAX_FILE_DESCRIPTORS_PER_MESSAGE;
    let mut file_send_content: Vec<String> = vec![];
    let mut file_sender: Vec<File> = vec![];
    let mut msg = SocketAncillary::new();

    for i in 0..NUMBER_OF_FILES {
        file_send_content.push(i.to_string() + "bla blubb fuu");
        file_sender.push(test.create_file_with_content(&mut file_send_content[i]));
        assert_that!(msg.add_fd(file_sender[i].file_descriptor().clone()), eq true);
    }

    let send_credentials = SocketCred::new();
    msg.set_creds(&send_credentials);

    sut_sender.try_send_msg(&mut msg).unwrap();

    let mut received_msg = SocketAncillary::new();
    sut_receiver.try_receive_msg(&mut received_msg).unwrap();

    let recv_credentials = received_msg.get_creds();
    assert_that!(recv_credentials, eq Some(send_credentials));

    let mut fd_vec = received_msg.extract_fds();
    assert_that!(fd_vec, len NUMBER_OF_FILES);

    for i in 0..NUMBER_OF_FILES {
        let file_receiver = File::from_file_descriptor(fd_vec.remove(0));
        let mut file_recv_content = String::new();
        file_receiver
            .read_to_string(&mut file_recv_content)
            .unwrap();

        assert_that!(file_recv_content, eq file_send_content[i]);
    }
}
