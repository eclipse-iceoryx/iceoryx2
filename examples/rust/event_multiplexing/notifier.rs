// Copyright (c) 2024 Contributors to the Eclipse Foundation
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

use clap::Parser;
use core::time::Duration;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let args = Args::parse();
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let event = node
        .service_builder(&args.service.as_str().try_into()?)
        .event()
        .open_or_create()?;

    let notifier = event.notifier_builder().create()?;

    while node.wait(CYCLE_TIME).is_ok() {
        notifier.notify_with_custom_event_id(EventId::new(args.event_id))?;

        println!("[service: \"{}\"] Trigger event ...", args.service);
    }

    println!("exit");

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Defines the service to which events are emitted.
    #[clap(short, long)]
    service: String,

    /// The event id used for triggering
    #[clap(short, long, default_value_t = 0)]
    event_id: usize,
}
