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

//! Receives `sensor_msgs.msg.JointState` flatbuffers on the service the
//! gateway CLI's flatbuffer translator fills from the ROS 2 topic
//! `/joint_states`.
//!
//! ```bash
//! cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_joint_state_subscriber
//! # in other shells:
//! #   cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
//! #       --static-mapping integrations/ros2/examples/flatbuffer_translator/mapping.toml \
//! #       --translator Flatbuffer \
//! #       --schema-path integrations/ros2/examples/flatbuffer_translator/schemas
//! #   ros2 topic pub -r 1 /joint_states sensor_msgs/msg/JointState \
//! #       "{header: {frame_id: base}, name: [shoulder, elbow], position: [0.1, 0.2]}"
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::schemas::joint_state::sensor_msgs::msg::JointState;

/// The iceoryx2 service paired with the ROS 2 topic `/joint_states` in
/// `mapping.toml`.
const SERVICE_NAME: &str = "JointState";
const SCHEMA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flatbuffer_translator/schemas/joint_state.bfbs"
);

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<Flatbuffer<JointState>>()
        .flatbuffer_schema_path(&SCHEMA_PATH.try_into()?)
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    coutln!("waiting for joint states on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            let joint_state = sample.payload_root()?;
            let header = joint_state.header();
            let stamp = header.and_then(|header| header.stamp());
            coutln!(
                "received: frame {:?}, stamped {}.{:09}",
                header
                    .and_then(|header| header.frame_id())
                    .unwrap_or_default(),
                stamp.map(|stamp| stamp.sec()).unwrap_or_default(),
                stamp.map(|stamp| stamp.nanosec()).unwrap_or_default()
            );
            let names = joint_state.name().into_iter().flatten();
            let positions = joint_state.position().into_iter().flatten();
            for (name, position) in names.zip(positions) {
                coutln!("  {name}: {position:.3}");
            }
        }
    }

    coutln!("exit");

    Ok(())
}
