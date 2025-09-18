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
mod command;
mod filter;

use anyhow::Result;
use clap::Parser;
use cli::Action;
use cli::Cli;
use iceoryx2_log::error;
use iceoryx2_log::{set_log_level_from_env_or, LogLevel};

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
            Action::Notify(options) => {
                if let Err(e) = command::notify(options, cli.format) {
                    error!("failed to notify service: {}", e);
                }
            }
            Action::Listen(options) => {
                if let Err(e) = command::listen(options, cli.format) {
                    error!("failed to wait for notifications: {}", e);
                }
            }
            Action::List(options) => {
                if let Err(e) = command::list(options.filter, cli.format) {
                    error!("failed to list services: {}", e);
                }
            }
            Action::Details(options) => {
                if let Err(e) = command::details(options.service, options.filter, cli.format) {
                    error!("failed to retrieve service details: {}", e);
                }
            }
            Action::Publish(options) => {
                if let Err(e) = command::publish(options, cli.format) {
                    error!("failed to publish messages: {}", e);
                }
            }
            Action::Subscribe(options) => {
                if let Err(e) = command::subscribe(options, cli.format) {
                    error!("failed to subscribe and receive messages: {}", e);
                }
            }
            Action::Record(options) => {
                if let Err(e) = command::record(options, cli.format) {
                    error!("failed to record data: {}", e);
                }
            }
            Action::Replay(options) => {
                if let Err(e) = command::replay(options, cli.format) {
                    error!("failed to replay data: {}", e);
                }
            }
            Action::Discovery(options) => {
                let should_publish = !options.disable_publish;
                let should_notify = !options.disable_notify;
                if let Err(e) = command::discovery(
                    options.rate,
                    options.detailed,
                    should_publish,
                    options.max_subscribers,
                    should_notify,
                    options.max_listeners,
                    cli.format,
                ) {
                    error!("failed to run service discovery: {:#}", e)
                }
            }
        }
    }

    Ok(())
}
