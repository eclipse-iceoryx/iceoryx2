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
    const INTERESTING_KEY: u32 = 1;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .blackboard_creator::<KeyType>()
        .add_with_default::<u64>(0)
        .add_with_default::<u64>(INTERESTING_KEY)
        .create()?;

    println!("Blackboard created.\n");

    let event_service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .event()
        .open_or_create()?;
    let notifier = event_service.notifier_builder().create()?;

    let writer = service.writer_builder().create()?;

    let entry_handle_mut = writer.entry::<u64>(&0)?;
    let entry_id = entry_handle_mut.entry_id();

    let interesting_entry_handle_mut = writer.entry::<u64>(&INTERESTING_KEY)?;
    let interesting_entry_id = interesting_entry_handle_mut.entry_id();

    // notify with entry id
    let mut counter: u64 = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        interesting_entry_handle_mut.update_with_copy(counter);
        notifier.notify_with_custom_event_id(interesting_entry_id)?;
        println!(
            "Trigger event with entry id {}",
            interesting_entry_id.as_value()
        );

        entry_handle_mut.update_with_copy(2 * counter);
        notifier.notify_with_custom_event_id(entry_id)?;
        println!("Trigger event with entry id {}", entry_id.as_value());
    }

    println!("exit");

    Ok(())
}
