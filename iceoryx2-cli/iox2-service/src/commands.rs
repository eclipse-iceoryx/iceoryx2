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

use anyhow::{Context, Error, Result};
use iceoryx2::prelude::*;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_bb_posix::signal::SignalHandler;
use iceoryx2_cli::filter::Filter;
use iceoryx2_cli::output::ServiceDescription;
use iceoryx2_cli::output::ServiceDescriptor;
use iceoryx2_cli::Format;
use iceoryx2_services_discovery::service_discovery::DiscoveryConfig;
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

#[derive(serde::Serialize)]
enum ChangeKind {
    Added,
    Removed,
}

#[derive(serde::Serialize)]
struct ChangeDetails {
    name: String,
    pattern: String,
    kind: ChangeKind,
}

impl ChangeDetails {
    fn new(kind: ChangeKind, details: &StaticConfig) -> Self {
        Self {
            name: details.name().to_string(),
            pattern: details.messaging_pattern().to_string(),
            kind,
        }
    }
}

#[derive(serde::Serialize)]
struct SerializableDiscoveryConfig {
    publish_events: bool,
    max_subscribers: usize,
    send_notifications: bool,
    max_listeners: usize,
    include_internal: bool,
}

impl SerializableDiscoveryConfig {
    fn from_config(config: &DiscoveryConfig) -> Self {
        Self {
            publish_events: config.publish_events,
            max_subscribers: config.max_subscribers,
            send_notifications: config.send_notifications,
            max_listeners: config.max_listeners,
            include_internal: config.include_internal,
        }
    }
}

/// Starts a service monitor.
///
/// # Arguments
///
/// * `service_name` - The name of the service monitoring service
/// * `rate` - The update rate in milliseconds between monitor refreshes
/// * `publish_events` - Whether to publish events about service changes
/// * `send_notifications` - Whether to send notifications about service changes
pub fn discovery(
    rate: u64,
    publish_events: bool,
    max_subscribers: usize,
    send_notifications: bool,
    max_listeners: usize,
    format: Format,
) -> Result<()> {
    let monitor_config = DiscoveryConfig {
        publish_events,
        max_subscribers,
        send_notifications,
        max_listeners,
        include_internal: false,
    };

    let mut service =
        DiscoveryService::<ipc::Service>::create(&monitor_config, &Config::global_config())
            .map_err(|e| anyhow::anyhow!("failed to create service monitor: {:?}", e))?;

    println!("=== Service Discovery Service Started ===");
    println!(
        "{}",
        format.as_string(&SerializableDiscoveryConfig::from_config(&monitor_config))?
    );

    while !SignalHandler::termination_requested() {
        match service.spin() {
            Ok((added, removed)) => {
                for service in added {
                    println!(
                        "{}",
                        format.as_string(&ChangeDetails::new(
                            ChangeKind::Added,
                            &service.static_details,
                        ))?,
                    )
                }
                for service in removed {
                    println!(
                        "{}",
                        format.as_string(&ChangeDetails::new(
                            ChangeKind::Removed,
                            &service.static_details
                        ))?
                    )
                }
            }
            Err(e) => {
                eprintln!("error during spin: {:?}", e);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(rate));
    }

    Ok(())
}
