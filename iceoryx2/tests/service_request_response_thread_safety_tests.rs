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

use std::sync::Barrier;

use iceoryx2::prelude::*;
use iceoryx2::testing::*;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;

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
    let barrier = Barrier::new(number_of_client_threads + 1);

    std::thread::scope(|s| {
        for _ in 0..number_of_client_threads {
            s.spawn(|| {
                barrier.wait();
                for n in 0..NUMBER_OF_ITERATIONS {
                    let mut request = client.loan().unwrap();
                    *request = n;
                    request.send().unwrap();
                }
            });
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
    });
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
    let barrier = Barrier::new(number_of_server_threads);

    for n in 0..NUMBER_OF_ITERATIONS {
        client.send_copy(n).unwrap();
    }

    std::thread::scope(|s| {
        let mut server_threads = vec![];
        for _ in 0..number_of_server_threads {
            server_threads.push(s.spawn(|| {
                let mut received_requests = [0; NUMBER_OF_ITERATIONS];
                barrier.wait();
                while let Ok(Some(request)) = server.receive() {
                    received_requests[*request] += 1;
                }

                received_requests
            }));
        }

        let mut received_requests = [0; NUMBER_OF_ITERATIONS];
        for t in server_threads {
            let requests = t.join().unwrap();
            for (n, count) in requests.iter().enumerate() {
                received_requests[n] += count;
            }
        }

        for n in received_requests {
            assert_that!(n, eq 1);
        }
    });
}
