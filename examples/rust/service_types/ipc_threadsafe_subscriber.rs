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

extern crate alloc;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);
static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new()
        // There are the `local_threadsafe::Service` and `ipc_threadsafe::Service`
        // versions where all ports are threadsafe but with the cost of an additional mutex
        // lock/unlock call.
        //
        // The thread-safe version of a particular service variant can also communicate with the
        // non-thread-safe version.
        .create::<ipc_threadsafe::Service>()?;

    let service = node
        .service_builder(&"Service-Variants-Example".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;

    let subscriber = Arc::new(service.subscriber_builder().create()?);
    let in_thread_subscriber = subscriber.clone();

    // The ports created by a thread-safe service implement `Send` and `Sync`, so they can be
    // be shared between threads.
    let t = std::thread::spawn(move || {
        while KEEP_RUNNING.load(Ordering::Relaxed) {
            std::thread::sleep(CYCLE_TIME);
            if let Some(sample) = in_thread_subscriber.receive().unwrap() {
                println!("[thread] received: {}", sample.payload());
            }
        }
    });

    while node.wait(CYCLE_TIME).is_ok() {
        if let Some(sample) = subscriber.receive()? {
            println!("[main] received: {}", sample.payload());
        }
    }

    KEEP_RUNNING.store(false, Ordering::Relaxed);
    let _ = t.join();

    println!("exit");

    Ok(())
}
