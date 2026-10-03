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

//! A gateway propagating the flatbuffer services of the example to ROS 2
//! through the hand-written [`FlatbufferTranslator`], under the static
//! mapping in `mapping.toml`.
//!
//! ```bash
//! cargo run --example flatbuffer_gateway
//! # in other shells:
//! #   cargo run --example flatbuffer_joint_readings_publisher
//! #   ros2 topic echo /joint_states
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::translator::FlatbufferTranslator;
use iceoryx2_integrations_ros2_link_adapter::mapping::static_mapping;
use iceoryx2_integrations_ros2_link_adapter::{
    Config as AdapterConfig, MirroredHeader, Ros2Adapter, StaticMapping,
};
use iceoryx2_link::Link;
use iceoryx2_link_gateway::Gateway;
use iceoryx2_log::warn;

const ORIGIN: &str = "flatbuffer_gateway";

const MAPPING: &str = include_str!("mapping.toml");
const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let mapping: static_mapping::Config = toml::from_str(MAPPING)?;
    let mapping = StaticMapping::new(mapping)?;
    let translator = FlatbufferTranslator {
        header: MirroredHeader::None,
    };
    let adapter = Ros2Adapter::new(&AdapterConfig {
        preload_types: mapping.type_names(),
        rosout: false,
    })?;

    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let mut gateway = Link::new(node, Gateway::new(adapter, mapping, translator));

    coutln!("gateway running, Ctrl-C to stop");
    while gateway.node().wait(CYCLE_TIME).is_ok() {
        if let Err(error) = gateway.discover() {
            warn!(from ORIGIN, "Discovery failed: {}", error);
        }
        if let Err(error) = gateway.propagate() {
            warn!(from ORIGIN, "Propagation failed: {}", error);
        }
    }

    coutln!("exit");

    Ok(())
}
