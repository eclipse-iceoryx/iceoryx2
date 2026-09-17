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

use clap::{Args, Parser, Subcommand};

use iceoryx2_cli::help_template;

#[derive(Parser)]
#[command(
    name = "iox2 link",
    bin_name = "iox2 link",
    about = "Launch a link extending iceoryx2 services across a boundary.",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    propagate_version = true,
    disable_help_subcommand = true,
    arg_required_else_help = true,
    help_template = help_template().with_subcommands().build(),
)]
pub struct Cli {
    #[command(subcommand)]
    pub backend: Backend,
}

#[derive(Subcommand)]
pub enum Backend {
    #[command(
        about = "Launch a tunnel to other iceoryx2 systems over a carrier.",
        long_about = None,
        disable_help_subcommand = true,
        help_template = help_template().with_external_command_hint().build(),
    )]
    Tunnel(TunnelArgs),

    #[command(
        about = "Launch a gateway to another middleware over an adapter.",
        long_about = None,
        disable_help_subcommand = true,
        help_template = help_template().with_external_command_hint().build(),
    )]
    Gateway(GatewayArgs),
}

#[derive(Args)]
pub struct TunnelArgs {
    #[arg(short, long, help = "List all installed tunnel CLIs")]
    pub list: bool,

    #[arg(
        short,
        long,
        help = "Display paths that will be checked for tunnel CLIs"
    )]
    pub paths: bool,

    #[arg(
        hide = true,
        required = false,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub invocation: Vec<String>,
}

#[derive(Args)]
pub struct GatewayArgs {
    #[arg(short, long, help = "List all installed gateway CLIs")]
    pub list: bool,

    #[arg(
        short,
        long,
        help = "Display paths that will be checked for gateway CLIs"
    )]
    pub paths: bool,

    #[arg(
        hide = true,
        required = false,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub invocation: Vec<String>,
}
