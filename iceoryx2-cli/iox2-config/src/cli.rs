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

use iceoryx2_bb_elementary::package_version::PackageVersion;
use iceoryx2_cli::help_template;
use iceoryx2_cli::HelpOptions;

#[derive(Parser)]
#[command(
    name = "iox2 config",
    bin_name = "iox2 config",
    about = "Query information about iceoryx2 configuration",
    long_about = None,
    version = PackageVersion::get_str(),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template(HelpOptions::PrintCommandSection),
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
    version = PackageVersion::get_str(),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template(HelpOptions::PrintCommandSection),
)]
pub struct ConfigShow {
    #[clap(subcommand)]
    pub action: Option<ShowSubcommand>,
}

#[derive(Parser)]
#[command(
    name = "iox2-config",
    about = "Query information about iceoryx2 configuration",
    long_about = None,
    version = PackageVersion::get_str(),
    disable_help_subcommand = true,
    arg_required_else_help = false,
    help_template = help_template(HelpOptions::PrintCommandSection),
)]
pub struct ConfigGenerate {
    #[clap(subcommand)]
    pub action: Option<GenerateSubcommand>,
}

#[derive(Subcommand, Debug)]
pub enum ShowSubcommand {
    #[clap(about = "Show system configuration")]
    System,
    #[clap(about = "Show current iceoryx2 configuration")]
    Current,
}

#[derive(Subcommand, Debug)]
pub enum GenerateSubcommand {
    #[clap(about = "Generate a default configuration file for the local user")]
    Local,
    #[clap(about = "Generate a default configuration file for the global system")]
    Global,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(about = "Show the currently used configuration", help_template = help_template(HelpOptions::DontPrintCommandSection))]
    Show {
        #[clap(subcommand)]
        subcommand: Option<ShowSubcommand>,
    },
    #[clap(about = "Generate a default configuration file", help_template = help_template(HelpOptions::DontPrintCommandSection))]
    Generate {
        #[clap(subcommand)]
        subcommand: Option<GenerateSubcommand>,
    },
}
