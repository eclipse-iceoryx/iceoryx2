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
mod commands;

use clap::CommandFactory;
use clap::Parser;
use cli::Cli;
use iceoryx2_bb_log::{set_log_level, LogLevel};
use iceoryx2_cli_utils::Format;

#[cfg(not(debug_assertions))]
use human_panic::setup_panic;
#[cfg(debug_assertions)]
extern crate better_panic;

fn main() {
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

    set_log_level(LogLevel::Warn);

    match Cli::try_parse() {
        Ok(cli) => {
            if let Some(action) = cli.action {
                match action {
                    cli::Action::List => {
                        if let Err(e) = commands::list(cli.format.unwrap_or(Format::Ron)) {
                            eprintln!("Failed to list nodes: {}", e);
                        }
                    }
                    cli::Action::Details(_) => todo!(),
                }
            } else {
                Cli::command().print_help().expect("Failed to print help");
            }
        }
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}
