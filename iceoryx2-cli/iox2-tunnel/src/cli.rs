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
    name = "iox2-tunnel",
    about = "Launch a tunnel between iceoryx2 instances.",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template("iox2 tunnel", HelpOptions::PrintCommandSection),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub transport: Option<Transport>,
}

#[derive(Parser)]
pub struct ZenohOptions {}

#[derive(Subcommand)]
pub enum Transport {
    #[clap(
        about = "Use Zenoh as the transport",
        help_template = help_template("iox2 tunnel zenoh", HelpOptions::DontPrintCommandSection)
    )]
    Zenoh(ZenohOptions),
}
