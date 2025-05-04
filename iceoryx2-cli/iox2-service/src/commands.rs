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

use anyhow::anyhow;
use anyhow::{Context, Error, Result};
use iceoryx2::prelude::*;
use iceoryx2_cli::filter::Filter;
use iceoryx2_cli::output::ServiceDescription;
use iceoryx2_cli::output::ServiceDescriptor;
use iceoryx2_cli::Format;
use iceoryx2_services_discovery::service_discovery::Config as DiscoveryConfig;
use iceoryx2_services_discovery::service_discovery::Discovery;
use iceoryx2_services_discovery::service_discovery::Service as DiscoveryService;

use crate::cli::OutputFilter;

pub fn list(filter: OutputFilter, format: Format) -> Result<()> {
    let mut services = Vec::<ServiceDescriptor>::new();

    ipc::Service::list(Config::global_config(), |service| {
        if filter.matches(&service) {
            services.push(ServiceDescriptor::from(&service));
        }
        CallbackProgression::Continue
    })
    .context("failed to retrieve services")?;

    services.sort_by_key(|pattern| match pattern {
        ServiceDescriptor::PublishSubscribe(name) => (name.clone(), 0),
        ServiceDescriptor::Event(name) => (name.clone(), 1),
        ServiceDescriptor::RequestResponse(name) => (name.clone(), 2),
        ServiceDescriptor::Undefined(name) => (name.to_string(), 3),
    });

    print!("{}", format.as_string(&services)?);

    Ok(())
}

pub fn details(service_name: String, filter: OutputFilter, format: Format) -> Result<()> {
    let mut error: Option<Error> = None;

    ipc::Service::list(Config::global_config(), |service| {
        if service_name == service.static_details.name().to_string() && filter.matches(&service) {
            match format.as_string(&ServiceDescription::from(&service)) {
                Ok(output) => {
                    print!("{}", output);
                    CallbackProgression::Continue
                }
                Err(e) => {
                    error = Some(e);
                    CallbackProgression::Stop
                }
            }
        } else {
            CallbackProgression::Continue
        }
    })?;

    if let Some(err) = error {
        return Err(err);
    }
    Ok(())
}

pub fn discovery(
    rate: u64,
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

    println!("=== Service Started (rate: {}ms) ===", rate);

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let guard = waitset
        .attach_interval(core::time::Duration::from_millis(rate))
        .map_err(|e| anyhow!("failed to attach interval to waitset: {:?}", e))?;
    let attachment = WaitSetAttachmentId::from_guard(&guard);

    let on_event = |attachment_id: WaitSetAttachmentId<ipc::Service>| {
        if attachment_id == attachment {
            let on_added = |service: &ServiceDetails<ipc::Service>| {
                println!(
                    "{}",
                    format
                        .as_string(&Discovery::Added(service.static_details.clone()))
                        .unwrap_or_default()
                )
            };
            let on_removed = |service: &ServiceDetails<ipc::Service>| {
                println!(
                    "{}",
                    format
                        .as_string(&Discovery::Removed(service.static_details.clone()))
                        .unwrap_or_default()
                )
            };
            if let Err(e) = service.spin(on_added, on_removed) {
                eprintln!("error while spinning service: {:?}", e);
            }
        }

        CallbackProgression::Continue
    };

    waitset
        .wait_and_process(on_event)
        .map_err(|e| anyhow!("error waiting on waitset: {:?}", e))?;

    Ok(())
}
