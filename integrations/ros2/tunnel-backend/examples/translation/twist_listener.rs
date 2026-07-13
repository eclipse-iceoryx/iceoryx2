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

//! Native iceoryx2 subscriber of [`twist::Twist`] structs on
//! `ros2://topics/cmd_vel` — reads fields directly from shared memory; the
//! tunnel translates the CDR arriving from ROS 2.
//!
//! ```bash
//! cargo run --example translation_twist_listener
//! ```

mod twist;

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_tunnel_backend::ros_header::RosHeader;

use twist::Twist;

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let service = node
        .service_builder(&"ros2://topics/cmd_vel".try_into()?)
        .publish_subscribe::<Twist>()
        .user_header::<RosHeader>()
        .open_or_create()?;
    let subscriber = service.subscriber_builder().create()?;

    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            coutln!(
                "received: {:?} (sequence: {})",
                *sample,
                sample.user_header().sequence_number
            );
        }
    }

    coutln!("exit");

    Ok(())
}
