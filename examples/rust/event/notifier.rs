// Copyright (c) 2023 Contributors to the Eclipse Foundation
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

    let event = node
        .service_builder(&"MyEventName".try_into()?)
        .event()
        .open_or_create()?;
    let max_event_id = event.static_config().event_id_max_value();

    let notifier = event.notifier_builder().create()?;

    let mut counter: usize = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        notifier.notify_with_custom_event_id(EventId::new(counter % max_event_id))?;

        println!("Trigger event with id {counter} ...");
    }

    println!("exit");

    Ok(())
}
