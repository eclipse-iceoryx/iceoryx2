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

use std::path::Path;

use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::listener::Listener;
use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_link_adapter::mapping::static_mapping;
use iceoryx2_integrations_ros2_link_adapter::{
    AllowList, Config as AdapterConfig, MirroredHeader, PassthroughTranslator,
    PlainStructTranslator, PrefixMapping, Ros2Adapter, StaticMapping, TopicSettings, TopicTypes,
    TypeName,
};
use iceoryx2_link::{Link, WakeCreationError};
use iceoryx2_link_adapter::{Mapping, Translator};
use iceoryx2_link_backend::WakeService;
use iceoryx2_link_gateway::Gateway;
use iceoryx2_log::{fail, info, warn};

use crate::ORIGIN;
use crate::cli::{self, Cli};

type Ros2Gateway<M, T> = Link<ipc::Service, Gateway<Ros2Adapter, M, T>>;

/// The operations the CLI's main loop requires of a gateway regardless of
/// the mapping and translator configured.
pub trait GatewayInstance {
    fn node(&self) -> &Node<ipc::Service>;
    fn listener(&mut self) -> Result<Listener<WakeService>, WakeCreationError>;
    /// One round of discovery and propagation.
    fn spin(&mut self);
}

impl<M, T> GatewayInstance for Ros2Gateway<M, T>
where
    M: Mapping<EndpointSettings = TopicSettings>,
    T: Translator<EndpointTypes = TopicTypes>,
{
    fn node(&self) -> &Node<ipc::Service> {
        Link::node(self)
    }

    fn listener(&mut self) -> Result<Listener<WakeService>, WakeCreationError> {
        Link::listener(self)
    }

    fn spin(&mut self) {
        if let Err(error) = self.discover() {
            warn!(from ORIGIN, "Discovery failed: {}", error);
        }
        if let Err(error) = self.propagate() {
            warn!(from ORIGIN, "Propagation failed: {}", error);
        }
    }
}

/// Creates the gateway for the mapping and translator selected on the
/// command line.
pub fn create_gateway(
    cli: &Cli,
    iceoryx_config: &iceoryx2::config::Config,
) -> anyhow::Result<Box<dyn GatewayInstance>> {
    let header = mirrored_header(cli);
    let gateway: Box<dyn GatewayInstance> = match (cli.mapping(), cli.translator) {
        (cli::Mapping::Prefix, cli::Translator::Passthrough) => {
            let config = fail!(
                from ORIGIN,
                when prefix_adapter_config(cli),
                "Failed to configure the adapter"
            );
            Box::new(fail!(
                from ORIGIN,
                when create(
                    iceoryx_config,
                    &config,
                    PrefixMapping::new(allowlist(cli)),
                    PassthroughTranslator { header },
                    cli.monitor,
                ),
                "Failed to create the gateway"
            ))
        }
        (cli::Mapping::Prefix, cli::Translator::PlainStruct) => {
            let config = fail!(
                from ORIGIN,
                when prefix_adapter_config(cli),
                "Failed to configure the adapter"
            );
            Box::new(fail!(
                from ORIGIN,
                when create(
                    iceoryx_config,
                    &config,
                    PrefixMapping::new(allowlist(cli)),
                    PlainStructTranslator { header },
                    cli.monitor,
                ),
                "Failed to create the gateway"
            ))
        }
        (cli::Mapping::Static(path), cli::Translator::Passthrough) => {
            let mapping = fail!(
                from ORIGIN,
                when load_static_mapping(&path),
                "Failed to load the static mapping"
            );
            let config = static_adapter_config(&mapping);
            Box::new(fail!(
                from ORIGIN,
                when create(
                    iceoryx_config,
                    &config,
                    mapping,
                    PassthroughTranslator { header },
                    cli.monitor
                ),
                "Failed to create the gateway"
            ))
        }
        (cli::Mapping::Static(path), cli::Translator::PlainStruct) => {
            let mapping = fail!(
                from ORIGIN,
                when load_static_mapping(&path),
                "Failed to load the static mapping"
            );
            let config = static_adapter_config(&mapping);
            Box::new(fail!(
                from ORIGIN,
                when create(
                    iceoryx_config,
                    &config,
                    mapping,
                    PlainStructTranslator { header },
                    cli.monitor
                ),
                "Failed to create the gateway"
            ))
        }
    };
    Ok(gateway)
}

/// The user header of mirrored services selected on the command line.
fn mirrored_header(cli: &Cli) -> MirroredHeader {
    if cli.ros_header {
        MirroredHeader::RosHeader
    } else {
        MirroredHeader::None
    }
}

fn create<M, T>(
    iceoryx_config: &iceoryx2::config::Config,
    adapter_config: &AdapterConfig,
    mapping: M,
    translator: T,
    monitor: bool,
) -> anyhow::Result<Ros2Gateway<M, T>>
where
    M: Mapping<EndpointSettings = TopicSettings>,
    T: Translator<EndpointTypes = TopicTypes>,
{
    let node = fail!(
        from ORIGIN,
        when NodeBuilder::new().config(iceoryx_config).create::<ipc::Service>(),
        "Failed to create the gateway's node"
    );
    let adapter = fail!(
        from ORIGIN,
        when Ros2Adapter::new(adapter_config),
        "Failed to create the ROS 2 adapter"
    );
    let gateway = Link::new(node, Gateway::new(adapter, mapping, translator));
    if !monitor {
        return Ok(gateway);
    }
    info!(from ORIGIN, "Monitoring what the bridges move");
    Ok(gateway.with_monitoring())
}

/// The adapter configuration for the prefix mapping. It preloads the types
/// named on the command line.
fn prefix_adapter_config(cli: &Cli) -> anyhow::Result<AdapterConfig> {
    let mut preload_types = Vec::with_capacity(cli.preload_types.len());
    for name in &cli.preload_types {
        preload_types.push(fail!(
            from ORIGIN,
            when TypeName::new(name),
            "Invalid --preload-type {:?}", name
        ));
    }
    Ok(AdapterConfig {
        preload_types,
        ..adapter_config()
    })
}

/// The adapter configuration for a static mapping. It preloads the types
/// of the mapping's entries.
fn static_adapter_config(mapping: &StaticMapping) -> AdapterConfig {
    AdapterConfig {
        preload_types: mapping.type_names(),
        ..adapter_config()
    }
}

/// The common adapter configuration.
fn adapter_config() -> AdapterConfig {
    AdapterConfig {
        rosout: false,
        ..AdapterConfig::default()
    }
}

fn allowlist(cli: &Cli) -> AllowList {
    match cli.allow.is_empty() {
        true => AllowList::all(),
        false => AllowList::new(&cli.allow),
    }
}

fn load_static_mapping(path: &Path) -> anyhow::Result<StaticMapping> {
    info!(from ORIGIN, "Loading the static mapping from {:?}", path);
    let content = fail!(
        from ORIGIN,
        when std::fs::read_to_string(path),
        "Failed to read the static mapping file {:?}", path
    );
    let config: static_mapping::Config = fail!(
        from ORIGIN,
        when toml::from_str(&content),
        "Failed to parse the static mapping file {:?}", path
    );
    let mapping = fail!(
        from ORIGIN,
        when StaticMapping::new(config),
        "Invalid static mapping in {:?}", path
    );
    Ok(mapping)
}
