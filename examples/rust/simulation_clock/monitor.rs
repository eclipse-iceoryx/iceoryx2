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

const DEADLINE: Duration = Duration::from_secs(2);

fn start_monitoring<ServiceVariant: iceoryx2::service::Service>()
-> Result<(), Box<dyn core::error::Error>>
where
    <ServiceVariant::Event as iceoryx2_cal::event::Event>::Listener: SynchronousMultiplexing,
{
    let node = NodeBuilder::new().create::<ServiceVariant>()?;
    let event_service_sample_sent = node
        .service_builder(&"Time/Controlled/Service/SampleSent".try_into()?)
        .event()
        .open_or_create()?;
    let event_service_sample_received = node
        .service_builder(&"Time/Controlled/Service/SampleReceived".try_into()?)
        .event()
        .open_or_create()?;

    let listener_sample_sent = event_service_sample_sent.listener_builder().create()?;
    let listener_sample_received = event_service_sample_received.listener_builder().create()?;

    println!(
        "{} ::: monitor start time: {}",
        core::any::type_name::<ServiceVariant>(),
        node.now()?
    );

    let waitset = node.waitset_builder().create()?;
    let sample_sent_attachment = waitset.attach_deadline(&listener_sample_sent, DEADLINE)?;
    let sample_received_attachment =
        waitset.attach_deadline(&listener_sample_received, DEADLINE)?;

    let on_event = |attachment_id: WaitSetAttachmentId<ServiceVariant>| {
        cout!("[current time: {}] ", node.now().unwrap());
        if attachment_id.has_event_from(&sample_sent_attachment) {
            listener_sample_sent.try_wait(|_| {}).unwrap();
            coutln!("sample was sent");
        } else if attachment_id.has_missed_deadline(&sample_sent_attachment) {
            coutln!("sample sent notification deadline missed");
        } else if attachment_id.has_event_from(&sample_received_attachment) {
            listener_sample_received
                .try_wait(|event_id| {
                    if event_id.id == EventId::new(0) {
                        coutln!("{} samples were received", event_id.count)
                    } else if event_id.id == EventId::new(1) {
                        coutln!("{} times the timeout was hit", event_id.count);
                    }
                })
                .unwrap();
        } else if attachment_id.has_missed_deadline(&sample_received_attachment) {
            coutln!("sample received notification deadline missed");
        } else {
            coutln!("spurious wakeup");
        }

        CallbackProgression::Continue
    };

    waitset.wait_and_process(on_event)?;

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
        start_monitoring::<ipc_simulated_clock::Service>()?;
    } else if args.len() == 2 && args[1] == "real" {
        start_monitoring::<ipc::Service>()?;
    } else {
        print_help();
    }

    Ok(())
}
