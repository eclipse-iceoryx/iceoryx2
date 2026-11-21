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
    AllowList, Config as AdapterConfig, EndpointSettings, MessageType, MirroredHeader,
    PassthroughTranslator, PlainStructTranslator, PrefixMapping, Ros2Adapter, StaticMapping,
    TypeName,
};
use iceoryx2_integrations_ros2_link_translator_flatbuffer::{
    FlatbufferTranslator, type_mismatches,
};
use iceoryx2_link::{Link, WakeCreationError};
use iceoryx2_link_adapter::{Mapping, SampleShape, TranscodesSamples, Translator};
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
    M: Mapping<EndpointSettings = EndpointSettings>,
    T: Translator<SampleShape, RemoteTypes = MessageType, Transcoders: TranscodesSamples>,
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
    let gateway: Box<dyn GatewayInstance> = match (cli.mapping(), cli.translator) {
        (cli::Mapping::Prefix, cli::Translator::Passthrough) => {
            let (config, mapping) = prefix_mapping(cli, iceoryx_config)?;
            let translator = passthrough_translator(cli);
            Box::new(create(iceoryx_config, &config, mapping, translator, cli)?)
        }
        (cli::Mapping::Prefix, cli::Translator::PlainStruct) => {
            let (config, mapping) = prefix_mapping(cli, iceoryx_config)?;
            let translator = plain_struct_translator(cli);
            Box::new(create(iceoryx_config, &config, mapping, translator, cli)?)
        }
        (cli::Mapping::Prefix, cli::Translator::Flatbuffer) => {
            let (config, mapping) = prefix_mapping(cli, iceoryx_config)?;
            let translator = flatbuffer_translator(cli, iceoryx_config)?;
            Box::new(create(iceoryx_config, &config, mapping, translator, cli)?)
        }
        (cli::Mapping::Static(path), cli::Translator::Passthrough) => {
            let (config, mapping) = static_mapping(load_static_mapping_config(&path)?, &path)?;
            let translator = passthrough_translator(cli);
            Box::new(create(iceoryx_config, &config, mapping, translator, cli)?)
        }
        (cli::Mapping::Static(path), cli::Translator::PlainStruct) => {
            let (config, mapping) = static_mapping(load_static_mapping_config(&path)?, &path)?;
            let translator = plain_struct_translator(cli);
            Box::new(create(iceoryx_config, &config, mapping, translator, cli)?)
        }
        (cli::Mapping::Static(path), cli::Translator::Flatbuffer) => {
            let mapping_config = load_static_mapping_config(&path)?;
            for mismatch in type_mismatches(&mapping_config) {
                warn!(from ORIGIN, "{}", mismatch);
            }
            let (config, mapping) = static_mapping(mapping_config, &path)?;
            let translator = flatbuffer_translator(cli, iceoryx_config)?;
            Box::new(create(iceoryx_config, &config, mapping, translator, cli)?)
        }
    };

    Ok(gateway)
}

fn create<M, T>(
    iceoryx_config: &iceoryx2::config::Config,
    adapter_config: &AdapterConfig,
    mapping: M,
    translator: T,
    cli: &Cli,
) -> anyhow::Result<Ros2Gateway<M, T>>
where
    M: Mapping<EndpointSettings = EndpointSettings>,
    T: Translator<SampleShape, RemoteTypes = MessageType, Transcoders: TranscodesSamples>,
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
    let mut gateway = Link::new(node, Gateway::new(adapter, mapping, translator));
    if cli.monitor {
        info!(from ORIGIN, "Monitoring what the bridges move");
        gateway = gateway.with_monitoring();
    }
    if cli.notify {
        info!(from ORIGIN, "Notifying the event service named after each service with delivered samples");
        gateway = gateway.with_notifications();
    }
    Ok(gateway)
}

fn prefix_mapping(
    cli: &Cli,
    iceoryx_config: &iceoryx2::config::Config,
) -> anyhow::Result<(AdapterConfig, PrefixMapping)> {
    let config = fail!(
        from ORIGIN,
        when prefix_adapter_config(cli),
        "Failed to configure the adapter"
    );
    let mapping = PrefixMapping::new(allowlist(cli), iceoryx_config.clone());
    Ok((config, mapping))
}

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

fn allowlist(cli: &Cli) -> AllowList {
    match cli.allow.is_empty() {
        true => AllowList::all(),
        false => AllowList::new(&cli.allow),
    }
}

fn static_mapping(
    mapping_config: static_mapping::Config,
    path: &Path,
) -> anyhow::Result<(AdapterConfig, StaticMapping)> {
    let mapping = fail!(
        from ORIGIN,
        when StaticMapping::new(mapping_config),
        "Invalid static mapping in {:?}", path
    );
    let config = static_adapter_config(&mapping);
    Ok((config, mapping))
}

fn load_static_mapping_config(path: &Path) -> anyhow::Result<static_mapping::Config> {
    info!(from ORIGIN, "Loading the static mapping from {:?}", path);
    let content = fail!(
        from ORIGIN,
        when std::fs::read_to_string(path),
        "Failed to read the static mapping file {:?}", path
    );
    let config = fail!(
        from ORIGIN,
        when toml::from_str(&content),
        "Failed to parse the static mapping file {:?}", path
    );
    Ok(config)
}

fn static_adapter_config(mapping: &StaticMapping) -> AdapterConfig {
    AdapterConfig {
        preload_types: mapping.type_names(),
        ..adapter_config()
    }
}

fn adapter_config() -> AdapterConfig {
    AdapterConfig {
        rosout: false,
        ..AdapterConfig::default()
    }
}

fn passthrough_translator(cli: &Cli) -> PassthroughTranslator {
    PassthroughTranslator {
        header: mirrored_header(cli),
    }
}

fn plain_struct_translator(cli: &Cli) -> PlainStructTranslator {
    PlainStructTranslator {
        header: mirrored_header(cli),
    }
}

fn flatbuffer_translator(
    cli: &Cli,
    iceoryx_config: &iceoryx2::config::Config,
) -> anyhow::Result<FlatbufferTranslator> {
    let schema_path = match &cli.schema_path {
        Some(path) => {
            let path = fail!(
                from ORIGIN,
                when iceoryx2::prelude::Path::new(path.as_os_str().as_encoded_bytes()),
                "Invalid --schema-path {:?}", path
            );
            Some(path)
        }
        None => iceoryx_config.global.service.flatbuffer_schema_path,
    };
    match &schema_path {
        Some(path) => info!(from ORIGIN, "Looking up flatbuffer schemas in {:?}", path.as_str()),
        None => warn!(from ORIGIN, "No flatbuffer schema path, ROS 2 topics cannot be mirrored"),
    }
    Ok(FlatbufferTranslator {
        header: mirrored_header(cli),
        schema_path,
    })
}

fn mirrored_header(cli: &Cli) -> MirroredHeader {
    if cli.ros_header {
        MirroredHeader::RosHeader
    } else {
        MirroredHeader::None
    }
}
