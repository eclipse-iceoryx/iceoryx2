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

use std::path::Path;

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

use iceoryx2_gateway::Config as GatewayConfig;
use iceoryx2_gateway::Gateway;
use iceoryx2_gateway_backend::traits::{Mapping, Passthrough, Translator};
use iceoryx2_integrations_ros2_gateway_backend::Config as BackendConfig;
use iceoryx2_integrations_ros2_gateway_backend::mapping::static_mapping;
use iceoryx2_integrations_ros2_gateway_backend::{
    AllowList, PlainStructTranslator, PrefixMapping, Ros2Backend, StaticMapping, TopicDescription,
    TypeName,
};

const ORIGIN: &str = "iox2-gateway-ros2";

type Ros2Gateway<M, T> = Gateway<ipc::Service, Ros2Backend<ipc::Service, M, T>>;

fn main() -> anyhow::Result<()> {
    install_panic_handlers!();

    set_log_level_from_env_or(LogLevel::Info);

    let cli = Cli::parse();

    info!(from ORIGIN, "Starting iox2-gateway-ros2 v{}", env!("CARGO_PKG_VERSION"));

    check_ros_environment()?;

    if let Some(name) = &cli.discovery_service {
        info!(from ORIGIN, "Discovery service: {:?}", name);
    }
    // Scope is decided by the mapping alone, so the gateway's own service
    // filter is left unset.
    let gateway_config = GatewayConfig {
        discovery_service: cli.discovery_service.clone(),
        services: None,
    };

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

    let (mut gateway, gateway_listener) = create_gateway(&cli, gateway_config)?;
    let user_listeners = open_user_listeners(gateway.node(), &cli.listener)?;

    let _gateway_wake_guard = gateway_listener
        .as_ref()
        .map(|listener| waitset.attach_notification(listener))
        .transpose()?;
    let _user_wake_guards: Vec<_> = user_listeners
        .iter()
        .map(|listener| waitset.attach_notification(listener))
        .collect::<Result<_, _>>()?;

    info!(from ORIGIN, "Gateway running. Ctrl-C to stop");

    waitset.wait_and_process(|_id| {
        gateway.spin();
        CallbackProgression::Continue
    })?;

    info!(from ORIGIN, "Gateway stopped");
    Ok(())
}

/// Resolves the mapping and translator selected on the command line and
/// builds the matching gateway.
#[allow(clippy::type_complexity)]
fn create_gateway(
    cli: &Cli,
    gateway_config: GatewayConfig,
) -> anyhow::Result<(
    Box<dyn GatewayHandle>,
    Option<Listener<local_threadsafe::Service>>,
)> {
    match (cli.mapping(), cli.translator) {
        (cli::Mapping::Prefix, cli::Translator::Passthrough) => {
            create_gateway_impl::<PrefixMapping, Passthrough<TopicDescription>>(
                cli.reactive_backend,
                PrefixMapping::new(AllowList::new(&cli.allow)),
                gateway_config,
                BackendConfig {
                    preload_types: parse_preload_types(&cli.preload_types)?,
                },
            )
        }
        (cli::Mapping::Prefix, cli::Translator::PlainStruct) => {
            create_gateway_impl::<PrefixMapping, PlainStructTranslator>(
                cli.reactive_backend,
                PrefixMapping::new(AllowList::new(&cli.allow)),
                gateway_config,
                BackendConfig {
                    preload_types: parse_preload_types(&cli.preload_types)?,
                },
            )
        }
        (cli::Mapping::Static(path), cli::Translator::Passthrough) => {
            let mapping = load_static_mapping(&path)?;
            let backend_config = BackendConfig {
                preload_types: mapping.type_names(),
            };
            create_gateway_impl::<StaticMapping, Passthrough<TopicDescription>>(
                cli.reactive_backend,
                mapping,
                gateway_config,
                backend_config,
            )
        }
        (cli::Mapping::Static(path), cli::Translator::PlainStruct) => {
            let mapping = load_static_mapping(&path)?;
            let backend_config = BackendConfig {
                preload_types: mapping.type_names(),
            };
            create_gateway_impl::<StaticMapping, PlainStructTranslator>(
                cli.reactive_backend,
                mapping,
                gateway_config,
                backend_config,
            )
        }
    }
}

#[allow(clippy::type_complexity)]
fn create_gateway_impl<M, T>(
    reactive_backend: bool,
    mapping: M,
    gateway_config: GatewayConfig,
    backend_config: BackendConfig,
) -> anyhow::Result<(
    Box<dyn GatewayHandle>,
    Option<Listener<local_threadsafe::Service>>,
)>
where
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
{
    let builder = Gateway::<ipc::Service, Ros2Backend<ipc::Service, M, T>>::new()
        .gateway_config(gateway_config)
        .iceoryx_config(iceoryx2::config::Config::default())
        .backend_config(backend_config)
        .mapping(mapping);

    if reactive_backend {
        let (gateway, listener) = fail!(
            from ORIGIN,
            when builder.reactive().create(),
            "Failed to create reactive Gateway"
        );
        info!(from ORIGIN, "Reactive backend");
        Ok((Box::new(gateway), Some(listener)))
    } else {
        let gateway = fail!(
            from ORIGIN,
            when builder.polled().create(),
            "Failed to create Gateway"
        );
        Ok((Box::new(gateway), None))
    }
}

/// Erases the concrete gateway type to simplify `main` logic.
trait GatewayHandle {
    fn node(&self) -> &Node<ipc::Service>;

    /// Advances discovery and propagation once.
    fn spin(&mut self);
}

impl<M, T> GatewayHandle for Ros2Gateway<M, T>
where
    M: Mapping<EndpointDescription = TopicDescription>,
    T: Translator<EndpointDescription = TopicDescription>,
{
    fn node(&self) -> &Node<ipc::Service> {
        Gateway::node(self)
    }

    fn spin(&mut self) {
        let _ = self.discover().inspect_err(|e| {
            warn!("Error encountered whilst discovering services: {}", e);
        });
        let _ = self.propagate().inspect_err(|e| {
            warn!("Error encountered whilst propagating between hosts: {e}");
        });
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

/// Parses repeated `--preload-type` values.
fn parse_preload_types(type_names: &[String]) -> anyhow::Result<Vec<TypeName>> {
    type_names
        .iter()
        .map(|entry| {
            TypeName::new(entry)
                .map_err(|error| anyhow::anyhow!("invalid --preload-type {entry:?}: {error}"))
        })
        .collect()
}

fn load_static_mapping(path: &Path) -> anyhow::Result<StaticMapping> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| anyhow::anyhow!("failed to read static mapping file {path:?}: {error}"))?;
    let config: static_mapping::Config = toml::from_str(&content)?;
    Ok(StaticMapping::new(config)?)
}

/// Fails fast when no sourced ROS 2 environment is detected, before any
/// rcl call can die in the dynamic loader.
fn check_ros_environment() -> anyhow::Result<()> {
    if std::env::var_os("AMENT_PREFIX_PATH").is_none() {
        anyhow::bail!(
            "no sourced ROS 2 environment detected (AMENT_PREFIX_PATH is unset); \
             source your ROS 2 setup and retry"
        );
    }
    Ok(())
}
