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

use core::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use std::time::Instant;

use iceoryx2_bb_posix::{barrier::*, udp_socket::*};
use iceoryx2_bb_system_types::ipv4_address::{self, Ipv4Address};
use iceoryx2_bb_testing::assert_that;

const TIMEOUT: Duration = Duration::from_millis(25);

#[test]
fn udp_socket_send_receive_works() {
    let sut_server = UdpServerBuilder::new().listen().unwrap();
    let sut_client_1 = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();
    let sut_client_2 = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let mut client_buffer_1 = [123u8, 23u8, 1u8, 0u8, 99u8];
    assert_that!(sut_client_1.send(&client_buffer_1), eq Ok(client_buffer_1.len()));

    let client_buffer_2 = [78u8, 123u8, 0u8, 23u8, 21u8, 1u8, 245u8, 0u8, 99u8];
    assert_that!(sut_client_2.send(&client_buffer_2), eq Ok(client_buffer_2.len()));

    let mut recv_buffer = [0u8; 16];
    let recv_details = sut_server
        .blocking_receive_from(&mut recv_buffer)
        .unwrap()
        .unwrap();
    assert_that!(recv_details.number_of_bytes, eq 5);

    for i in 0..client_buffer_1.len() {
        assert_that!(recv_buffer[i], eq client_buffer_1[i]);
    }

    let server_buffer_1 = [9u8, 8u8, 7u8, 6u8];
    assert_that!( sut_server.send_to(
        &server_buffer_1,
        recv_details.source_ip,
        recv_details.source_port,
    ), eq Ok(server_buffer_1.len()));

    assert_that!(sut_client_2.try_receive(&mut client_buffer_1), eq Ok(0));
    assert_that!(sut_client_1.blocking_receive(&mut client_buffer_1), eq Ok(server_buffer_1.len()));

    for i in 0..server_buffer_1.len() {
        assert_that!(server_buffer_1[i], eq client_buffer_1[i]);
    }
}

#[test]
fn udp_socket_server_with_same_address_and_port_fails() {
    let sut_server_1 = UdpServerBuilder::new()
        .address(Ipv4Address::new(127, 0, 0, 1))
        .listen()
        .unwrap();

    let sut_server_2 = UdpServerBuilder::new()
        .address(Ipv4Address::new(127, 0, 0, 1))
        .port(sut_server_1.port())
        .listen();

    assert_that!(sut_server_2.err().unwrap(), eq UdpServerCreateError::AddressAlreadyInUse);
}

#[test]
fn udp_socket_when_socket_goes_out_of_scope_address_is_free_again() {
    let port;
    {
        let sut_server_1 = UdpServerBuilder::new()
            .address(ipv4_address::LOCALHOST)
            .listen()
            .unwrap();
        port = sut_server_1.port();
    }

    let sut_server_2 = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .port(port)
        .listen();

    assert_that!(sut_server_2, is_ok);
}

#[test]
fn udp_socket_server_has_correct_address() {
    let port = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap()
        .port();

    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .port(port)
        .listen()
        .unwrap();

    assert_that!(sut_server.address(), eq ipv4_address::LOCALHOST);
    assert_that!(sut_server.port(), eq port);
}

#[test]
fn udp_socket_client_returns_address_of_server() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    assert_that!(sut_client.address(), eq sut_server.address());
    assert_that!(sut_client.port(), eq sut_server.port());
}

#[test]
fn udp_socket_client_can_send_data_to_server() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let send_buffer = [12u8, 24u8, 36u8];
    assert_that!(sut_client.send(&send_buffer).unwrap(), eq send_buffer.len());

    let mut recv_buffer = [0u8; 8];
    assert_that!(sut_server.blocking_receive_from(&mut recv_buffer).unwrap().unwrap().number_of_bytes, eq send_buffer.len());
}

#[test]
fn udp_socket_server_can_send_data_to_client() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let send_buffer = [12u8, 24u8, 36u8];
    assert_that!(sut_client.send(&send_buffer).unwrap(), eq send_buffer.len());

    let mut recv_buffer = [0u8; 8];
    let client_addr = sut_server
        .blocking_receive_from(&mut recv_buffer)
        .unwrap()
        .unwrap();

    assert_that!(sut_server.send_to(&send_buffer, client_addr.source_ip, client_addr.source_port).unwrap(), eq send_buffer.len());

    let mut recv_buffer = [0u8; 8];
    assert_that!(sut_client.blocking_receive(&mut recv_buffer).unwrap(), eq send_buffer.len());
}

#[test]
fn udp_socket_client_try_receive_does_not_block() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let mut recv_buffer = [0u8; 8];
    assert_that!(sut_client.try_receive(&mut recv_buffer).unwrap(), eq 0);
}

#[test]
fn udp_socket_server_try_receive_from_does_not_block() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let mut recv_buffer = [0u8; 8];
    assert_that!(
        sut_server.try_receive_from(&mut recv_buffer).unwrap(),
        is_none
    );
}

#[test]
fn udp_socket_client_timed_receive_does_block_for_at_least_timeout() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let mut recv_buffer = [0u8; 8];
    let start = Instant::now();
    assert_that!(sut_client.timed_receive(&mut recv_buffer, TIMEOUT).unwrap(), eq 0);
    assert_that!(start.elapsed(), time_at_least TIMEOUT);
}

#[test]
fn udp_socket_server_timed_receive_from_does_block_for_at_least_timeout() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let mut recv_buffer = [0u8; 8];
    let start = Instant::now();
    assert_that!(
        sut_server
            .timed_receive_from(&mut recv_buffer, TIMEOUT)
            .unwrap(),
        is_none
    );
    assert_that!(start.elapsed(), time_at_least TIMEOUT);
}

#[test]
fn udp_socket_client_blocking_receive_does_block() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let send_buffer = [12u8, 24u8, 36u8];
    assert_that!(sut_client.send(&send_buffer).unwrap(), eq send_buffer.len());

    let mut recv_buffer = [0u8; 8];
    let client_addr = sut_server
        .blocking_receive_from(&mut recv_buffer)
        .unwrap()
        .unwrap();

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let counter = AtomicU64::new(0);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            let mut recv_buffer = [0u8; 8];
            let receive_result = sut_client.blocking_receive(&mut recv_buffer);
            counter.store(1, Ordering::Relaxed);
            assert_that!(receive_result.unwrap(), eq 3);
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        let send_result =
            sut_server.send_to(&send_buffer, client_addr.source_ip, client_addr.source_port);

        assert_that!(t1.join(), is_ok);
        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
        assert_that!(send_result, is_ok);
    });
}

#[test]
fn udp_socket_server_blocking_receive_from_does_block() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let counter = AtomicU64::new(0);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            let mut recv_buffer = [0u8; 8];
            let receive_result = sut_server.blocking_receive_from(&mut recv_buffer);
            counter.store(1, Ordering::Relaxed);
            assert_that!(receive_result.unwrap(), is_some);
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        let send_buffer = [12u8, 24u8, 36u8];
        let send_result = sut_client.send(&send_buffer);

        assert_that!(t1.join(), is_ok);
        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
        assert_that!(send_result, is_ok);
    });
}

#[test]
fn udp_socket_client_timed_receive_does_blocks() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let send_buffer = [12u8, 24u8, 36u8];
    assert_that!(sut_client.send(&send_buffer).unwrap(), eq send_buffer.len());

    let mut recv_buffer = [0u8; 8];
    let client_addr = sut_server
        .blocking_receive_from(&mut recv_buffer)
        .unwrap()
        .unwrap();

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let counter = AtomicU64::new(0);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            let mut recv_buffer = [0u8; 8];
            let timed_receive_result = sut_client.timed_receive(&mut recv_buffer, TIMEOUT * 100);
            counter.store(1, Ordering::Relaxed);
            assert_that!(timed_receive_result.unwrap(), eq 3);
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        let send_result =
            sut_server.send_to(&send_buffer, client_addr.source_ip, client_addr.source_port);

        assert_that!(t1.join(), is_ok);
        assert_that!(counter_old, eq 0);
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
        assert_that!(send_result, is_ok);
    });
}

#[test]
fn udp_socket_server_timed_receive_from_does_block() {
    let sut_server = UdpServerBuilder::new()
        .address(ipv4_address::LOCALHOST)
        .listen()
        .unwrap();

    let sut_client = UdpClientBuilder::new(ipv4_address::LOCALHOST)
        .connect_to(sut_server.port())
        .unwrap();

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new(2).create(&barrier_handle).unwrap();
    let counter = AtomicU64::new(0);

    std::thread::scope(|s| {
        let t1 = s.spawn(|| {
            barrier.wait();
            let mut recv_buffer = [0u8; 8];
            let timed_receive_result =
                sut_server.timed_receive_from(&mut recv_buffer, TIMEOUT * 100);
            counter.store(1, Ordering::Relaxed);
            assert_that!(timed_receive_result.unwrap(), is_some)
        });

        barrier.wait();
        std::thread::sleep(TIMEOUT);
        let counter_old = counter.load(Ordering::Relaxed);
        let send_buffer = [12u8, 24u8, 36u8];
        let send_result = sut_client.send(&send_buffer);

        assert_that!(t1.join(), is_ok);
        assert_that!(counter.load(Ordering::Relaxed), eq 1);
        assert_that!(counter_old, eq 0);
        assert_that!(send_result, is_ok);
    });
}
