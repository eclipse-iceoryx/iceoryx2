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

use clap::Parser;

use iceoryx2_cli::help_template;

#[derive(Parser)]
#[command(
    name = "iox2 tunnel zenoh",
    bin_name = "iox2 tunnel zenoh",
    about = "Launch an iceoryx2 tunnel using Zenoh as the transport.",
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
        long,
        short = 'd',
        help = "Name of a service providing discovery updates to connect to"
    )]
    pub discovery_service: Option<String>,

    #[clap(
        long,
        value_name = "RATE",
        conflicts_with = "reactive",
        help = "Poll for discovery updates and samples at the provided rate in milliseconds [default: 100]"
    )]
    pub poll: Option<u64>,

    #[clap(
        long,
        conflicts_with = "poll",
        help = "Reactively process discovery updates and samples"
    )]
    pub reactive: bool,
}
