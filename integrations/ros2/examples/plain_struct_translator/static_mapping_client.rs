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

//! Sends `example_interfaces/srv/AddTwoInts` requests on the service paired
//! with the ROS 2 service `/add_two_ints` in `static_mapping.toml` and prints
//! the responses. The application handles only the native
//! [`AddTwoIntsRequest`] and [`AddTwoIntsResponse`] structs; the gateway's
//! plain-struct translator does the CDR (de)serialization.
//!
//! ```bash
//! cargo run --manifest-path integrations/ros2/Cargo.toml --example plain_struct_static_mapping_client
//! # in other shells:
//! #   cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
//! #       --static-mapping integrations/ros2/examples/plain_struct_translator/static_mapping.toml \
//! #       --translator PlainStruct
//! #   ros2 run demo_nodes_cpp add_two_ints_server
//! ```

use core::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::plain_struct_translator::{
    AddTwoIntsRequest, AddTwoIntsResponse,
};

/// The iceoryx2 service paired with the ROS 2 service `/add_two_ints` in
/// `static_mapping.toml`.
const SERVICE_NAME: &str = "AddTwoInts";

const CYCLE_TIME: Duration = Duration::from_secs(1);

const A: i64 = 2;
const B: i64 = 3;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .request_response::<AddTwoIntsRequest, AddTwoIntsResponse>()
        .open_or_create()?;

    let client = service.client_builder().create()?;

    coutln!("sending: {A} + {B}");
    let mut pending_response = client.send_copy(AddTwoIntsRequest::new(A, B))?;

    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(response) = pending_response.receive()? {
            let AddTwoIntsResponse(payload) = response.payload();
            coutln!("received: {}", payload.sum);
        }

        coutln!("sending: {A} + {B}");
        pending_response = client.send_copy(AddTwoIntsRequest::new(A, B))?;
    }

    coutln!("exit");

    Ok(())
}
