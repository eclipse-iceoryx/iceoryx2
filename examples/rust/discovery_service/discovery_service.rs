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

// JSON is currently used as the serialization format.
fn deserialize(bytes: &[u8]) -> Result<Discovery, Box<dyn core::error::Error>> {
    let discovery =
        serde_json::from_slice(bytes).map_err(|e| Box::new(e) as Box<dyn core::error::Error>)?;

    Ok(discovery)
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let publish_subscribe = node
        .service_builder(service_discovery::service_name())
        .publish_subscribe::<service_discovery::Payload>()
        .open()
        .map_err(|error| {
            eprintln!("Unable to open service discovery service. Was it started?");
            error
        })?;

    let subscriber = publish_subscribe.subscriber_builder().create()?;

    let event = node
        .service_builder(service_discovery::service_name())
        .event()
        .open()
        .map_err(|error| {
            eprintln!("unable to open service discovery service. Was it started?");
            error
        })?;
    let listener = event.listener_builder().create()?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let guard = waitset.attach_notification(&listener)?;
    let attachment = WaitSetAttachmentId::from_guard(&guard);

    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        if attachment_id == attachment {
            // Drain all pending.
            listener.try_wait_all(|_| {}).unwrap();

            // Process new discovery information.
            while let Ok(Some(sample)) = subscriber.receive() {
                match deserialize(sample.payload()) {
                    Ok(Discovery::Added(details)) => {
                        println!("Added: {:?}", details.name());
                    }
                    Ok(Discovery::Removed(details)) => {
                        println!("Removed: {:?}", details.name());
                    }
                    Err(e) => {
                        eprintln!("error deserializing discovery event: {}", e);
                    }
                }
            }
        }

        CallbackProgression::Continue
    };

    waitset.wait_and_process(on_event)?;

    Ok(())
}
