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
        .blackboard_opener::<KeyType>()
        .open()?;

    let event_service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .event()
        .open_or_create()?;
    let notifier = event_service.notifier_builder().create()?;

    let writer = service.writer_builder().create()?;
    let writer_handle = writer.entry::<u64>(&0)?;
    let interesting_writer_handle = writer.entry::<u64>(&interesting_key)?;

    // notify with entry id
    let mut counter: u64 = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        interesting_writer_handle.update_with_copy(counter);
        notifier.notify_with_custom_event_id(interesting_writer_handle.entry_id())?;

        writer_handle.update_with_copy(2 * counter);
        notifier.notify_with_custom_event_id(writer_handle.entry_id())?;

        println!("Trigger event with entry id...");
    }

    println!("exit");

    Ok(())
}
