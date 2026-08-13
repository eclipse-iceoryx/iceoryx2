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

//! Publishes `geometry_msgs/msg/Twist` on the service statically mapped to
//! the ROS 2 topic `/cmd_vel`. The application handles only the native
//! [`Twist`] struct; the gateway's plain-struct translator does the CDR
//! (de)serialization.
//!
//! ```bash
//! ros2 run demo_nodes_iceoryx2 static_mapping_plain_struct_translator_publisher
//! # in other shells:
//! #   cargo run --bin iox2-gateway-ros2 -- \
//! #       --static-mapping workspace/src/demo_nodes/static_mapping_cmdvel.toml \
//! #       --translator PlainStruct
//! #   ros2 topic echo /cmd_vel
//! ```

use core::time::Duration;

use demo_nodes_iceoryx2::Twist;
use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_interop::RosHeader;

/// The iceoryx2 service paired with the ROS 2 topic `/cmd_vel` in
/// `static_mapping_cmdvel.toml`.
const SERVICE_NAME: &str = "CmdVel";

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<Twist>()
        .user_header::<RosHeader>()
        .open_or_create()?;

    let publisher = service.publisher_builder().create()?;

    let mut counter = 1u64;
    while node.wait(CYCLE_TIME).is_ok() {
        let mut sample = publisher.loan()?;

        let twist = sample.payload_mut();
        twist.0.linear.x = 0.5;
        twist.0.angular.z = (counter % 10) as f64 * 0.1;

        coutln!("send: {:?}", sample.payload());
        sample.send()?;
        counter += 1;
    }

    coutln!("exit");

    Ok(())
}
