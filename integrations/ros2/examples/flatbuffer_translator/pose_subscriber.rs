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

//! Receives [`Pose`] flatbuffers on the service the example gateway fills
//! from the ROS 2 topic `/pose`.
//!
//! ```bash
//! cargo run --example flatbuffer_pose_subscriber
//! # in other shells:
//! #   cargo run --example flatbuffer_gateway
//! #   ros2 topic pub -r 1 /pose geometry_msgs/msg/PoseStamped \
//! #       "{header: {frame_id: world}, pose: {position: {x: 1.0}, orientation: {w: 1.0}}}"
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::schemas::Pose;

const SERVICE_NAME: &str = "Pose";
const SCHEMA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flatbuffer_translator/schemas/pose.bfbs"
);

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<Flatbuffer<Pose>>()
        .flatbuffer_schema_path(&SCHEMA_PATH.try_into()?)
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    coutln!("waiting for poses on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            let pose = sample.payload_root()?;
            let position = pose.position();
            let rotation = pose.rotation();
            coutln!(
                "received: frame {:?} at {}.{:09}, position {:?}, rotation {:?}",
                pose.frame(),
                pose.seconds(),
                pose.nanoseconds(),
                (position.x(), position.y(), position.z()),
                (rotation.x(), rotation.y(), rotation.z(), rotation.w())
            );
        }
    }

    coutln!("exit");

    Ok(())
}
