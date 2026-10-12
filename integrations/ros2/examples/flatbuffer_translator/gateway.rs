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

    // The mapping maps `iceoryx2` service names to the equivalent used by the
    // remote mechanism, in this case ROS 2 topics. Here we use the static
    // mapping which is explicitly defined in a TOML file.
    let mapping: static_mapping::Config = toml::from_str(MAPPING)?;
    let mapping = StaticMapping::new(mapping)?;

    // The translator provides the mechanism for converting bytes in shared
    // memory to the bytes sent over ROS 2 and vice-versa. Here the custom
    // translator defined in this example is used.
    let translator = FlatbufferTranslator {
        // Configures whether the gateway creates services that mirror ROS 2
        // topics with a header containing the ROS 2 MessageInfo, or none.
        // This must be consistent across the system as `iceoryx2` will not
        // establish connections between services with different headers.
        //
        // Here, no header is used.
        header: MirroredHeader::None,
    };

    // The adapter is the gateway's connection to ROS 2.
    let adapter = Ros2Adapter::new(&AdapterConfig {
        // Configures the gateway to preload the typesupports for all types
        // defined by the mapping. If a typesupport is not found, the gateway
        // fails to initialize.
        //
        // If left empty (the default), all typesupports are dynamically loaded
        // on first use, returning an error if a required typesupport is not
        // found.
        preload_types: mapping.type_names(),
        // Disables publishing log output on /rosout.
        rosout: false,
    })?;

    // A node is created to hook into `iceoryx2` on this host and then linked
    // to ROS 2 via a gateway that uses the ROS 2 adapter to interface with ROS
    // 2, and the mapping and translator defined above.
    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let mut gateway = Link::new(node, Gateway::new(adapter, mapping, translator));

    coutln!("gateway running, Ctrl-C to stop");
    while gateway.node().wait(CYCLE_TIME).is_ok() {
        // Discovery checks for local services and remote topics that appeared
        // or vanished since the last cycle, and establishes connections
        // according to the mapping and translator.
        if let Err(error) = gateway.discover() {
            warn!(from ORIGIN, "Discovery failed: {}", error);
        }
        // Propagation moves the samples pending on every open pair in both
        // directions, transcoding between what is stored in shared memory
        // and what is sent over ROS 2.
        if let Err(error) = gateway.propagate() {
            warn!(from ORIGIN, "Propagation failed: {}", error);
        }
    }

    coutln!("exit");

    Ok(())
}
