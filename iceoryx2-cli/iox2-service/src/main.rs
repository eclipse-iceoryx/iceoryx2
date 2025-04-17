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

use anyhow::Result;
use clap::Parser;
use cli::Action;
use cli::Cli;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::set_log_level_from_env_or;
use iceoryx2_bb_log::LogLevel;

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

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // --help and --version is treated as a parse error by clap
            // printing the error is actually printing the result of those commands ...
            let _ = e.print();
            return Ok(());
        }
    };

    if let Some(action) = cli.action {
        match action {
            Action::List(options) => {
                if let Err(e) = commands::list(options.filter, cli.format) {
                    error!("Failed to list services: {}", e);
                }
            }
            Action::Details(options) => {
                if let Err(e) = commands::details(options.service, options.filter, cli.format) {
                    error!("Failed to retrieve service details: {}", e);
                }
            }
            Action::Monitor(options) => {
                let should_publish = options.disable_publish == false;
                let should_notify = options.disable_notify == false;
                if let Err(_e) = commands::monitor(
                    options.service_name.as_str(),
                    options.rate,
                    should_publish,
                    should_notify,
                ) {
                    error!("Failed to start service monitor")
                }
            }
        }
    }

    Ok(())
}
