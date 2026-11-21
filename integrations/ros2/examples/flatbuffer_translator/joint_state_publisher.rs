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

//! Publishes `sensor_msgs.msg.JointState` flatbuffers on the service the
//! gateway CLI's flatbuffer translator propagates to the ROS 2 topic
//! `/joint_states`.
//!
//! ```bash
//! cargo run --manifest-path integrations/ros2/Cargo.toml --example flatbuffer_joint_state_publisher
//! # in other shells:
//! #   cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
//! #       --static-mapping integrations/ros2/examples/flatbuffer_translator/mapping.toml \
//! #       --translator Flatbuffer \
//! #       --schema-path integrations/ros2/examples/flatbuffer_translator/schemas
//! #   ros2 topic echo /joint_states
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::schemas::joint_state::builtin_interfaces::msg::Time;
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::schemas::joint_state::sensor_msgs::msg::{
    JointState, JointStateArgs,
};
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::schemas::joint_state::std_msgs::msg::{
    Header, HeaderArgs,
};

/// The iceoryx2 service paired with the ROS 2 topic `/joint_states` in
/// `mapping.toml`.
const SERVICE_NAME: &str = "JointState";
const SCHEMA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flatbuffer_translator/schemas/joint_state.bfbs"
);

const CYCLE_TIME: Duration = Duration::from_secs(1);
const INITIAL_RESERVED_MEMORY: usize = 256;

const FRAME_ID: &str = "base";
const NAMES: [&str; 2] = ["shoulder", "elbow"];

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<Flatbuffer<JointState>>()
        .flatbuffer_schema_path(&SCHEMA_PATH.try_into()?)
        .open_or_create()?;

    let publisher = service
        .publisher_builder()
        .initial_reserved_memory(INITIAL_RESERVED_MEMORY)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    let mut counter: u64 = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        let mut sample = publisher.loan_flatbuffer()?;
        let builder = sample.flatbuffer_builder();

        let angle = counter as f64 * 0.1;
        let frame_id = builder.create_string(FRAME_ID);
        let header = Header::create(
            builder,
            &HeaderArgs {
                stamp: Some(&Time::new(counter as i32, 0)),
                frame_id: Some(frame_id),
            },
        );
        let names: Vec<_> = NAMES
            .iter()
            .map(|name| builder.create_string(name))
            .collect();
        let name = builder.create_vector(&names);
        let position = builder.create_vector(&[angle, -angle]);
        let velocity = builder.create_vector(&[0.1, -0.1]);
        let effort = builder.create_vector(&[1.0, 0.5]);
        let joint_state = JointState::create(
            builder,
            &JointStateArgs {
                header: Some(header),
                name: Some(name),
                position: Some(position),
                velocity: Some(velocity),
                effort: Some(effort),
            },
        );

        let sample = sample.assume_init(joint_state);
        coutln!("send joint state {counter}, angle {angle:.1}");
        sample.send()?;
    }

    coutln!("exit");

    Ok(())
}
