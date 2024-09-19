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

use crate::cli::DetailsFilter;
use crate::output::*;
use anyhow::{Context, Error, Result};
use iceoryx2::prelude::*;
use iceoryx2_cli_utils::Filter;
use iceoryx2_cli_utils::Format;

pub fn list(format: Format) -> Result<()> {
    let mut services = ServiceList::new();

    ipc::Service::list(Config::global_config(), |service| {
        services.push(ServiceDescriptor::from(service));
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

pub fn details(service_name: String, filter: DetailsFilter, format: Format) -> Result<()> {
    let mut error: Option<Error> = None;

    ipc::Service::list(Config::global_config(), |service| {
        if service_name == service.static_details.name().to_string() {
            let description = ServiceDescription::from(&service);

            if filter.matches(&description) {
                match format.as_string(&description) {
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
                // Filter did not match
                CallbackProgression::Continue
            }
        } else {
            // Service name did not match
            CallbackProgression::Continue
        }
    })?;

    if let Some(err) = error {
        return Err(err);
    }
    Ok(())
}
