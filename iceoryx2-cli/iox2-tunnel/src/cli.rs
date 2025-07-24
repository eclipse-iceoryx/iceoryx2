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
use clap::Subcommand;

use iceoryx2_cli::help_template;
use iceoryx2_cli::HelpOptions;

#[derive(Parser)]
#[command(
    name = "iox2 tunnel",
    bin_name = "iox2 tunnel",
    about = "Launch a tunnel between iceoryx2 instances.",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template(HelpOptions::PrintCommandSection),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub transport: Option<Transport>,

    #[clap(
        long,
        short = 'd',
        global = true,
        help = "Optionally provide the name of a service providing discovery updates to connect to"
    )]
    pub discovery_service: Option<String>,

    #[clap(
        long,
        value_name = "RATE",
        global = true,
        conflicts_with = "reactive",
        help = "Periodically poll for discovery updates and samples at the provided rate (in milliseconds) [default]"
    )]
    pub poll: Option<u64>,

    #[clap(
        long,
        global = true,
        conflicts_with = "poll",
        help = "Reactively process discovery updates and samples"
    )]
    pub reactive: bool,
}

#[derive(Parser)]
pub struct ZenohOptions {
    #[clap(
        short,
        long,
        value_name = "PATH",
        help = "Path to a Zenoh configuration file to use to configure the Zenoh session used by the tunnel"
    )]
    pub zenoh_config: Option<String>,
}

#[derive(Subcommand)]
pub enum Transport {
    #[clap(
        about = "Use Zenoh as the transport",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Zenoh(ZenohOptions),
}
