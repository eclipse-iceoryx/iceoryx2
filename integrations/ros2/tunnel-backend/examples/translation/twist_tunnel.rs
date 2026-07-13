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

//! Polled tunnel bridging `/cmd_vel` (`geometry_msgs/msg/Twist`) with
//! payload translation: local applications exchange the native `Twist`
//! struct while CDR crosses the wire.
//!
//! ```bash
//! cargo run --example translation_twist_tunnel
//! # in other shells:
//! #   cargo run --example translation_twist_talker
//! #   ros2 topic echo /cmd_vel
//! ```

mod twist;

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_tunnel_backend::Config as BackendConfig;
use iceoryx2_integrations_ros2_tunnel_backend::{PrefixMapping, Ros2Backend, TopicConfig};
use iceoryx2_services_tunnel::Tunnel;

use twist::{TWIST_TYPE_NAME, TwistTranslator};

const POLL_INTERVAL: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let backend_config = BackendConfig {
        topics: vec![TopicConfig::new("/cmd_vel", TWIST_TYPE_NAME)?],
    };

    let mut tunnel =
        Tunnel::<ipc::Service, Ros2Backend<ipc::Service, PrefixMapping, TwistTranslator>>::new()
            .backend_config(backend_config)
            .polled()
            .create()?;

    while tunnel.node().wait(POLL_INTERVAL).is_ok() {
        tunnel.discover()?;
        tunnel.propagate()?;
    }

    coutln!("exit");

    Ok(())
}
