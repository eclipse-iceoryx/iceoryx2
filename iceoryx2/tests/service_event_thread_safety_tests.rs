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

use core::sync::atomic::Ordering;
use std::sync::Barrier;

use iceoryx2::prelude::*;
use iceoryx2::testing::*;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicBool;
use iceoryx2_pal_concurrency_sync::iox_atomic::IoxAtomicUsize;

#[test]
fn notifying_events_concurrently_works() {
    let _watchdog = Watchdog::new();
    type ServiceType = ipc_threadsafe::Service;
    let service_name = generate_service_name();
    let config = generate_isolated_config();
    const NUMBER_OF_ITERATIONS: usize = 2000;
    let number_of_notifier_threads: usize = SystemInfo::NumberOfCpuCores.value().min(2);

    let node = NodeBuilder::new()
        .config(&config)
        .create::<ServiceType>()
        .unwrap();
    let service = node
        .service_builder(&service_name)
        .event()
        .max_notifiers(1)
        .max_listeners(1)
        .event_id_max_value(NUMBER_OF_ITERATIONS)
        .create()
        .unwrap();
    let notifier = service.notifier_builder().create().unwrap();
    let listener = service.listener_builder().create().unwrap();
    let barrier = Barrier::new(number_of_notifier_threads + 1);

    let number_of_finished_notifier_threads = IoxAtomicUsize::new(0);
    std::thread::scope(|s| {
        for _ in 0..number_of_notifier_threads {
            s.spawn(|| {
                barrier.wait();
                for n in 0..NUMBER_OF_ITERATIONS {
                    while notifier
                        .notify_with_custom_event_id(EventId::new(n))
                        .unwrap()
                        == 0
                    {}
                }
                number_of_finished_notifier_threads.fetch_add(1, Ordering::Relaxed);
            });
        }
        barrier.wait();

        let mut total_received_events = 0;
        let mut received_events = [0; NUMBER_OF_ITERATIONS];
        while total_received_events < number_of_notifier_threads * NUMBER_OF_ITERATIONS {
            if let Ok(Some(event)) = listener.try_wait_one() {
                received_events[event.as_value()] += 1;
                total_received_events += 1;
            } else if number_of_finished_notifier_threads.load(Ordering::Relaxed)
                == number_of_notifier_threads
            {
                break;
            }
        }

        // ensure all events are read
        while let Ok(Some(event)) = listener.try_wait_one() {
            received_events[event.as_value()] += 1;
            total_received_events += 1;
        }

        for n in received_events {
            assert_that!(n, ge 1);
            assert_that!(n, le number_of_notifier_threads);
        }
    });
}

#[test]
fn listening_on_events_concurrently_works() {
    let _watchdog = Watchdog::new();
    type ServiceType = ipc_threadsafe::Service;
    let service_name = generate_service_name();
    let config = generate_isolated_config();
    let number_of_listener_threads: usize = SystemInfo::NumberOfCpuCores.value().min(2);
    const NUMBER_OF_ITERATIONS: usize = 2000;

    let node = NodeBuilder::new()
        .config(&config)
        .create::<ServiceType>()
        .unwrap();
    let service = node
        .service_builder(&service_name)
        .event()
        .max_notifiers(1)
        .max_listeners(1)
        .event_id_max_value(NUMBER_OF_ITERATIONS)
        .create()
        .unwrap();
    let notifier = service.notifier_builder().create().unwrap();
    let listener = service.listener_builder().create().unwrap();
    let barrier = Barrier::new(number_of_listener_threads + 1);
    let notification_finished = IoxAtomicBool::new(false);

    std::thread::scope(|s| {
        let mut listener_threads = vec![];
        for _ in 0..number_of_listener_threads {
            listener_threads.push(s.spawn(|| {
                let mut received_events = [0; NUMBER_OF_ITERATIONS];
                barrier.wait();
                loop {
                    if let Ok(Some(event)) = listener.try_wait_one() {
                        received_events[event.as_value()] += 1;
                    } else if notification_finished.load(Ordering::Relaxed) {
                        break;
                    }
                }

                // ensure all events are received
                while let Ok(Some(event)) = listener.try_wait_one() {
                    received_events[event.as_value()] += 1;
                }

                received_events
            }));
        }

        barrier.wait();
        for n in 0..NUMBER_OF_ITERATIONS {
            while notifier
                .notify_with_custom_event_id(EventId::new(n))
                .unwrap()
                == 0
            {}
        }
        notification_finished.store(true, Ordering::Relaxed);

        let mut received_events = [0; NUMBER_OF_ITERATIONS];
        for t in listener_threads {
            let events = t.join().unwrap();
            for (n, count) in events.iter().enumerate() {
                received_events[n] += count;
            }
        }

        for n in received_events {
            assert_that!(n, eq 1);
        }
    });
}
