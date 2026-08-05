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

//! Receives `std_msgs/msg/String` as CDR bytes from the service statically
//! mapped to the ROS 2 topic `/chatter` and prints it with the remote
//! origin from the [`RosHeader`]. The application does its own
//! (de)serialization.
//!
//! ```bash
//! ros2 run demo_nodes_iceoryx2 static_mapping_passthrough_translator_subscriber
//! # in other shells:
//! #   cargo run --bin iox2-tunnel-ros2 -- --static-mapping workspace/src/demo_nodes/static_mapping_chatter.toml
//! #   ros2 run demo_nodes_cpp talker
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use demo_nodes_iceoryx2::{RosHeader, StdMsgStringByte, as_bytes};

/// The iceoryx2 service paired with the ROS 2 topic `/chatter` in
/// `static_mapping_chatter.toml`.
const SERVICE_NAME: &str = "Chatter";

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<[StdMsgStringByte]>()
        .user_header::<RosHeader>()
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    coutln!("waiting for messages on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            let message: std_msgs::msg::String = cdr::deserialize(as_bytes(sample.payload()))?;
            let header = sample.user_header();

            coutln!(
                "received: \"{}\" (sequence: {}, timestamp: {} ns, gid: {:02x?})",
                message.data,
                header.sequence_number,
                header.source_timestamp_ns,
                header.gid
            );
        }
    }

    coutln!("exit");

    Ok(())
}
