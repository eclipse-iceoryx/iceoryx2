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

#[cfg(target_os = "freebsd")]
fn main() {
    println!("Not supported on this platform");
}

#[cfg(not(target_os = "freebsd"))]
mod cli;

#[cfg(not(target_os = "freebsd"))]
mod command;

#[cfg(not(target_os = "freebsd"))]
mod supported_platform {

    use clap::CommandFactory;
    #[cfg(not(debug_assertions))]
    use human_panic::setup_panic;
    #[cfg(debug_assertions)]
    extern crate better_panic;

    use crate::cli;
    use crate::cli::Transport;
    use crate::command;

    use clap::Parser;
    use cli::Cli;

    use iceoryx2_log::set_log_level_from_env_or;
    use iceoryx2_log::LogLevel;

    pub fn main() -> anyhow::Result<()> {
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

        set_log_level_from_env_or(LogLevel::Info);

        let cli = match Cli::try_parse() {
            Ok(cli) => cli,
            Err(e) => {
                // --help and --version is treated as a parse error by clap
                // printing the error is actually printing the result of those commands ...
                let _ = e.print();
                return Ok(());
            }
        };

        // TODO(#1102): Organize into separate modules per-transport
        if let Some(transport) = cli.transport {
            match transport {
                Transport::Zenoh(zenoh_options) => {
                    #[cfg(feature = "tunnel_zenoh")]
                    command::zenoh(
                        zenoh_options.zenoh_config,
                        zenoh_options.common.reactive,
                        zenoh_options.common.discovery_service,
                        zenoh_options.common.poll,
                    )?;
                    #[cfg(not(feature = "tunnel_zenoh"))]
                    {
                        println!("Zenoh transport is not available. Please rebuild with the 'tunnel_zenoh' feature enabled.");
                        return Ok(());
                    }
                }
            }
        } else {
            Cli::command().print_help().expect("Failed to print help");
        }

        Ok(())
    }
}

#[cfg(not(target_os = "freebsd"))]
fn main() -> anyhow::Result<()> {
    supported_platform::main()
}
