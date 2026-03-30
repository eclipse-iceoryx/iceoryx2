// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use alloc::vec;
use alloc::vec::Vec;
use core::time::Duration;
use iceoryx2_bb_concurrency::atomic::{AtomicUsize, Ordering};
use iceoryx2_bb_posix::barrier::*;
use iceoryx2_bb_posix::clock::{nanosleep, Time};
use iceoryx2_bb_posix::socket_pair::*;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

const TIMEOUT: Duration = Duration::from_millis(50);

#[test]
pub fn try_receive_never_blocks() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let zeros = vec![0; 10];
    let mut received_data = vec![0; zeros.len()];
    let result = sut_lhs.try_receive(&mut received_data).unwrap();
    assert_that!(result, eq 0);
    assert_that!(received_data, eq zeros);

    let result = sut_rhs.try_receive(&mut received_data).unwrap();
    assert_that!(result, eq 0);
    assert_that!(received_data, eq zeros);
}

#[test]
pub fn send_receive_works() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let send_data = Vec::from(b"!hello hypnotoad!");

    let result = sut_lhs.try_send(&send_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());

    let mut received_data = vec![0; send_data.len()];
    let result = sut_rhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());
    assert_that!(send_data, eq received_data);
}

#[test]
pub fn bidirectional_send_receive_works() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let send_data = Vec::from(b"hello, is it me you're looking for");

    // lhs -> rhs
    let result = sut_lhs.try_send(&send_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());

    let mut received_data = vec![0; send_data.len()];
    let result = sut_rhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());
    assert_that!(send_data, eq received_data);

    // rhs -> lhs
    let send_data = Vec::from(b"I can see it in your bits that you are just a test");
    let result = sut_rhs.try_send(&send_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());

    let mut received_data = vec![0; send_data.len()];
    let result = sut_lhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());
    assert_that!(send_data, eq received_data);
}

#[test]
pub fn cannot_receive_my_own_data() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let send_data_lhs = Vec::from(b"its a dirdy birdy");
    let send_data_rhs = Vec::from(b"meow");

    sut_lhs.try_send(&send_data_lhs).unwrap();
    sut_rhs.try_send(&send_data_rhs).unwrap();

    let mut received_data = vec![0; send_data_rhs.len()];
    let result = sut_lhs.try_receive(&mut received_data).unwrap();
    assert_that!(result, eq send_data_rhs.len());
    assert_that!(send_data_rhs, eq received_data);
    let result = sut_lhs.try_receive(&mut received_data).unwrap();
    assert_that!(result, eq 0);

    let mut received_data = vec![0; send_data_lhs.len()];
    let result = sut_rhs.try_receive(&mut received_data).unwrap();
    assert_that!(result, eq send_data_lhs.len());
    assert_that!(send_data_lhs, eq received_data);
    let result = sut_rhs.try_receive(&mut received_data).unwrap();
    assert_that!(result, eq 0);
}

#[test]
pub fn timed_receive_blocks_for_at_least_timeout() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let mut received_data = vec![0; 10];

    let start = Time::now().unwrap();
    let result = sut_lhs.timed_receive(&mut received_data, TIMEOUT).unwrap();
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
    assert_that!(result, eq 0);

    let start = Time::now().unwrap();
    let result = sut_rhs.timed_receive(&mut received_data, TIMEOUT).unwrap();
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
    assert_that!(result, eq 0);
}

#[test]
pub fn timed_receive_blocks_until_message_arrives() {
    let _watchdog = Watchdog::new();

    let counter = AtomicUsize::new(0);
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let send_message = Vec::from(b"are you in a deadlock - call Ted Krabovsky");
    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();
    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut buffer = vec![0; send_message.len()];
                barrier.wait();
                let result = sut_rhs.timed_receive(&mut buffer, TIMEOUT * 1000).unwrap();
                counter.store(1, Ordering::Relaxed);
                assert_that!(result, eq send_message.len());
                assert_that!(buffer, eq send_message);
            })
            .expect("failed to spawn thread");

        barrier.wait();
        nanosleep(TIMEOUT).expect("failed to sleep");
        assert_that!(counter.load(Ordering::Relaxed), eq 0);
        let result = sut_lhs.try_send(&send_message).unwrap();
        assert_that!(result, eq send_message.len());

        Ok(())
    })
    .expect("failed to execute thread scope");
}

#[test]
pub fn blocking_receive_blocks_until_message_arrives() {
    let _watchdog = Watchdog::new();

    let counter = AtomicUsize::new(0);
    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let send_message = Vec::from(b"are you in a deadlock - call Ted Krabovsky");
    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();
    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                let mut buffer = vec![0; send_message.len()];
                barrier.wait();
                let result = sut_rhs.blocking_receive(&mut buffer).unwrap();
                counter.store(1, Ordering::Relaxed);
                assert_that!(result, eq send_message.len());
                assert_that!(buffer, eq send_message);
            })
            .expect("failed to spawn thread");

        barrier.wait();
        nanosleep(TIMEOUT).expect("failed to sleep");
        assert_that!(counter.load(Ordering::Relaxed), eq 0);
        let result = sut_lhs.try_send(&send_message).unwrap();
        assert_that!(result, eq send_message.len());

        Ok(())
    })
    .expect("failed to execute thread scope");
}

#[test]
pub fn timed_send_blocks_for_at_least_timeout() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, _sut_rhs) = StreamingSocket::create_pair().unwrap();

    let mut send_data = vec![];
    send_data.resize(128, 55);

    loop {
        let result = sut_lhs.try_send(&send_data).unwrap();
        if result != send_data.len() {
            break;
        }
    }

    let start = Time::now().unwrap();
    let result = sut_lhs.timed_send(&send_data, TIMEOUT).unwrap();
    assert_that!(start.elapsed().unwrap(), time_at_least TIMEOUT);
    assert_that!(result, eq 0);
}

#[test]
pub fn timed_send_blocks_until_message_buffer_is_free_again() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let send_data = Vec::from(b"Q");
    let mut number_of_data_sent = 0;

    loop {
        let result = sut_lhs.try_send(&send_data).unwrap();
        if result != send_data.len() {
            break;
        }
        number_of_data_sent += 1;
    }

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let counter = AtomicUsize::new(0);
    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                barrier.wait();
                let result = sut_lhs.timed_send(&send_data, TIMEOUT * 100).unwrap();
                counter.store(1, Ordering::Relaxed);
                assert_that!(result, eq send_data.len());
            })
            .expect("failed to spawn thread");

        let mut receive_buffer = vec![0; number_of_data_sent];

        barrier.wait();
        nanosleep(TIMEOUT).expect("failed to sleep");
        assert_that!(counter.load(Ordering::Relaxed), eq 0);
        let result = sut_rhs.try_receive(&mut receive_buffer).unwrap();
        assert_that!(result, eq number_of_data_sent);
        for byte in receive_buffer {
            assert_that!(byte, eq b'Q');
        }

        Ok(())
    })
    .expect("failed to execute thread scope");
}

#[test]
pub fn blocking_send_blocks_until_message_buffer_is_free_again() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let send_data = Vec::from(b"X");
    let mut number_of_data_sent = 0;

    loop {
        let result = sut_lhs.try_send(&send_data).unwrap();
        if result != send_data.len() {
            break;
        }
        number_of_data_sent += 1;
    }

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let counter = AtomicUsize::new(0);
    thread_scope(|s| {
        s.thread_builder()
            .spawn(|| {
                barrier.wait();
                let result = sut_lhs.blocking_send(&send_data).unwrap();
                counter.store(1, Ordering::Relaxed);
                assert_that!(result, eq send_data.len());
            })
            .expect("failed to spawn thread");

        let mut receive_buffer = vec![0; number_of_data_sent];

        barrier.wait();
        nanosleep(TIMEOUT).expect("failed to sleep");
        assert_that!(counter.load(Ordering::Relaxed), eq 0);
        let result = sut_rhs.try_receive(&mut receive_buffer).unwrap();
        assert_that!(result, eq number_of_data_sent);

        for byte in receive_buffer {
            assert_that!(byte, eq b'X');
        }

        Ok(())
    })
    .expect("failed to execute thread scope");
}

#[test]
pub fn peeking_message_does_not_remove_message() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();

    let send_data = Vec::from(b"get schwifty!");

    let result = sut_lhs.try_send(&send_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());

    for _ in 0..5 {
        let mut received_data = vec![0; send_data.len()];
        let result = sut_rhs.peek(&mut received_data);
        assert_that!(result, is_ok);
        assert_that!(result.unwrap(), eq send_data.len());
        assert_that!(send_data, eq received_data);
    }

    let mut received_data = vec![0; send_data.len()];
    let result = sut_rhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());
    assert_that!(send_data, eq received_data);
}

#[test]
pub fn send_from_duplicated_socket_works() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();
    let sut_lhs_dup = sut_lhs.duplicate().unwrap();

    let send_data = Vec::from(b"!hello hypnotoad!");

    let result = sut_lhs_dup.try_send(&send_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());

    let mut received_data = vec![0; send_data.len()];
    let result = sut_rhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());
    assert_that!(send_data, eq received_data);
}

#[test]
pub fn receive_from_duplicated_socket_works() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();
    let sut_rhs_dup = sut_rhs.duplicate().unwrap();

    let send_data = Vec::from(b"!hello hypnotoad!");

    let result = sut_lhs.try_send(&send_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());

    let mut received_data = vec![0; send_data.len()];
    let result = sut_rhs_dup.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data.len());
    assert_that!(send_data, eq received_data);
}

#[test]
pub fn multiple_duplicated_sockets_can_send() {
    let _watchdog = Watchdog::new();

    let (sut_lhs, sut_rhs) = StreamingSocket::create_pair().unwrap();
    let sut_lhs_dup_1 = sut_lhs.duplicate().unwrap();
    let sut_lhs_dup_2 = sut_lhs.duplicate().unwrap();

    let send_data_1 = Vec::from(b"!1!");
    let send_data_2 = Vec::from(b"!2!");
    let send_data_3 = Vec::from(b"!3!");

    let result = sut_lhs.try_send(&send_data_1);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data_1.len());

    let result = sut_lhs_dup_1.try_send(&send_data_2);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data_2.len());

    let result = sut_lhs_dup_2.try_send(&send_data_3);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data_3.len());

    let mut received_data = vec![0; send_data_1.len()];

    let result = sut_rhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data_1.len());
    assert_that!(send_data_1, eq received_data);

    let result = sut_rhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data_2.len());
    assert_that!(send_data_2, eq received_data);

    let result = sut_rhs.try_receive(&mut received_data);
    assert_that!(result, is_ok);
    assert_that!(result.unwrap(), eq send_data_3.len());
    assert_that!(send_data_3, eq received_data);
}
