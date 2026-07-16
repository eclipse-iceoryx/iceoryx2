// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use std::path::PathBuf;

use clap::{Parser, ValueEnum};

use iceoryx2_cli::help_template;

#[derive(Parser)]
#[command(
    name = "iox2 tunnel ros2",
    bin_name = "iox2 tunnel ros2",
    about = "Launch an iceoryx2 tunnel bridging to ROS 2.",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    help_template = help_template().build(),
)]
pub struct Cli {
    #[clap(
        long = "static",
        value_name = "TOML",
        help = "Path to a static mapping TOML file; services are mapped by the \
                'ros2://topics/' name prefix when omitted"
    )]
    static_mapping: Option<PathBuf>,

    #[clap(
        long,
        value_enum,
        default_value_t = Translator::Passthrough,
        help = "Payload treatment for tunneled services"
    )]
    pub translator: Translator,

    #[clap(
        long,
        short = 'd',
        help = "Name of a service providing discovery updates to connect to"
    )]
    pub discovery_service: Option<String>,

    #[clap(
        long = "service",
        short = 's',
        value_name = "NAME",
        action = clap::ArgAction::Append,
        help = "Restrict tunneling to the listed service names. May be repeated. When omitted, all discovered services are tunneled."
    )]
    pub services: Vec<String>,

    #[clap(
        long,
        value_name = "RATE",
        help = "Polling rate in milliseconds for discovery and sample propagation \
                (defaults to 100ms when no other flags are given; otherwise must be \
                set explicitly to enable polling)"
    )]
    pub poll: Option<u64>,

    #[clap(
        long = "reactive-backend",
        help = "Reactively wake the tunnel when the backend has new data"
    )]
    pub reactive_backend: bool,

    #[clap(
        long,
        value_name = "EVENT_SERVICE",
        help = "Additionally wake the tunnel when the named iceoryx2 event service fires (repeatable)"
    )]
    pub listener: Vec<String>,
}

impl Cli {
    /// The selected service-to-topic mapping.
    pub fn mapping(&self) -> Mapping {
        match &self.static_mapping {
            Some(config) => Mapping::Static(config.clone()),
            None => Mapping::Prefix,
        }
    }
}

/// How iceoryx2 services are mapped onto ROS 2 topics.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Mapping {
    /// By the `ros2://topics/` name prefix.
    Prefix,
    /// Per the entries of the static mapping TOML at the given path.
    Static(PathBuf),
}

#[derive(ValueEnum, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Translator {
    /// Payload bytes cross unmodified (serialized CDR on the iceoryx2 side).
    Passthrough,
    /// Fixed-size payloads cross as their rosidl C structs, transcoded to
    /// and from CDR via the type's introspection data.
    Introspection,
}
