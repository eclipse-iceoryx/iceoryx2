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

//! Reactive tunnel bridging `/chatter` (`std_msgs/msg/String`) between
//! iceoryx2 and ROS 2: woken by incoming data instead of polling, with a
//! timeout as the discovery cadence.
//!
//! ```bash
//! cargo run --example tunnel_reactive
//! # in other shells:
//! #   ros2 run demo_nodes_iceoryx2 talker
//! #   ros2 run demo_nodes_iceoryx2 listener
//! #   ros2 run demo_nodes_cpp talker
//! #   ros2 topic echo /chatter
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_tunnel_backend::{Config, Ros2Backend, TopicConfig};
use iceoryx2_services_tunnel::Tunnel;

/// Upper bound on the wake latency for discovery; propagation is
/// event-driven.
const DISCOVERY_INTERVAL: Duration = Duration::from_millis(500);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let backend_config = Config {
        topics: vec![TopicConfig::new("/chatter", "std_msgs/msg/String")?],
    };

    let (mut tunnel, listener) = Tunnel::<ipc::Service, Ros2Backend<ipc::Service>>::new()
        .backend_config(backend_config)
        .reactive()
        .create()?;

    while listener.timed_wait(|_| {}, DISCOVERY_INTERVAL).is_ok() {
        tunnel.discover()?;
        tunnel.propagate()?;
    }

    coutln!("exit");

    Ok(())
}
