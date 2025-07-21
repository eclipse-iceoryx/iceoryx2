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

use iceoryx2::prelude::*;
use iceoryx2_services_discovery::*;
use service_discovery::Discovery;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let publish_subscribe = node
        .service_builder(service_discovery::service_name())
        .publish_subscribe::<service_discovery::Payload>()
        .open()
        .inspect_err(|_| {
            eprintln!("Unable to open service discovery service. Was it started?");
        })?;

    let subscriber = publish_subscribe.subscriber_builder().create()?;

    let event = node
        .service_builder(service_discovery::service_name())
        .event()
        .open()
        .inspect_err(|_| {
            eprintln!("unable to open service discovery service. Was it started?");
        })?;
    let listener = event.listener_builder().create()?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let guard = waitset.attach_notification(&listener)?;
    let attachment = WaitSetAttachmentId::from_guard(&guard);

    println!("Discovery service ready!");

    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        if attachment_id == attachment {
            // Drain all pending.
            listener.try_wait_all(|_| {}).unwrap();

            // Process new discovery information.
            while let Ok(Some(sample)) = subscriber.receive() {
                match sample.payload() {
                    Discovery::Added(details) => {
                        println!("Added: {:?}", details.name());
                    }
                    Discovery::Removed(details) => {
                        println!("Removed: {:?}", details.name());
                    }
                }
            }
        }

        CallbackProgression::Continue
    };

    waitset.wait_and_process(on_event)?;

    Ok(())
}
