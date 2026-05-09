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

use clap::Parser;

use cli::Cli;

use iceoryx2::prelude::*;
use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::LogLevel;
use iceoryx2_log::fail;
use iceoryx2_log::info;
use iceoryx2_log::set_log_level_from_env_or;
use iceoryx2_log::warn;

use iceoryx2_integrations_zenoh_tunnel_backend::ZenohBackend;
use iceoryx2_services_tunnel::Config as TunnelConfig;
use iceoryx2_services_tunnel::Tunnel;

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();

    set_log_level_from_env_or(LogLevel::Info);

    let cli = Cli::parse();

    let tunnel_config = TunnelConfig {
        discovery_service: cli.discovery_service,
        services: if cli.services.is_empty() {
            None
        } else {
            Some(cli.services)
        },
    };
    let iceoryx_config = iceoryx2::config::Config::default();
    let zenoh_config = match cli.zenoh_config {
        Some(path) => zenoh::Config::from_file(&path)
            .map_err(|e| anyhow::anyhow!("failed to read zenoh config file '{path}': {e}"))?,
        None => zenoh::Config::default(),
    };

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    if cli.reactive {
        let result = Tunnel::<ipc::Service, ZenohBackend<ipc::Service>>::new()
            .tunnel_config(tunnel_config)
            .iceoryx_config(iceoryx_config)
            .backend_config(zenoh_config)
            .reactive()
            .create();
        let (mut tunnel, listener) = fail!(
            from "iox2-tunnel-zenoh",
            when result,
            "Failed to create reactive Tunnel"
        );

        let _guard = waitset.attach_notification(&listener)?;
        info!(from "iox2-tunnel-zenoh", "Reactive mode");

        waitset.wait_and_process(|_id| {
            let _ = tunnel.discover().inspect_err(|e| {
                warn!("Error encountered whilst discovering services: {}", e);
            });
            let _ = tunnel.propagate().inspect_err(|e| {
                warn!("Error encountered whilst propagating between hosts: {e}");
            });
            CallbackProgression::Continue
        })?;
    } else {
        let result = Tunnel::<ipc::Service, ZenohBackend<ipc::Service>>::new()
            .tunnel_config(tunnel_config)
            .iceoryx_config(iceoryx_config)
            .backend_config(zenoh_config)
            .polled()
            .create();
        let mut tunnel = fail!(
            from "iox2-tunnel-zenoh",
            when result,
            "Failed to create Tunnel"
        );

        let rate = cli.poll.unwrap_or(100);
        info!(from "iox2-tunnel-zenoh", "Polling rate {}ms", rate);

        let guard = waitset.attach_interval(core::time::Duration::from_millis(rate))?;
        let tick = WaitSetAttachmentId::from_guard(&guard);

        waitset.wait_and_process(|id| {
            if id == tick {
                let _ = tunnel.discover().inspect_err(|e| {
                    warn!("Error encountered whilst discovering services: {}", e);
                });
                let _ = tunnel.propagate().inspect_err(|e| {
                    warn!("Error encountered whilst propagating between hosts: {e}");
                });
            }
            CallbackProgression::Continue
        })?;
    }

    Ok(())
}
