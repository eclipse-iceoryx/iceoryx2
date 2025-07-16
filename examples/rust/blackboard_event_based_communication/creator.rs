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

use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;
    type KeyType = u32;
    let interesting_key = 99;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard_creator::<KeyType>()
        .add_with_default::<u64>(0)
        .add_with_default::<u64>(interesting_key)
        .create()?;

    let event_service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .event()
        .open_or_create()?;
    let listener = event_service.listener_builder().create()?;

    let reader = service.reader_builder().create()?;
    let reader_handle = reader.entry::<u64>(&interesting_key)?;

    // wait for entry id
    while node.wait(Duration::ZERO).is_ok() {
        if let Ok(Some(id)) = listener.timed_wait_one(CYCLE_TIME) {
            if id == reader_handle.entry_id() {
                println!("read: {}", reader_handle.get());
            }
        }
    }

    println!("exit");

    Ok(())
}
