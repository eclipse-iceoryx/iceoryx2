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

use clap::ValueEnum;
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

#[derive(ValueEnum, Clone, Debug)]
pub enum ShowSubcommand {
    System,
    Current,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum GenerateSubcommand {
    Local,
    Global,
}

#[derive(Subcommand)]
pub enum Action {
    #[clap(
        about = "Show the currently used configuration", 
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Show {
        #[clap(value_enum, help = "Specify which configuration to show")]
        config: Option<ShowSubcommand>,
    },
    #[clap(
        about = "Generate a default configuration file", 
        help_template = help_template(HelpOptions::DontPrintCommandSection)
    )]
    Generate {
        #[clap(value_enum, help = "Specify what kind of configuration to generate")]
        config: Option<GenerateSubcommand>,
        #[clap(short, long, help = "Force overwrite existing configuration file")]
        force: bool,
    },
    #[clap(about = "Explain the configuration parameters and their descriptions", help_template = help_template(HelpOptions::DontPrintCommandSection))]
    Explain,
}
