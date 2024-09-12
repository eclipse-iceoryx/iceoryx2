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
use colored::*;

#[derive(Parser, Debug)]
#[command(
    name = "iox2",
    about = "The command-line interface to iceoryx2",
    long_about = None,
    version = env!("CARGO_PKG_VERSION"),
    disable_help_subcommand = true,
    arg_required_else_help = true,
    help_template = help_template(),
)]
pub struct Cli {
    #[arg(short, long, help = "List all installed commands")]
    pub list: bool,

    #[arg(
        short,
        long,
        help = "Display paths that will be checked for installed commands"
    )]
    pub paths: bool,

    #[arg(
        short,
        long,
        help = "Specify to execute development versions of commands if they exist"
    )]
    pub dev: bool,

    #[arg(
        hide = true,
        required = false,
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub external_command: Vec<String>,
}

fn help_template() -> String {
    format!(
        "{}{}{}\n\n{}\n{{options}}\n\n{}\n{{subcommands}}{}{}",
        "Usage: ".bright_green().bold(),
        "iox2 ".bold(),
        "[OPTIONS] [COMMAND]",
        "Options:".bright_green().bold(),
        "Commands:".bright_green().bold(),
        "  ...         ".bold(),
        "See all installed commands with --list"
    )
}
