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

//! Receives `geometry_msgs/msg/Twist` from the service statically mapped to
//! the ROS 2 topic `/cmd_vel`. The application handles only the native
//! [`Twist`] struct; the tunnel's introspection translator does the CDR
//! (de)serialization.
//!
//! ```bash
//! ros2 run demo_nodes_iceoryx2 static_mapping_introspection_translator_subscriber
//! # in other shells:
//! #   cargo run --bin iox2-tunnel-ros2 -- \
//! #       --static-mapping workspace/src/demo_nodes/static_mapping_cmdvel.toml \
//! #       --translator introspection
//! #   ros2 topic pub -r 1 /cmd_vel geometry_msgs/msg/Twist "{linear: {x: 0.5}}"
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use demo_nodes_iceoryx2::{RosHeader, Twist};

/// The iceoryx2 service paired with the ROS 2 topic `/cmd_vel` in
/// `static_mapping_cmdvel.toml`.
const SERVICE_NAME: &str = "CmdVel";

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<Twist>()
        .user_header::<RosHeader>()
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    coutln!("waiting for messages on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            let header = sample.user_header();

            coutln!(
                "received: {:?} (sequence: {}, timestamp: {} ns, gid: {:02x?})",
                sample.payload(),
                header.sequence_number,
                header.source_timestamp_ns,
                header.gid
            );
        }
    }

    coutln!("exit");

    Ok(())
}
