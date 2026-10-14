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

//! Receives [`JointReadings`] flatbuffers on the service the example
//! gateway fills from the ROS 2 topic `/joint_states`.
//!
//! ```bash
//! cargo run --example flatbuffer_joint_readings_subscriber
//! # in other shells:
//! #   cargo run --example flatbuffer_gateway
//! #   ros2 topic pub -r 1 /joint_states sensor_msgs/msg/JointState \
//! #       "{header: {frame_id: base}, name: [shoulder, elbow], position: [0.1, 0.2]}"
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::schemas::JointReadings;

const SERVICE_NAME: &str = "JointReadings";
const SCHEMA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flatbuffer_translator/schemas/joint_readings.bfbs"
);

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<Flatbuffer<JointReadings>>()
        .flatbuffer_schema_path(&SCHEMA_PATH.try_into()?)
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    coutln!("waiting for readings on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            let readings = sample.payload_root()?;
            let captured_at = readings.captured_at();
            coutln!(
                "received: frame {:?}, captured at {}.{:09}",
                readings.frame(),
                captured_at.seconds(),
                captured_at.nanoseconds()
            );
            for (joint, angle) in readings.joints().iter().zip(readings.angles()) {
                coutln!("  {joint}: {angle:.3}");
            }
        }
    }

    coutln!("exit");

    Ok(())
}
