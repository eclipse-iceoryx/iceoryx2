// Copyright (c) 2026 Contributors to the Eclipse Foundation
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
use cli::{Backend, Cli};
use command::{GATEWAY_BACKEND, GATEWAY_EMPTY, TUNNEL_BACKEND, TUNNEL_EMPTY};
use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::{LogLevel, set_log_level_from_env_or};

const ORIGIN: &str = "iox2-link";

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();

    set_log_level_from_env_or(LogLevel::Info);

    let cli = Cli::parse();
    match &cli.backend {
        Backend::Tunnel(args) => run(
            TUNNEL_BACKEND,
            TUNNEL_EMPTY,
            args.list,
            args.paths,
            &args.invocation,
        ),
        Backend::Gateway(args) => run(
            GATEWAY_BACKEND,
            GATEWAY_EMPTY,
            args.list,
            args.paths,
            &args.invocation,
        ),
    }

    Ok(())
}

fn run(backend: &str, when_empty: &str, list: bool, paths: bool, invocation: &[String]) {
    if list {
        if let Err(e) = command::list(backend, when_empty) {
            eprintln!("Failed to list commands: {e}");
        }
    } else if paths {
        if let Err(e) = command::paths(backend) {
            eprintln!("Failed to list search paths: {e}");
        }
    } else if let [implementation, args @ ..] = invocation {
        let args = (!args.is_empty()).then_some(args);
        if let Err(e) = command::execute(backend, implementation, args) {
            eprintln!("Failed to execute command: {e}");
            std::process::exit(1);
        }
    } else if let Err(e) = command::list(backend, when_empty) {
        eprintln!("Failed to list commands: {e}");
    }
}
