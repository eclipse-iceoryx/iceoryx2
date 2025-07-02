// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use core::time::Duration;
use examples_common::TransmissionData;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .request_response::<u64, TransmissionData>()
        .open_or_create()?;

    let client = service.client_builder().create()?;

    let mut request_counter: u64 = 0;
    let mut response_counter: u64 = 0;

    // sending first request by using slower, inefficient copy API
    println!("send request {request_counter} ...");
    let mut pending_response = client.send_copy(request_counter)?;

    while node.wait(CYCLE_TIME).is_ok() {
        // acquire all responses to our request from our buffer that were sent by the servers
        while let Some(response) = pending_response.receive()? {
            println!("  received response {response_counter}: {:?}", *response);
            response_counter += 1;
        }

        request_counter += 1;
        // send all other requests by using zero copy API
        let request = client.loan_uninit()?;
        let request = request.write_payload(request_counter);

        pending_response = request.send()?;

        println!("send request {request_counter} ...");
    }

    println!("exit");

    Ok(())
}
