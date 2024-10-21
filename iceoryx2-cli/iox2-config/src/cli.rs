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

use clap::Parser;
use clap::Subcommand;

use iceoryx2_cli::help_template;

#[derive(Parser)]
#[command(
    name = "iox2-config",
    about = "Query information about iceoryx2 configuration",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template("iox2 config", false),
)]
pub struct Cli {
    #[clap(subcommand)]
    pub action: Option<Action>,
}

#[derive(Parser)]
#[command(
    name = "iox2-config",
    about = "Query information about iceoryx2 configuration",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template("iox2 config show", false),
)]
pub struct Config {
    #[clap(subcommand)]
    pub action: Option<ShowSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum ShowSubcommand {
    #[clap(about = "Show system configuration")]
    System,
    #[clap(about = "Show current iceoryx2 configuration")]
    Current,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(about = "Show the currently used configuration")]
    Show {
        #[clap(subcommand)]
        subcommand: Option<ShowSubcommand>,
    },
    #[clap(about = "Generate a default configuration file")]
    Generate,
}
