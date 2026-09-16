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
use clap::Parser;
use iceoryx2_cli::help_template;

#[derive(Parser)]
#[command(
    name = "iox2 link tunnel zenoh",
    bin_name = "iox2 link tunnel zenoh",
    about = "Launch an iceoryx2 tunnel to other iceoryx2 systems over zenoh.",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    help_template = help_template().build(),
)]
pub struct Cli {
    #[clap(
        short,
        long,
        value_name = "PATH",
        help = "Path to a zenoh configuration file"
    )]
    pub zenoh_config: Option<String>,

    #[clap(
        long = "allow",
        short = 'a',
        value_name = "SERVICE",
        action = clap::ArgAction::Append,
        help = "Bridge iceoryx2 services whose names match this wildcard pattern, where '*' \
                matches zero or more characters and '?' matches one. Repeatable. When omitted, \
                all services are bridged."
    )]
    pub allow: Vec<String>,

    #[clap(
        long,
        value_name = "RATE",
        help = "Polling rate in milliseconds for discovery and sample propagation \
                (defaults to 100ms when no other wake source is given; otherwise \
                must be set explicitly to enable polling)"
    )]
    pub poll: Option<u64>,

    #[clap(long, help = "Wake the tunnel when the peers have new data")]
    pub reactive: bool,

    #[clap(
        long,
        value_name = "EVENT_SERVICE",
        help = "Additionally wake the tunnel when the named iceoryx2 event service fires (repeatable)"
    )]
    pub listener: Vec<String>,

    #[clap(
        long,
        help = "Report what every bridge moved after each propagation, at trace level"
    )]
    pub monitor: bool,
}
