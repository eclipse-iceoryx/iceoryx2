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

//! Native iceoryx2 publisher of [`twist::Twist`] structs on
//! `ros2://topics/cmd_vel` — no CDR in sight; the tunnel translates.
//!
//! ```bash
//! cargo run --example translation_twist_talker
//! ```

mod twist;

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_tunnel_backend::ros_header::RosHeader;

use twist::{Twist, Vector3};

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;
    let service = node
        .service_builder(&"ros2://topics/cmd_vel".try_into()?)
        .publish_subscribe::<Twist>()
        .user_header::<RosHeader>()
        .open_or_create()?;
    let publisher = service.publisher_builder().create()?;

    let mut counter = 0u32;
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        let twist = Twist {
            linear: Vector3 {
                x: f64::from(counter) * 0.1,
                ..Default::default()
            },
            angular: Vector3 {
                z: 0.5,
                ..Default::default()
            },
        };

        let mut sample = publisher.loan_uninit()?;
        // Outgoing samples carry no origin information; the header exists
        // so that the service type matches the bridged subscriber side.
        *sample.user_header_mut() = RosHeader::default();
        let sample = sample.write_payload(twist);
        sample.send()?;

        coutln!("sent: {:?}", twist);
    }

    coutln!("exit");

    Ok(())
}
