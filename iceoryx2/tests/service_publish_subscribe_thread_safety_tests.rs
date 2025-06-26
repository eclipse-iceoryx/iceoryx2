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
fn loaning_and_sending_samples_concurrently_works() {
    let _watchdog = Watchdog::new();
    type ServiceType = ipc_threadsafe::Service;
    let service_name = generate_service_name();
    let config = generate_isolated_config();
    const NUMBER_OF_ITERATIONS: usize = 2000;
    let number_of_publisher_threads: usize = SystemInfo::NumberOfCpuCores.value().min(2);

    let node = NodeBuilder::new()
        .config(&config)
        .create::<ServiceType>()
        .unwrap();
    let service = node
        .service_builder(&service_name)
        .publish_subscribe::<usize>()
        .max_publishers(1)
        .max_subscribers(1)
        .subscriber_max_buffer_size(number_of_publisher_threads * NUMBER_OF_ITERATIONS)
        .create()
        .unwrap();
    let publisher = service
        .publisher_builder()
        .max_loaned_samples(number_of_publisher_threads + 1)
        .create()
        .unwrap();
    let subscriber = service.subscriber_builder().create().unwrap();
    let barrier = Barrier::new(number_of_publisher_threads + 1);

    std::thread::scope(|s| {
        for _ in 0..number_of_publisher_threads {
            s.spawn(|| {
                barrier.wait();
                for n in 0..NUMBER_OF_ITERATIONS {
                    let mut sample = publisher.loan().unwrap();
                    *sample = n;
                    sample.send().unwrap();
                }
            });
        }
        barrier.wait();

        let mut total_received_samples = 0;
        let mut received_samples = [0; NUMBER_OF_ITERATIONS];
        while total_received_samples < number_of_publisher_threads * NUMBER_OF_ITERATIONS {
            if let Ok(Some(sample)) = subscriber.receive() {
                received_samples[*sample] += 1;
                total_received_samples += 1;
            }
        }

        for n in received_samples {
            assert_that!(n, eq number_of_publisher_threads);
        }
    });
}

#[test]
fn receiving_samples_concurrently_works() {
    let _watchdog = Watchdog::new();
    type ServiceType = ipc_threadsafe::Service;
    let service_name = generate_service_name();
    let config = generate_isolated_config();
    let number_of_subscriber_threads: usize = SystemInfo::NumberOfCpuCores.value().min(2);
    const NUMBER_OF_ITERATIONS: usize = 2000;

    let node = NodeBuilder::new()
        .config(&config)
        .create::<ServiceType>()
        .unwrap();
    let service = node
        .service_builder(&service_name)
        .publish_subscribe::<usize>()
        .max_publishers(1)
        .max_subscribers(1)
        .subscriber_max_buffer_size(number_of_subscriber_threads * NUMBER_OF_ITERATIONS)
        .create()
        .unwrap();
    let publisher = service.publisher_builder().create().unwrap();
    let subscriber = service.subscriber_builder().create().unwrap();
    let barrier = Barrier::new(number_of_subscriber_threads);

    for n in 0..NUMBER_OF_ITERATIONS {
        publisher.send_copy(n).unwrap();
    }

    std::thread::scope(|s| {
        let mut subscriber_threads = vec![];
        for _ in 0..number_of_subscriber_threads {
            subscriber_threads.push(s.spawn(|| {
                let mut received_samples = [0; NUMBER_OF_ITERATIONS];
                barrier.wait();
                while let Ok(Some(sample)) = subscriber.receive() {
                    received_samples[*sample] += 1;
                }

                received_samples
            }));
        }

        let mut received_samples = [0; NUMBER_OF_ITERATIONS];
        for t in subscriber_threads {
            let samples = t.join().unwrap();
            for (n, count) in samples.iter().enumerate() {
                received_samples[n] += count;
            }
        }

        for n in received_samples {
            assert_that!(n, eq 1);
        }
    });
}
