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

//! Polled tunnel bridging the iceoryx2 services and ROS 2 topics paired
//! in a `StaticMapping` TOML file (`mapping.toml` by default).
//!
//! ```bash
//! cargo run --example tunnel_static_mapping [-- <mapping.toml>]
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_tunnel_backend::Config as BackendConfig;
use iceoryx2_integrations_ros2_tunnel_backend::mapping;
use iceoryx2_integrations_ros2_tunnel_backend::{Ros2Backend, StaticMapping};
use iceoryx2_services_tunnel::Config as TunnelConfig;
use iceoryx2_services_tunnel::Tunnel;

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const DEFAULT_MAPPING_FILE: &str = "examples/mapping.toml";

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_MAPPING_FILE.to_string());

    let mapping_config: mapping::static_mapping::Config =
        toml::from_str(&std::fs::read_to_string(&path)?)?;
    let mapping = StaticMapping::new(mapping_config)?;

    let tunnel_config = TunnelConfig::default();
    let backend_config = BackendConfig {
        topics: mapping.topics(),
    };

    let mut tunnel = Tunnel::<ipc::Service, Ros2Backend<ipc::Service, StaticMapping>>::new()
        .tunnel_config(tunnel_config)
        .backend_config(backend_config)
        .mapping(mapping)
        .polled()
        .create()?;

    while tunnel.node().wait(POLL_INTERVAL).is_ok() {
        tunnel.discover()?;
        tunnel.propagate()?;
    }

    coutln!("exit");

    Ok(())
}
