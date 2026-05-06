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
use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::{LogLevel, set_log_level_from_env_or};

use crate::cli::GenerateSubcommand;
use crate::cli::ShowSubcommand;

fn main() -> Result<()> {
    install_panic_handlers!();

    set_log_level_from_env_or(LogLevel::Warn);

    let cli = Cli::parse();
    if let Some(action) = cli.action {
        match action {
            Action::Show { config } => match config {
                ShowSubcommand::System => {
                    if let Err(e) = command::show_system_config() {
                        eprintln!("Failed to show options: {e}");
                    }
                }
                ShowSubcommand::Current => {
                    if let Err(e) = command::show_current_config() {
                        eprintln!("Failed to show options: {e}");
                    }
                }
            },
            Action::Generate { config, force } => match config {
                GenerateSubcommand::Local => {
                    if let Err(e) = command::generate_local(force) {
                        eprintln!("Failed to generate configuration file: {e}");
                    }
                }
                GenerateSubcommand::Global => {
                    if let Err(e) = command::generate_global(force) {
                        eprintln!("Failed to generate configuration file: {e}");
                    }
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
