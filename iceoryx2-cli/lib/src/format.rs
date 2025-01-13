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

use anyhow::{anyhow, Context, Error, Result};
use clap::ValueEnum;
use core::str::FromStr;
use serde::Serialize;

#[derive(Clone, Copy, ValueEnum)]
#[value(rename_all = "UPPERCASE")]
pub enum Format {
    Ron,
    Json,
    Yaml,
}

impl Format {
    pub fn as_string<T: Serialize>(self, data: &T) -> Result<String> {
        match self {
            Format::Ron => ron::ser::to_string_pretty(
                data,
                ron::ser::PrettyConfig::new().separate_tuple_members(true),
            )
            .context("failed to serialize to RON format"),
            Format::Json => {
                serde_json::to_string_pretty(data).context("failed to serialize to JSON format")
            }
            Format::Yaml => serde_yaml::to_string(data).context("failed to serialize to YAML"),
        }
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "RON" => Ok(Format::Ron),
            "JSON" => Ok(Format::Json),
            "YAML" => Ok(Format::Yaml),
            _ => Err(anyhow!("unsupported output format '{}'", s)),
        }
    }
}
