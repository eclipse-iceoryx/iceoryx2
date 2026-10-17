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

const CYCLE_TIME: Duration = Duration::from_secs(2);

fn start_subscribing<ServiceVariant: iceoryx2::service::Service>()
-> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ServiceVariant>()?;

    let pubsub_service = node
        .service_builder(&"Time/Controlled/Service".try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;
    let event_service_sample_sent = node
        .service_builder(&"Time/Controlled/Service/SampleSent".try_into()?)
        .event()
        .open_or_create()?;
    let event_service_sample_received = node
        .service_builder(&"Time/Controlled/Service/SampleReceived".try_into()?)
        .event()
        .open_or_create()?;

    let subscriber = pubsub_service.subscriber_builder().create()?;
    let listener = event_service_sample_sent.listener_builder().create()?;
    let notifier = event_service_sample_received.notifier_builder().create()?;

    println!(
        "{} ::: receiver start time: {}",
        core::any::type_name::<ServiceVariant>(),
        node.now()?
    );
    while listener.timed_wait(|_| {}, CYCLE_TIME).is_ok() {
        let mut number_of_received_samples = 0;
        cout!("[current time: {}] ", node.now()?);
        while subscriber.receive()?.is_some() {
            if number_of_received_samples == 0 {
                coutln!("");
            }
            number_of_received_samples += 1;
            coutln!("  received {} sample(s) ", number_of_received_samples);
            notifier.notify_with_custom_event_id(EventId::new(0))?;
        }

        if number_of_received_samples == 0 {
            notifier.notify_with_custom_event_id(EventId::new(1))?;
            coutln!("nothing received ");
        }
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
        start_subscribing::<ipc_simulated_clock::Service>()?;
    } else if args.len() == 2 && args[1] == "real" {
        start_subscribing::<ipc::Service>()?;
    } else {
        print_help();
    }

    Ok(())
}
