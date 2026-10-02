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

//! Publishes [`Pose`] flatbuffers on the service the example gateway
//! propagates to the ROS 2 topic `/pose`.
//!
//! ```bash
//! cargo run --example flatbuffer_pose_publisher
//! # in other shells:
//! #   cargo run --example flatbuffer_gateway
//! #   ros2 topic echo /pose
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::flatbuffer_translator::schemas::{
    Pose, PoseArgs, Quat, Vec3,
};

const SERVICE_NAME: &str = "Pose";
const SCHEMA_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/flatbuffer_translator/schemas/pose.bfbs"
);

const CYCLE_TIME: Duration = Duration::from_secs(1);
const INITIAL_RESERVED_MEMORY: usize = 256;

const FRAME: &str = "world";

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<Flatbuffer<Pose>>()
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

        let x = counter as f64 * 0.5;
        let frame = builder.create_string(FRAME);
        let pose = Pose::create(
            builder,
            &PoseArgs {
                seconds: counter as i32,
                nanoseconds: 0,
                frame: Some(frame),
                position: Some(&Vec3::new(x, 0.0, 0.0)),
                rotation: Some(&Quat::new(0.0, 0.0, 0.0, 1.0)),
            },
        );

        let sample = sample.assume_init(pose);
        coutln!("send pose {counter}, x {x:.1}");
        sample.send()?;
    }

    coutln!("exit");

    Ok(())
}
