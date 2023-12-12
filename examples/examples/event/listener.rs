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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_name = ServiceName::new("MyEventName")?;

    let event = zero_copy::Service::new(&event_name)
        .event()
        .open_or_create()?;

    let mut listener = event.listener().create()?;

    while let Iox2Event::Tick = Iox2::wait(Duration::ZERO) {
        if let Ok(events) = listener.timed_wait(CYCLE_TIME) {
            for event_id in events {
                println!("event was triggered with id: {:?}", event_id);
            }
        }
    }

    println!("exit ...");

    Ok(())
}
