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

#[cfg(not(debug_assertions))]
use human_panic::setup_panic;
#[cfg(debug_assertions)]
extern crate better_panic;

mod cli;
mod commands;
mod filter;
mod monitor;

use anyhow::{anyhow, Result};
use clap::CommandFactory;
use clap::Parser;
use cli::Action;
use cli::Cli;
use iceoryx2_bb_log::{set_log_level, LogLevel};

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

    set_log_level(LogLevel::Warn);

    let cli = Cli::try_parse().map_err(|e| anyhow!("{}", e))?;
    if let Some(action) = cli.action {
        match action {
            Action::List(options) => {
                if let Err(e) = commands::list(options.filter, cli.format) {
                    eprintln!("Failed to list services: {}", e);
                }
            }
            Action::Details(options) => {
                if let Err(e) = commands::details(options.service, options.filter, cli.format) {
                    eprintln!("Failed to retrieve service details: {}", e);
                }
            }
            Action::Monitor(options) => {
                if let Err(_e) = commands::monitor(options.rate) {
                    eprintln!("Failed to start service monitor")
                }
            }
        }
    } else {
        Cli::command().print_help().expect("Failed to print help");
    }

    Ok(())
}
