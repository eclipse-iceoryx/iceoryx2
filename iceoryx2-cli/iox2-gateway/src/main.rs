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

mod cli;
mod command;

use clap::Parser;
use cli::Cli;
use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::LogLevel;
use iceoryx2_log::set_log_level_from_env_or;

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();

    set_log_level_from_env_or(LogLevel::Info);

    let cli = Cli::parse();

    if cli.list {
        if let Err(e) = command::list() {
            eprintln!("Failed to list commands: {e}");
        }
    } else if cli.paths {
        if let Err(e) = command::paths() {
            eprintln!("Failed to list search paths: {e}");
        }
    } else if !cli.external_command.is_empty() {
        let command_name = &cli.external_command[0];
        let command_args = if cli.external_command.len() > 1 {
            Some(&cli.external_command[1..])
        } else {
            None
        };
        if let Err(e) = command::execute(command_name, command_args) {
            eprintln!("Failed to execute command: {e}");
        }
    } else {
        if let Err(e) = command::list() {
            eprintln!("Failed to list commands: {e}");
        }
    }

    Ok(())
}
