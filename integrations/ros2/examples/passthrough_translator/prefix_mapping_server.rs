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

//! Answers `example_interfaces/srv/AddTwoInts` requests received as CDR
//! bytes on the service prefix-mapped to the ROS 2 service `/add_two_ints`. The application deserializes the requests
//! and serializes the responses itself; the gateway's passthrough translator
//! propagates the bytes unchanged.
//!
//! ```bash
//! cargo run --manifest-path integrations/ros2/Cargo.toml --example passthrough_prefix_mapping_server
//! # in other shells:
//! #   cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
//! #       --allow /add_two_ints
//! #   ros2 service call /add_two_ints example_interfaces/srv/AddTwoInts "{a: 2, b: 3}"
//! ```

use core::time::Duration;

use cdr::{CdrLe, Infinite};
use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::passthrough_translator::{
    AddTwoIntsRequestByte, AddTwoIntsResponseByte, request_as_bytes,
};
use ros_env::example_interfaces::srv::{AddTwoInts_Request, AddTwoInts_Response};

/// The iceoryx2 service mapped by the name prefix to the ROS 2 service
/// `/add_two_ints`.
const SERVICE_NAME: &str = "ros2://services/add_two_ints";

const CYCLE_TIME: Duration = Duration::from_millis(100);
const INITIAL_MAX_PAYLOAD_SIZE: usize = 64;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .request_response::<[AddTwoIntsRequestByte], [AddTwoIntsResponseByte]>()
        .open_or_create()?;

    let server = service
        .server_builder()
        .initial_max_slice_len(INITIAL_MAX_PAYLOAD_SIZE)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    coutln!("waiting for requests on {SERVICE_NAME}");
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(request) = server.receive()? {
            let message: AddTwoInts_Request =
                cdr::deserialize(request_as_bytes(request.payload()))?;
            let sum = message.a + message.b;
            coutln!(
                "received: {} + {}, responding: {}",
                message.a,
                message.b,
                sum
            );

            let response: Vec<AddTwoIntsResponseByte> =
                cdr::serialize::<_, _, CdrLe>(&AddTwoInts_Response { sum }, Infinite)?
                    .into_iter()
                    .map(AddTwoIntsResponseByte)
                    .collect();
            request
                .loan_slice_uninit(response.len())?
                .write_from_slice(&response)
                .send()?;
        }
    }

    coutln!("exit");

    Ok(())
}
