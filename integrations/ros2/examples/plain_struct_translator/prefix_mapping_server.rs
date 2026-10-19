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

//! Answers `example_interfaces/srv/AddTwoInts` requests on the service
//! prefix-mapped to the ROS 2 service `/add_two_ints`. The application
//! handles only the native [`AddTwoIntsRequest`] and [`AddTwoIntsResponse`]
//! structs; the gateway's plain-struct translator does the CDR
//! (de)serialization.
//!
//! ```bash
//! cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_prefix_mapping_server
//! # in other shells:
//! #   cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
//! #       --allow /add_two_ints \
//! #       --translator PlainStruct
//! #   ros2 service call /add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 3}"
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::plain_struct_translator::{
    AddTwoIntsRequest, AddTwoIntsResponse,
};

/// The iceoryx2 service mapped by the name prefix to the ROS 2 service
/// `/add_two_ints`.
const SERVICE_NAME: &str = "ros2://services/add_two_ints";

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .request_response::<AddTwoIntsRequest, AddTwoIntsResponse>()
        .open_or_create()?;

    let server = service.server_builder().create()?;

    coutln!("waiting for requests on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(request) = server.receive()? {
            let AddTwoIntsRequest(payload) = request.payload();
            let sum = payload.a + payload.b;
            coutln!(
                "received: {} + {}, responding: {}",
                payload.a,
                payload.b,
                sum
            );

            request.send_copy(AddTwoIntsResponse::new(sum))?;
        }
    }

    coutln!("exit");

    Ok(())
}
