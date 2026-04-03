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

use iceoryx2_bb_posix::barrier::BarrierBuilder;
use iceoryx2_bb_posix::barrier::BarrierHandle;
use iceoryx2_bb_posix::mutex::Handle;
use iceoryx2_bb_posix::mutex::MutexBuilder;
use iceoryx2_bb_posix::mutex::MutexHandle;

use iceoryx2::prelude::*;
use iceoryx2::testing::*;
use iceoryx2_bb_concurrency::atomic::AtomicBool;
use iceoryx2_bb_concurrency::atomic::AtomicUsize;
use iceoryx2_bb_concurrency::atomic::Ordering;
use iceoryx2_bb_posix::system_configuration::SystemInfo;
use iceoryx2_bb_posix::thread::thread_scope;
use iceoryx2_bb_testing::assert_that;
use iceoryx2_bb_testing::watchdog::Watchdog;
use iceoryx2_bb_testing_macros::test;

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

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new((number_of_notifier_threads + 1) as u32)
        .create(&barrier_handle)
        .unwrap();

    let number_of_finished_notifier_threads = AtomicUsize::new(0);
    thread_scope(|s| {
        for _ in 0..number_of_notifier_threads {
            s.thread_builder()
                .spawn(|| {
                    barrier.wait();
                    for n in 0..NUMBER_OF_ITERATIONS {
                        while notifier
                            .notify_with_custom_event_id(EventId::new(n))
                            .unwrap()
                            == 0
                        {}
                    }
                    number_of_finished_notifier_threads.fetch_add(1, Ordering::Relaxed);
                })
                .expect("failed to spawn thread");
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
        }

        for n in received_events {
            assert_that!(n, ge 1);
            assert_that!(n, le number_of_notifier_threads);
        }

        Ok(())
    })
    .expect("failed to spawn thread");
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

    let barrier_handle = BarrierHandle::new();
    let barrier = BarrierBuilder::new((number_of_listener_threads + 1) as u32)
        .create(&barrier_handle)
        .unwrap();

    let notification_finished = AtomicBool::new(false);
    let all_events_handle = MutexHandle::<Vec<usize>>::new();
    let all_events = MutexBuilder::new()
        .create(vec![0usize; NUMBER_OF_ITERATIONS], &all_events_handle)
        .expect("failed to create mutex");

    thread_scope(|s| {
        for _ in 0..number_of_listener_threads {
            s.thread_builder()
                .spawn(|| {
                    let mut received_events = vec![0usize; NUMBER_OF_ITERATIONS];
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

                    let mut guard = all_events.lock().unwrap();
                    for (i, count) in received_events.iter().enumerate() {
                        guard[i] += count;
                    }
                })
                .expect("failed to spawn thread");
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

        Ok(())
    })
    .expect("failed to spawn thread");

    let guard = all_events.lock().unwrap();
    for n in guard.iter() {
        assert_that!(*n, eq 1);
    }
}
