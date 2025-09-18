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

use anyhow::anyhow;
use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2_cli::output::DiscoveryEvent;
use iceoryx2_cli::output::ServiceDescription;
use iceoryx2_cli::output::ServiceDescriptor;
use iceoryx2_cli::Format;
use iceoryx2_services_discovery::service_discovery::Config as DiscoveryConfig;
use iceoryx2_services_discovery::service_discovery::Service as DiscoveryService;

pub(crate) fn discovery(
    rate: u64,
    detailed: bool,
    publish_events: bool,
    max_subscribers: usize,
    send_notifications: bool,
    max_listeners: usize,
    format: Format,
) -> Result<()> {
    let discovery_config = DiscoveryConfig {
        publish_events,
        max_subscribers,
        send_notifications,
        max_listeners,
        include_internal: false,
        ..Default::default()
    };

    let mut service =
        DiscoveryService::<ipc::Service>::create(&discovery_config, Config::global_config())
            .map_err(|e| anyhow::anyhow!("failed to create service: {:?}", e))?;

    println!("Discovering Services (rate: {rate}ms)");

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let guard = waitset
        .attach_interval(core::time::Duration::from_millis(rate))
        .map_err(|e| anyhow!("failed to attach interval to waitset: {:?}", e))?;
    let tick = WaitSetAttachmentId::from_guard(&guard);

    let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
        if id == tick {
            let on_added = |service: &ServiceDetails<ipc::Service>| {
                if detailed {
                    println!(
                        "{}",
                        format
                            .as_string(&DiscoveryEvent::Added(ServiceDescription::from(service)))
                            .unwrap_or_default()
                    )
                } else {
                    println!(
                        "{}",
                        format
                            .as_string(&DiscoveryEvent::Added(ServiceDescriptor::from(service)))
                            .unwrap_or_default()
                    )
                }
            };
            let on_removed = |service: &ServiceDetails<ipc::Service>| {
                if detailed {
                    println!(
                        "{}",
                        format
                            .as_string(&DiscoveryEvent::Removed(ServiceDescription::from(service)))
                            .unwrap_or_default()
                    )
                } else {
                    println!(
                        "{}",
                        format
                            .as_string(&DiscoveryEvent::Removed(ServiceDescriptor::from(service)))
                            .unwrap_or_default()
                    )
                }
            };
            if let Err(e) = service.spin(on_added, on_removed) {
                eprintln!("error while spinning service: {e:?}");
            }
        }

        CallbackProgression::Continue
    };

    waitset
        .wait_and_process(on_event)
        .map_err(|e| anyhow!("error waiting on waitset: {:?}", e))?;

    Ok(())
}
