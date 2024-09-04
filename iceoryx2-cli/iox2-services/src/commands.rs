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

use anyhow::{anyhow, Error, Result};
use iceoryx2::prelude::*;

use crate::cli::DetailsFilter;

pub fn list() -> Result<()> {
    ipc::Service::list(Config::global_config(), |service| {
        println!("- {}", &service.static_details.name().as_str());
        CallbackProgression::Continue
    })
    .map_err(Error::new)
}

pub fn details(name: &str, filter: DetailsFilter) -> Result<()> {
    let service_name: ServiceName = name.try_into()?;
    let mut error: Option<Error> = None;

    ipc::Service::list(Config::global_config(), |service| {
        if service_name == *service.static_details.name() {
            match filter {
                DetailsFilter::None => match serde_yaml::to_string(&service.to_serializable()) {
                    Ok(details_string) => print!("{}", details_string),
                    Err(e) => error = Some(anyhow!(e)),
                },
                DetailsFilter::Static => match serde_yaml::to_string(&service.static_details) {
                    Ok(details_string) => print!("{}", details_string),
                    Err(e) => error = Some(anyhow!(e)),
                },
                DetailsFilter::Dynamic => {
                    if let Some(dynamic_details) = &service.dynamic_details {
                        match serde_yaml::to_string(&dynamic_details.to_serializable()) {
                            Ok(details_string) => print!("{}", details_string),
                            Err(e) => error = Some(anyhow!(e)),
                        }
                    }
                }
            }
            CallbackProgression::Stop
        } else {
            CallbackProgression::Continue
        }
    })
    .map_err(Error::new)?;

    if let Some(err) = error {
        return Err(err);
    }
    Ok(())
}
