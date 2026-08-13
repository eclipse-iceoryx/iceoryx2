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

//! Publishes `std_msgs/msg/String` as CDR bytes on the service statically
//! mapped to the ROS 2 topic `/chatter`. The application does its own
//! (de)serialization; the gateway's passthrough translator forwards the
//! bytes unmodified.
//!
//! ```bash
//! ros2 run demo_nodes_iceoryx2 static_mapping_passthrough_translator_publisher
//! # in other shells:
//! #   cargo run --bin iox2-gateway-ros2 -- --static-mapping workspace/src/demo_nodes/static_mapping_chatter.toml
//! #   ros2 topic echo /chatter
//! ```

use core::time::Duration;

use cdr::{CdrLe, Infinite};
use demo_nodes_iceoryx2::StdMsgStringByte;
use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_interop::RosHeader;

/// The iceoryx2 service paired with the ROS 2 topic `/chatter` in
/// `static_mapping_chatter.toml`.
const SERVICE_NAME: &str = "Chatter";

const CYCLE_TIME: Duration = Duration::from_secs(1);
const INITIAL_MAX_PAYLOAD_SIZE: usize = 64;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<[StdMsgStringByte]>()
        .user_header::<RosHeader>()
        .open_or_create()?;

    let publisher = service
        .publisher_builder()
        .initial_max_slice_len(INITIAL_MAX_PAYLOAD_SIZE)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    let mut counter = 1u64;
    while node.wait(CYCLE_TIME).is_ok() {
        let message = std_msgs::msg::String {
            data: format!("Hello from iceoryx2: {counter}"),
        };
        let payload = cdr::serialize::<_, _, CdrLe>(&message, Infinite)?;

        let sample = publisher.loan_slice_uninit(payload.len())?;
        let sample = sample.write_from_fn(|index| StdMsgStringByte(payload[index]));

        coutln!("send: \"{}\" ({} bytes)", message.data, payload.len());
        sample.send()?;
        counter += 1;
    }

    coutln!("exit");

    Ok(())
}
