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

use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);
static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn background_thread() {
    // Another node is created inside this thread to communicate with the main thread
    let node = NodeBuilder::new()
        // Optionally, a name can be provided to the node which helps identifying them later during
        // debugging or introspection
        .name(&"threadnode".try_into().unwrap())
        .create::<local::Service>()
        .unwrap();

    let service = node
        .service_builder(&"Service-Variants-Example".try_into().unwrap())
        .publish_subscribe::<u64>()
        .open_or_create()
        .unwrap();

    let subscriber = service.subscriber_builder().create().unwrap();

    while KEEP_RUNNING.load(Ordering::Relaxed) {
        std::thread::sleep(CYCLE_TIME);
        while let Some(sample) = subscriber.receive().unwrap() {
            println!("[thread] received: {}", sample.payload());
        }
    }
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new()
        // Optionally, a name can be provided to the node which helps identifying them later during
        // debugging or introspection
        .name(&"mainnode".try_into()?)
        // When choosing `local::Service` the service does not use inter-process mechanisms
        // like shared memory or unix domain sockets but mechanisms like socketpairs and heap.
        //
        // Those services can communicate only within a single process.
        .create::<local::Service>()?;

    let service = node
        .service_builder(&"Service-Variants-Example".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;
    let background_thread = std::thread::spawn(background_thread);

    let mut counter = 0u64;
    while node.wait(CYCLE_TIME).is_ok() {
        println!("send: {counter}");
        publisher.send_copy(counter)?;
        counter += 1;
    }

    KEEP_RUNNING.store(false, Ordering::Relaxed);
    let _ = background_thread.join();
    println!("exit");

    Ok(())
}
