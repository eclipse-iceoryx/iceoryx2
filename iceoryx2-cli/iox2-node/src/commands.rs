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
use iceoryx2_cli::filter::Filter;
use iceoryx2_cli::filter::NodeIdentifier;
use iceoryx2_cli::output::NodeDescription;
use iceoryx2_cli::output::NodeDescriptor;
use iceoryx2_cli::output::NodeList;
use iceoryx2_cli::Format;

use crate::cli::OutputFilter;

pub fn list(filter: OutputFilter, format: Format) -> Result<()> {
    let mut nodes = Vec::<NodeDescriptor>::new();
    Node::<ipc::Service>::list(Config::global_config(), |node| {
        if filter.matches(&node) {
            nodes.push(NodeDescriptor::from(&node));
        }
        CallbackProgression::Continue
    })
    .context("failed to retrieve nodes")?;

    print!(
        "{}",
        format.as_string(&NodeList {
            num: nodes.len(),
            details: nodes
        })?
    );

    Ok(())
}

pub fn details(identifier: NodeIdentifier, filter: OutputFilter, format: Format) -> Result<()> {
    let mut error: Option<Error> = None;

    Node::<ipc::Service>::list(Config::global_config(), |node| {
        if identifier.matches(&node) && filter.matches(&node) {
            match format.as_string(&NodeDescription::from(&node)) {
                Ok(output) => {
                    print!("{output}");
                }
                Err(e) => {
                    error = Some(e);
                }
            }
        }
        CallbackProgression::Continue
    })
    .context("failed to retrieve nodes")?;

    Ok(())
}
