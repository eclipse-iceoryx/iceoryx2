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

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use iceoryx2_bb_container::semantic_string::SemanticString;
use iceoryx2_bb_posix::{
    barrier::{BarrierBuilder, BarrierHandle},
    clock::Time,
    message_queue::*,
    semaphore::ClockType,
    unique_system_id::UniqueSystemId,
    unix_datagram_socket::CreationMode,
};
use iceoryx2_bb_system_types::file_name::FileName;
use iceoryx2_bb_testing::{assert_that, test_requires};
use iceoryx2_pal_posix::posix::POSIX_SUPPORT_MESSAGE_QUEUE;

const TIMEOUT: Duration = Duration::from_millis(25);

fn generate_mq_name() -> FileName {
    let mut file = FileName::new(b"message_queue_tests_").unwrap();
    file.push_bytes(
        UniqueSystemId::new()
            .unwrap()
            .value()
            .to_string()
            .as_bytes(),
    )
    .unwrap();
    file
}

#[test]
fn message_queue_creating_queue_works() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let sut_sender = MessageQueueBuilder::new(&name)
        .capacity(3)
        .clock_type(ClockType::Realtime)
        .create_sender::<u8>(CreationMode::PurgeAndCreate)
        .unwrap();

    assert_that!(sut_sender.name(), eq & name);
    assert_that!(sut_sender.capacity(), eq 3);
    assert_that!(sut_sender.clock_type(), eq ClockType::Realtime);
    assert_that!(sut_sender, len 0);
}

#[test]
fn message_queue_creating_queue_twice_fails() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let _sut_sender =
        MessageQueueBuilder::new(&name).create_sender::<u8>(CreationMode::PurgeAndCreate);

    let sut = MessageQueueBuilder::new(&name).create_sender::<u8>(CreationMode::CreateExclusive);

    assert_that!(sut.err().unwrap(), eq MessageQueueCreationError::AlreadyExist);
}

#[test]
fn message_queue_does_exist_works() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();

    assert_that!(does_message_queue_exist::<u8>(&name), eq Ok(false));

    let _sut_sender =
        MessageQueueBuilder::new(&name).create_sender::<u8>(CreationMode::PurgeAndCreate);

    assert_that!(does_message_queue_exist::<u8>(&name), eq Ok(true));
}

#[test]
fn message_queue_open_queue_works() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let _sender = MessageQueueBuilder::new(&name)
        .capacity(8)
        .create_sender::<u32>(CreationMode::PurgeAndCreate)
        .unwrap();

    let sut = MessageQueueBuilder::new(&name)
        .capacity(5)
        .open_receiver::<u32>()
        .unwrap();

    assert_that!(sut.name(), eq & name);
    assert_that!(sut.capacity(), eq 8);
    assert_that!(sut, len 0);
}

#[test]
fn message_queue_open_non_existing_queue_fails() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let sut = MessageQueueBuilder::new(&name)
        .capacity(5)
        .open_receiver::<u32>();

    assert_that!(sut.err().unwrap(), eq MessageQueueOpenError::DoesNotExist);
}

#[test]
fn message_queue_open_fails_when_capacity_requirement_not_satisfied() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let _sender = MessageQueueBuilder::new(&name)
        .capacity(2)
        .create_sender::<u32>(CreationMode::PurgeAndCreate)
        .unwrap();

    let sut = MessageQueueBuilder::new(&name)
        .capacity(5)
        .open_receiver::<u32>();

    assert_that!(
        sut.err().unwrap(), eq
        MessageQueueOpenError::CapacitySmallerThanRequired
    );
}

#[test]
fn message_queue_open_fails_when_message_size_requirement_not_satisfied() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let _sender = MessageQueueBuilder::new(&name)
        .capacity(8)
        .create_sender::<u8>(CreationMode::PurgeAndCreate)
        .unwrap();

    let sut = MessageQueueBuilder::new(&name)
        .capacity(5)
        .open_receiver::<u64>();

    assert_that!(
        sut.err().unwrap(), eq
        MessageQueueOpenError::MessageSizeDoesNotFit
    );
}

#[test]
fn message_queue_try_send_receive_works() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut_sender = MessageQueueBuilder::new(&name)
        .create_duplex::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    let mut sut_receiver = MessageQueueBuilder::new(&name)
        .open_duplex::<usize>()
        .unwrap();

    for i in 0..sut_sender.capacity() {
        sut_sender.try_send(&i).unwrap();
        assert_that!(sut_receiver, len i + 1);
    }

    for i in 0..sut_sender.capacity() {
        let received = sut_receiver.try_receive().unwrap();
        assert_that!(received, is_some);
        assert_that!(received.unwrap().value, eq i);
        assert_that!(sut_receiver, len sut_sender.capacity() - i - 1);
    }
}

#[test]
fn message_queue_try_send_receive_with_prio_works() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut_sender = MessageQueueBuilder::new(&name)
        .create_duplex::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    let mut sut_receiver = MessageQueueBuilder::new(&name)
        .open_duplex::<usize>()
        .unwrap();

    for i in 0..sut_sender.capacity() {
        sut_sender
            .try_send_with_prio(&i, (2 * i + 1) as u32)
            .unwrap();
        assert_that!(sut_receiver, len i + 1);
    }

    for i in 0..sut_sender.capacity() {
        let received = sut_receiver.try_receive().unwrap();
        assert_that!(received, is_some);
        assert_that!(
            received.as_ref().unwrap().value, eq
            sut_sender.capacity() - i - 1
        );
        assert_that!(
            received.as_ref().unwrap().priority,
            eq(2 * (sut_sender.capacity() - i - 1) + 1) as u32
        );
        assert_that!(sut_receiver, len sut_sender.capacity() - i - 1);
    }
}

#[test]
fn message_queue_try_send_does_not_block() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut_sender = MessageQueueBuilder::new(&name)
        .capacity(1)
        .create_sender::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    assert_that!(sut_sender.try_send(&12893).unwrap(), eq true);
    assert_that!(sut_sender.try_send(&12893).unwrap(), eq false);
}

#[test]
fn message_queue_try_receive_does_not_block() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut_receiver = MessageQueueBuilder::new(&name)
        .capacity(1)
        .create_receiver::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    assert_that!(sut_receiver.try_receive().unwrap(), is_none);
}

#[test]
fn message_queue_blocking_send_does_block() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let counter = AtomicU64::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut sut_sender = MessageQueueBuilder::new(&name)
                .capacity(1)
                .create_sender::<usize>(CreationMode::PurgeAndCreate)
                .unwrap();

            let try_send_result = sut_sender.try_send(&12893);
            barrier.wait();
            sut_sender.blocking_send(&12893).unwrap();
            counter.store(1, std::sync::atomic::Ordering::SeqCst);

            assert_that!(try_send_result, is_ok);
            assert_that!(try_send_result.unwrap(), eq true);
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::SeqCst);
        let mut sut_receiver = MessageQueueBuilder::new(&name)
            .capacity(1)
            .open_receiver::<usize>()
            .unwrap();
        assert_that!(counter_old, eq 0);
        assert_that!(sut_receiver.try_receive().unwrap().unwrap().value, eq 12893);
    });
}

#[test]
fn message_queue_blocking_receive_does_block() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let counter = AtomicU64::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut sut_receiver = MessageQueueBuilder::new(&name)
                .create_duplex::<usize>(CreationMode::PurgeAndCreate)
                .unwrap();

            barrier.wait();
            let receive_result = sut_receiver.blocking_receive();
            counter.store(1, std::sync::atomic::Ordering::SeqCst);
            assert_that!(receive_result.unwrap().value, eq 981293);
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::SeqCst);
        let mut sut_sender = MessageQueueBuilder::new(&name)
            .open_duplex::<usize>()
            .unwrap();
        assert_that!(counter_old, eq 0);
        assert_that!(sut_sender.try_send(&981293).unwrap(), eq true);
    });
}

#[test]
fn message_queue_blocking_timed_send_does_block() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let counter = AtomicU64::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut sut_sender = MessageQueueBuilder::new(&name)
                .capacity(1)
                .create_sender::<usize>(CreationMode::PurgeAndCreate)
                .unwrap();

            let try_send_result = sut_sender.try_send(&12893);
            barrier.wait();
            let timed_send_result = sut_sender.timed_send(&12893, TIMEOUT * 10);
            counter.store(1, std::sync::atomic::Ordering::SeqCst);

            assert_that!(try_send_result.unwrap(), eq true);
            assert_that!(timed_send_result.unwrap(), eq true);
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::SeqCst);
        let mut sut_receiver = MessageQueueBuilder::new(&name)
            .capacity(1)
            .open_receiver::<usize>()
            .unwrap();
        assert_that!(counter_old, eq 0);
        assert_that!(sut_receiver.try_receive().unwrap().unwrap().value, eq 12893);
    });
}

#[test]
fn message_queue_timed_receive_does_block() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&handle).unwrap();
    let counter = AtomicU64::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut sut_receiver = MessageQueueBuilder::new(&name)
                .create_duplex::<usize>(CreationMode::PurgeAndCreate)
                .unwrap();

            barrier.wait();
            let timed_receive_result = sut_receiver.timed_receive(TIMEOUT * 10);
            counter.store(1, std::sync::atomic::Ordering::SeqCst);
            assert_that!(timed_receive_result.unwrap().unwrap().value, eq 981293);
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::SeqCst);
        let mut sut_sender = MessageQueueBuilder::new(&name)
            .open_duplex::<usize>()
            .unwrap();
        assert_that!(counter_old, eq 0);
        assert_that!(sut_sender.try_send(&981293).unwrap(), eq true);
    });
}

#[test]
fn message_queue_timed_send_returns_false_on_timeout() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut = MessageQueueBuilder::new(&name)
        .capacity(1)
        .create_duplex::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    sut.try_send(&123).unwrap();
    assert_that!(!sut.timed_send(&123, TIMEOUT).unwrap(), eq true);
}

#[test]
fn message_queue_timed_receive_returns_none_on_timeout() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut = MessageQueueBuilder::new(&name)
        .create_duplex::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    assert_that!(sut.timed_receive(TIMEOUT).unwrap(), is_none);
}

#[test]
fn message_queue_timed_send_waits_at_least_for_timeout() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut = MessageQueueBuilder::new(&name)
        .capacity(1)
        .create_duplex::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    sut.try_send(&123).unwrap();
    let start = Time::now().unwrap();
    assert_that!(sut.timed_send(&123, TIMEOUT).unwrap(), eq false);
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
}

#[test]
fn message_queue_timed_receive_waits_at_least_for_timeout() {
    test_requires!(POSIX_SUPPORT_MESSAGE_QUEUE);

    let name = generate_mq_name();
    let mut sut = MessageQueueBuilder::new(&name)
        .capacity(1)
        .create_duplex::<usize>(CreationMode::PurgeAndCreate)
        .unwrap();

    let start = Time::now().unwrap();
    assert_that!(sut.timed_receive(TIMEOUT).unwrap(), is_none);
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
}
