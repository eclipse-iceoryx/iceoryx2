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
use iceoryx2::tracker::service::Tracker;
use iceoryx2_cli::filter::Filter;
use iceoryx2_cli::output::ServiceDescription;
use iceoryx2_cli::output::ServiceDescriptor;
use iceoryx2_cli::Format;

use crate::cli::OutputFilter;

pub fn list(filter: OutputFilter, format: Format) -> Result<()> {
    let mut services = Vec::<ServiceDescriptor>::new();

    ipc::Service::list(Config::global_config(), |service| {
        if filter.matches(&service) {
            services.push(ServiceDescriptor::from(service));
        }
        CallbackProgression::Continue
    })
    .context("failed to retrieve services")?;

    services.sort_by_key(|pattern| match pattern {
        ServiceDescriptor::PublishSubscribe(name) => (name.clone(), 0),
        ServiceDescriptor::Event(name) => (name.clone(), 1),
        ServiceDescriptor::Undefined(name) => (name.to_string(), 2),
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

// Monitor services that come and go.
//
// 1. Periodically check for added/removed services
// 2. Publish changes on internal topic
pub fn monitor(rate: u64) -> Result<()> {
    println!("Starting Service Monitor...");

    let node = NodeBuilder::new()
        .config(Config::global_config())
        .create::<ipc::Service>()
        .expect("failed to create monitor node");
    let service_name =
        ServiceName::new("iox2://monitor/services").expect("failed to create monitor service name");
    let _service_pubsub = node
        .service_builder(&service_name)
        .publish_subscribe::<u64>()
        .create()
        .expect("failed to create publish-subscribe service");
    let _service_event = node
        .service_builder(&service_name)
        .event()
        .create()
        .expect("failed to create event service");

    let mut tracker = Tracker::<ipc::Service>::new();

    println!("Monitoring services (update rate: {}ms)", rate);

    loop {
        // identify added/removed services
        let (_added, _removed) = tracker.sync(Config::global_config());

        // publish changes

        // wait
        std::thread::sleep(std::time::Duration::from_millis(rate));
    }

    // Ok(())
}
