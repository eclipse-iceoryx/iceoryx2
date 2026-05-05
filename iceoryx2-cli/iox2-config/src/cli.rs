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
use iceoryx2_cli::HelpOptions;
use iceoryx2_cli::help_template;

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

#[derive(Subcommand)]
pub enum ShowSubcommand {
    #[clap(
        about = "Show the system-wide configuration",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    System,
    #[clap(
        about = "Show the currently loaded configuration",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Current,
}

#[derive(Subcommand)]
pub enum GenerateSubcommand {
    #[clap(
        about = "Generate a local configuration file",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Local,
    #[clap(
        about = "Generate a global configuration file",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Global,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(
        about = "Show the currently used configuration",
        subcommand_required = true,
        arg_required_else_help = true,
        help_template = help_template(HelpOptions::PrintCommandSection)
    )]
    Show {
        #[clap(subcommand)]
        config: ShowSubcommand,
    },
    #[clap(
        about = "Generate a default configuration file",
        subcommand_required = true,
        arg_required_else_help = true,
        help_template = help_template(HelpOptions::PrintCommandSection)
    )]
    Generate {
        #[clap(subcommand)]
        config: GenerateSubcommand,
        #[clap(
            short,
            long,
            global = true,
            help = "Force overwrite existing configuration file"
        )]
        force: bool,
    },
    #[clap(
        about = "Explain the configuration parameters and their descriptions",
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Explain,
}
