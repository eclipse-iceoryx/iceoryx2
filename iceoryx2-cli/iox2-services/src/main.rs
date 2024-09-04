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

use clap::{CommandFactory, FromArgMatches};

#[cfg(not(debug_assertions))]
use human_panic::setup_panic;
#[cfg(debug_assertions)]
extern crate better_panic;

mod cli;

use cli::Action;

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

    match cli::Cli::command().try_get_matches() {
        Ok(matches) => {
            let parsed = cli::Cli::from_arg_matches(&matches).expect("Failed to parse arguments");
            match parsed.action {
                Some(action) => match action {
                    Action::List => {
                        println!("Listing all services...");
                    }
                    Action::Info { service } => {
                        println!("Getting information for service: {}", service);
                    }
                },
                None => {
                    cli::Cli::command().print_help().unwrap();
                }
            }
        }
        Err(err) => match err.kind() {
            _ => {
                eprintln!("{}", err);
                std::process::exit(1);
            }
        },
    }
}
