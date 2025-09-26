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

use anyhow::{Context, Result};
use iceoryx2::prelude::*;
use iceoryx2_cli::filter::Filter;
use iceoryx2_cli::output::ServiceDescriptor;
use iceoryx2_cli::Format;

use crate::cli::OutputFilter;

pub(crate) fn list(filter: OutputFilter, format: Format) -> Result<()> {
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

    println!("{}", format.as_string(&services)?);

    Ok(())
}
