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
use iceoryx2::{port::listener::Listener, prelude::*};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let args = Args::parse();

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    // factory lambda to create a listener with a given service name
    let create_listener =
        |service: &String| -> Result<Listener<ipc::Service>, Box<dyn core::error::Error>> {
            let event = node
                .service_builder(&service.as_str().try_into()?)
                .event()
                .open_or_create()?;

            Ok(event.listener_builder().create()?)
        };

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let mut listeners = vec![];
    let mut listener_attachments: HashMap<
        WaitSetAttachmentId<ipc::Service>,
        (&String, &Listener<ipc::Service>),
    > = HashMap::new();
    let mut guards = vec![];

    // create a listener for every service
    for service in &args.services {
        listeners.push((service, create_listener(service)?));
    }

    // attach all listeners to the waitset and store the guard
    for (service, listener) in &listeners {
        let guard = waitset.attach_notification(listener)?;
        listener_attachments.insert(WaitSetAttachmentId::from_guard(&guard), (service, listener));
        guards.push(guard);
    }

    println!("Waiting on the following services: {:?}", args.services);

    // the callback that is called when a listener has received an event
    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        if let Some((service_name, listener)) = listener_attachments.get(&attachment_id) {
            print!("Received trigger from \"{service_name}\" ::");

            // IMPORTANT:
            // We need to collect all notifications since the WaitSet will wake us up as long as
            // there is something to read. If we skip this step completely we will end up in a
            // busy loop.
            listener
                .try_wait_all(|event_id| {
                    print!(" {event_id:?}");
                })
                .unwrap();

            println!();
        }

        CallbackProgression::Continue
    };

    // loops until the user has pressed CTRL+c, the application has received a SIGTERM or SIGINT
    // signal or the user has called explicitly `waitset.stop()` in the `on_event` callback. We
    // didn't add this to the example so feel free to play around with it.
    waitset.wait_and_process(on_event)?;

    println!("exit");

    Ok(())
}

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Defines the service to which events are emitted.
    #[clap(short, long)]
    services: Vec<String>,
}
