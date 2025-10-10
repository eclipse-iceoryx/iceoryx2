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
mod supported_platform {

    #[cfg(not(debug_assertions))]
    use human_panic::setup_panic;
    use iceoryx2_bb_log::fail;
    #[cfg(debug_assertions)]
    extern crate better_panic;

    use super::cli;

    use clap::Parser;
    use cli::Cli;
    use cli::Transport;

    use iceoryx2::prelude::*;

    use iceoryx2_bb_log::info;
    use iceoryx2_bb_log::set_log_level_from_env_or;
    use iceoryx2_bb_log::warn;
    use iceoryx2_bb_log::LogLevel;

    use iceoryx2_tunnel::Tunnel;

    #[cfg(feature = "tunnel_zenoh")]
    use iceoryx2_tunnel_zenoh::ZenohBackend;

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

        // TODO(#1102): Organize into separate modules per-transport
        if let Some(transport) = cli.transport {
            match transport {
                Transport::Zenoh(zenoh_options) => {
                    #[cfg(feature = "tunnel_zenoh")]
                    {
                        let tunnel_config = iceoryx2_tunnel::Config {
                            discovery_service: cli.discovery_service,
                        };
                        let iceoryx_config = iceoryx2::config::Config::default();
                        let zenoh_config = match zenoh_options.zenoh_config {
                            Some(path) => zenoh::Config::from_file(&path).map_err(|e| {
                                anyhow::anyhow!("failed to read zenoh config file '{path}': {e}")
                            })?,
                            None => zenoh::Config::default(),
                        };

                        let tunnel = Tunnel::<ipc::Service, ZenohBackend<ipc::Service>>::create(
                            &tunnel_config,
                            &iceoryx_config,
                            &zenoh_config,
                        );
                        let mut tunnel = fail!(
                            from "iox2 tunnel",
                            when tunnel,
                            "Failed to create Tunnel"
                        );

                        let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

                        if cli.reactive {
                            // TODO(functionality): Make tunnel (or its endpoints) attachable to waitset
                            unimplemented!("Reactive mode is not yet supported.");
                        } else {
                            let rate = cli.poll.unwrap_or(100);
                            info!(from "iox2 tunnel", "Polling rate {}ms", rate);

                            let guard =
                                waitset.attach_interval(core::time::Duration::from_millis(rate))?;
                            let tick = WaitSetAttachmentId::from_guard(&guard);

                            let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
                                if id == tick {
                                    let _ = tunnel.discover().inspect_err(|e| {
                                        warn!(
                                            "Error encountered whilst discoverying services: {}",
                                            e
                                        );
                                    });
                                    let _ = tunnel.propagate().inspect_err(|e| {
                                        warn!(
                                            "Error encountered whilst propagating between hosts: {e}"
                                        );
                                    });
                                }
                                CallbackProgression::Continue
                            };

                            waitset.wait_and_process(on_event)?;
                        }
                    }
                    #[cfg(not(feature = "tunnel_zenoh"))]
                    {
                        println!("Zenoh transport is not available. Please rebuild with the 'tunnel_zenoh' feature enabled.");
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(not(target_os = "freebsd"))]
fn main() -> anyhow::Result<()> {
    supported_platform::main()
}
