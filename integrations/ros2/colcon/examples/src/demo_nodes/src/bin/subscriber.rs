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

//! A simple application built as a ROS 2 package with colcon.
//!
//! ```bash
//! ros2 run demo_nodes_iceoryx2 subscriber
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;

const SERVICE_NAME: &str = "Counter";

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .publish_subscribe::<u64>()
        .open_or_create()?;

    let subscriber = service.subscriber_builder().create()?;

    coutln!("waiting on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(sample) = subscriber.receive()? {
            coutln!("received {}", *sample);
        }
    }

    coutln!("exit");

    Ok(())
}
