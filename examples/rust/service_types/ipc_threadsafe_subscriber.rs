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

extern crate alloc;
use alloc::boxed::Box;
use alloc::sync::Arc;

use iceoryx2::prelude::*;
use iceoryx2_bb_posix::clock::nanosleep;
use iceoryx2_bb_posix::thread::{ThreadBuilder, ThreadName};

static KEEP_RUNNING: AtomicBool = AtomicBool::new(true);
const CYCLE_TIME: Duration = Duration::from_secs(1);

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
    let other_thread = ThreadBuilder::new()
        .name(&ThreadName::from_bytes(b"other_thread").unwrap())
        .spawn(move || {
            while KEEP_RUNNING.load(Ordering::Relaxed) {
                nanosleep(CYCLE_TIME).unwrap();
                if let Some(sample) = in_thread_subscriber.receive().unwrap() {
                    cout!("[thread] received: {}", sample.payload());
                }
            }
        })?;

    while node.wait(CYCLE_TIME).is_ok() {
        if let Some(sample) = subscriber.receive()? {
            cout!("[main] received: {}", sample.payload());
        }
    }

    KEEP_RUNNING.store(false, Ordering::Relaxed);
    drop(other_thread);

    cout!("exit");

    Ok(())
}
