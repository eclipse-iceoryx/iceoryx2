// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

extern crate alloc;
use alloc::boxed::Box;

use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn start_publishing<ServiceVariant: iceoryx2::service::Service>()
-> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ServiceVariant>()?;

    let pubsub_service = node
        .service_builder(&"Time/Controlled/Service".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;
    let event_service = node
        .service_builder(&"Time/Controlled/Service/SampleSent".try_into()?)
        .event()
        .open_or_create()?;

    let publisher = pubsub_service.publisher_builder().create()?;
    let notifier = event_service.notifier_builder().create()?;

    let mut counter: u64 = 0;

    println!(
        "{} ::: sender start time: {}",
        core::any::type_name::<ServiceVariant>(),
        node.now()?
    );
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;

        publisher.send_copy(counter)?;
        notifier.notify()?;

        coutln!("[current time: {}] Send sample {counter} ...", node.now()?);
    }

    coutln!("exit");

    Ok(())
}

fn print_help() {
    coutln!("provide one argument, either:");
    coutln!("  - 'sim' to use the simulation clock or");
    coutln!("  - 'real' to use the real clock");
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 && args[1] == "sim" {
        start_publishing::<ipc_simulated_clock::Service>()?;
    } else if args.len() == 2 && args[1] == "real" {
        start_publishing::<ipc::Service>()?;
    } else {
        print_help();
    }

    Ok(())
}
