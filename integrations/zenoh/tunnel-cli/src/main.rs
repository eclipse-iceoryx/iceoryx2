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

use iceoryx2::node::Node;
use iceoryx2::port::listener::Listener;
use iceoryx2::prelude::*;
use iceoryx2::service::local_threadsafe;
use iceoryx2_cli::install_panic_handlers;
use iceoryx2_log::LogLevel;
use iceoryx2_log::fail;
use iceoryx2_log::info;
use iceoryx2_log::set_log_level_from_env_or;
use iceoryx2_log::warn;

use iceoryx2_integrations_zenoh_tunnel_backend::ZenohBackend;
use iceoryx2_services_tunnel::Config as TunnelConfig;
use iceoryx2_services_tunnel::Tunnel;

const ORIGIN: &str = "iox2-tunnel-zenoh";

type IpcTunnel = Tunnel<ipc::Service, ZenohBackend<ipc::Service>>;

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();

    set_log_level_from_env_or(LogLevel::Info);

    let cli = Cli::parse();

    info!(from ORIGIN, "Starting iox2-tunnel-zenoh v{}", env!("CARGO_PKG_VERSION"));

    if let Some(name) = &cli.discovery_service {
        info!(from ORIGIN, "Discovery service: {:?}", name);
    }

    let tunnel_config = TunnelConfig {
        discovery_service: cli.discovery_service,
        services: if cli.services.is_empty() {
            None
        } else {
            Some(cli.services)
        },
    };
    let iceoryx_config = iceoryx2::config::Config::default();
    let zenoh_config = parse_zenoh_config(cli.zenoh_config.as_deref())?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    // Polling defaults to 100ms only when no explicit wake source is given.
    // As soon as `--reactive-backend` or `--listener` is set, polling is
    // opt-in via `--poll`.
    let poll_rate = match cli.poll {
        Some(rate) => Some(rate),
        None if !cli.reactive_backend && cli.listener.is_empty() => Some(100),
        None => None,
    };
    let _interval_guard = match poll_rate {
        Some(rate) => {
            info!(from ORIGIN, "Polling at {}ms", rate);
            Some(waitset.attach_interval(core::time::Duration::from_millis(rate))?)
        }
        None => {
            info!(from ORIGIN, "Polling disabled");
            None
        }
    };

    let (mut tunnel, tunnel_listener) = create_tunnel(
        cli.reactive_backend,
        tunnel_config,
        iceoryx_config,
        zenoh_config,
    )?;
    let user_listeners = open_user_listeners(tunnel.node(), &cli.listener)?;

    let _tunnel_wake_guard = tunnel_listener
        .as_ref()
        .map(|l| waitset.attach_notification(l))
        .transpose()?;
    let _user_wake_guards: Vec<_> = user_listeners
        .iter()
        .map(|l| waitset.attach_notification(l))
        .collect::<Result<_, _>>()?;

    info!(from ORIGIN, "Tunnel running — Ctrl-C to stop");

    waitset.wait_and_process(|_id| {
        spin(&mut tunnel);
        CallbackProgression::Continue
    })?;

    info!(from ORIGIN, "Tunnel stopped");
    Ok(())
}

fn parse_zenoh_config(path: Option<&str>) -> anyhow::Result<zenoh::Config> {
    match path {
        Some(p) => {
            info!(from ORIGIN, "Loading zenoh config from {:?}", p);
            zenoh::Config::from_file(p)
                .map_err(|e| anyhow::anyhow!("failed to read zenoh config file '{p}': {e}"))
        }
        None => {
            info!(from ORIGIN, "Using default zenoh config");
            Ok(zenoh::Config::default())
        }
    }
}

fn open_user_listeners(
    node: &Node<ipc::Service>,
    names: &[String],
) -> anyhow::Result<Vec<Listener<ipc::Service>>> {
    names
        .iter()
        .map(|name| {
            let service_name = name.as_str().try_into().map_err(|e| {
                anyhow::anyhow!("invalid --listener service name {:?}: {:?}", name, e)
            })?;
            let service = node
                .service_builder(&service_name)
                .event()
                .open_or_create()
                .map_err(|e| {
                    anyhow::anyhow!("failed to open --listener event service {:?}: {}", name, e)
                })?;
            let listener = service.listener_builder().create()?;
            info!(from ORIGIN, "Listener: {:?}", name);
            Ok(listener)
        })
        .collect()
}

fn create_tunnel(
    reactive_backend: bool,
    tunnel_config: TunnelConfig,
    iceoryx_config: iceoryx2::config::Config,
    zenoh_config: zenoh::Config,
) -> anyhow::Result<(IpcTunnel, Option<Listener<local_threadsafe::Service>>)> {
    let builder = Tunnel::<ipc::Service, ZenohBackend<ipc::Service>>::new()
        .tunnel_config(tunnel_config)
        .iceoryx_config(iceoryx_config)
        .backend_config(zenoh_config);

    if reactive_backend {
        let (tunnel, listener) = fail!(
            from ORIGIN,
            when builder.reactive().create(),
            "Failed to create reactive Tunnel"
        );
        info!(from ORIGIN, "Reactive backend");
        Ok((tunnel, Some(listener)))
    } else {
        let tunnel = fail!(
            from ORIGIN,
            when builder.polled().create(),
            "Failed to create Tunnel"
        );
        Ok((tunnel, None))
    }
}

fn spin(tunnel: &mut IpcTunnel) {
    let _ = tunnel.discover().inspect_err(|e| {
        warn!("Error encountered whilst discovering services: {}", e);
    });
    let _ = tunnel.propagate().inspect_err(|e| {
        warn!("Error encountered whilst propagating between hosts: {e}");
    });
}
