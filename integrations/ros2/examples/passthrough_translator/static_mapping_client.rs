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

//! Sends `example_interfaces/srv/AddTwoInts` requests as CDR bytes on the
//! service paired with the ROS 2 service `/add_two_ints` in
//! `static_mapping.toml` and prints the responses. The application
//! serializes the requests and deserializes the responses itself; the
//! gateway's passthrough translator propagates the bytes unchanged.
//!
//! ```bash
//! cargo run --manifest-path integrations/ros2/Cargo.toml --example passthrough_static_mapping_client
//! # in other shells:
//! #   cargo run --manifest-path integrations/ros2/Cargo.toml --bin iox2-link-gateway-ros2 -- \
//! #       --static-mapping integrations/ros2/examples/passthrough_translator/static_mapping.toml
//! #   ros2 run demo_nodes_cpp add_two_ints_server
//! ```

use core::time::Duration;

use cdr::{CdrLe, Infinite};
use iceoryx2::prelude::*;
use iceoryx2_integrations_ros2_examples::passthrough_translator::{
    AddTwoIntsRequestByte, AddTwoIntsResponseByte, response_as_bytes,
};
use ros_env::example_interfaces::srv::{AddTwoInts_Request, AddTwoInts_Response};

/// The iceoryx2 service paired with the ROS 2 service `/add_two_ints` in
/// `static_mapping.toml`.
const SERVICE_NAME: &str = "AddTwoInts";

const CYCLE_TIME: Duration = Duration::from_secs(1);
const INITIAL_MAX_PAYLOAD_SIZE: usize = 64;

const A: i64 = 2;
const B: i64 = 3;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&SERVICE_NAME.try_into()?)
        .request_response::<[AddTwoIntsRequestByte], [AddTwoIntsResponseByte]>()
        .open_or_create()?;

    let client = service
        .client_builder()
        .initial_max_slice_len(INITIAL_MAX_PAYLOAD_SIZE)
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    let request: Vec<AddTwoIntsRequestByte> =
        cdr::serialize::<_, _, CdrLe>(&AddTwoInts_Request { a: A, b: B }, Infinite)?
            .into_iter()
            .map(AddTwoIntsRequestByte)
            .collect();

    coutln!("sending: {A} + {B}");
    let mut pending_response = client
        .loan_slice_uninit(request.len())?
        .write_from_slice(&request)
        .send()?;

    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(response) = pending_response.receive()? {
            let message: AddTwoInts_Response =
                cdr::deserialize(response_as_bytes(response.payload()))?;
            coutln!("received: {}", message.sum);
        }

        coutln!("sending: {A} + {B}");
        pending_response = client
            .loan_slice_uninit(request.len())?
            .write_from_slice(&request)
            .send()?;
    }

    coutln!("exit");

    Ok(())
}
