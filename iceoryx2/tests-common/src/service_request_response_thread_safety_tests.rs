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

use alloc::vec;
use alloc::vec::Vec;

use iceoryx2::prelude::*;
use iceoryx2::testing::*;
use iceoryx2_bb_posix::barrier::BarrierBuilder;
use iceoryx2_bb_posix::barrier::BarrierHandle;
use iceoryx2_bb_posix::mutex::Handle;
use iceoryx2_bb_posix::mutex::MutexBuilder;
use iceoryx2_bb_posix::mutex::MutexHandle;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

#[test]
fn loaning_and_sending_requests_concurrently_works() {
    let _watchdog = Watchdog::new();
    type ServiceType = ipc_threadsafe::Service;
    let service_name = generate_service_name();
    let config = generate_isolated_config();
    const NUMBER_OF_ITERATIONS: usize = 500;
    let number_of_client_threads: usize = SystemInfo::NumberOfCpuCores.value().min(2);

    let node = NodeBuilder::new()
        .config(&config)
        .create::<ServiceType>()
        .unwrap();
    let service = node
        .service_builder(&service_name)
        .request_response::<usize, usize>()
        .max_servers(1)
        .max_clients(1)
        .enable_fire_and_forget_requests(true)
        .max_active_requests_per_client(number_of_client_threads * NUMBER_OF_ITERATIONS)
        .create()
        .unwrap();
    let client = service.client_builder().create().unwrap();
    let server = service.server_builder().create().unwrap();

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new((number_of_client_threads + 1) as u32)
        .create(&barrier_handle)
        .unwrap();

    thread_scope(|s| {
        for _ in 0..number_of_client_threads {
            s.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    for n in 0..NUMBER_OF_ITERATIONS {
                        let mut request = client.loan().unwrap();
                        *request = n;
                        request.send().unwrap();
                    }
                })
                .expect("failed to spawn thread");
        }
        barrier.wait();

        let mut total_received_request = 0;
        let mut received_request = [0; NUMBER_OF_ITERATIONS];
        while total_received_request < number_of_client_threads * NUMBER_OF_ITERATIONS {
            if let Ok(Some(request)) = server.receive() {
                received_request[*request] += 1;
                total_received_request += 1;
            }
        }

        for n in received_request {
            assert_that!(n, eq number_of_client_threads);
        }

        Ok(())
    })
    .expect("failed to spawn thread");
}

#[test]
fn receiving_requests_concurrently_works() {
    let _watchdog = Watchdog::new();
    type ServiceType = ipc_threadsafe::Service;
    let service_name = generate_service_name();
    let config = generate_isolated_config();
    let number_of_server_threads: usize = SystemInfo::NumberOfCpuCores.value().min(2);
    const NUMBER_OF_ITERATIONS: usize = 500;

    let node = NodeBuilder::new()
        .config(&config)
        .create::<ServiceType>()
        .unwrap();
    let service = node
        .service_builder(&service_name)
        .request_response::<usize, usize>()
        .max_servers(1)
        .max_clients(1)
        .enable_fire_and_forget_requests(true)
        .max_active_requests_per_client(number_of_server_threads * NUMBER_OF_ITERATIONS)
        .create()
        .unwrap();
    let server = service.server_builder().create().unwrap();
    let client = service.client_builder().create().unwrap();

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new((number_of_server_threads + 1) as u32)
        .create(&barrier_handle)
        .unwrap();

    for n in 0..NUMBER_OF_ITERATIONS {
        client.send_copy(n).unwrap();
    }

    let all_requests_handle = MutexHandle::<Vec<usize>>::new();
    let all_requests = MutexBuilder::new()
        .create(vec![0usize; NUMBER_OF_ITERATIONS], &all_requests_handle)
        .expect("failed to create mutex");

    thread_scope(|s| {
        for _ in 0..number_of_server_threads {
            s.thread_builder()
                .spawn(|| {
                    let mut received_requests = vec![0usize; NUMBER_OF_ITERATIONS];
                    barrier.wait();
                    while let Ok(Some(request)) = server.receive() {
                        received_requests[*request] += 1;
                    }

                    let mut guard = all_requests.lock().unwrap();
                    for (i, count) in received_requests.iter().enumerate() {
                        guard[i] += count;
                    }
                })
                .expect("failed to spawn thread");
        }

        barrier.wait();

        Ok(())
    })
    .expect("failed to spawn thread");

    let guard = all_requests.lock().unwrap();
    for n in guard.iter() {
        assert_that!(*n, eq 1);
    }
}
