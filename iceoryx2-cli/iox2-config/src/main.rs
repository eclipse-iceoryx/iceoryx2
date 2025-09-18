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

mod cli;
mod command;

use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use cli::Action;
use cli::Cli;
use iceoryx2_log::{set_log_level_from_env_or, LogLevel};

#[cfg(not(debug_assertions))]
use human_panic::setup_panic;

use crate::cli::GenerateSubcommand;
use crate::cli::ShowSubcommand;
#[cfg(debug_assertions)]
extern crate better_panic;

fn main() -> Result<()> {
    #[cfg(not(debug_assertions))]
    {
        setup_panic!();
    }
    #[cfg(debug_assertions)]
    {
        better_panic::Settings::debug()
            .most_recent_first(false)
            .lineno_suffix(true)
            .verbosity(better_panic::Verbosity::Full)
            .install();
    }

    set_log_level_from_env_or(LogLevel::Warn);

    let cli = Cli::parse();
    if let Some(action) = cli.action {
        match action {
            Action::Show { config } => match config {
                Some(ShowSubcommand::System) => {
                    if let Err(e) = command::show_system_config() {
                        eprintln!("Failed to show options: {e}");
                    }
                }
                Some(ShowSubcommand::Current) => {
                    if let Err(e) = command::show_current_config() {
                        eprintln!("Failed to show options: {e}");
                    }
                }
                None => {
                    Cli::command().print_help().expect("Failed to print help");
                }
            },
            Action::Generate { config, force } => match config {
                Some(GenerateSubcommand::Local) => {
                    if let Err(e) = command::generate_local(force) {
                        eprintln!("Failed to generate configuration file: {e}");
                    }
                }
                Some(GenerateSubcommand::Global) => {
                    if let Err(e) = command::generate_global(force) {
                        eprintln!("Failed to generate configuration file: {e}");
                    }
                }
                None => {
                    Cli::command().print_help().expect("Failed to print help");
                }
            },
            Action::Explain => {
                if let Err(e) = command::explain() {
                    eprintln!("Failed to display configuration description: {e}");
                }
            }
        }
    } else {
        Cli::command().print_help().expect("Failed to print help");
    }

    Ok(())
}
