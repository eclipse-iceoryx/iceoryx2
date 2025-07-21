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

const CYCLE_TIME: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .request_response::<u64, TransmissionData>()
        .open_or_create()?;

    let server = service.server_builder().create()?;

    println!("Server ready to receive requests!");

    let mut counter = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        while let Some(active_request) = server.receive()? {
            println!("received request: {:?}", *active_request);

            let response = TransmissionData {
                x: 5 + counter,
                y: 6 * counter,
                funky: 7.77,
            };
            println!("  send response: {response:?}");
            // send first response by using the slower, non-zero-copy API
            active_request.send_copy(response)?;

            // use zero copy API, send out some responses to demonstrate the streaming API
            for n in 0..*active_request % 2 {
                let response = active_request.loan_uninit()?;
                let response = response.write_payload(TransmissionData {
                    x: counter * (n as i32 + 1),
                    y: counter + n as i32,
                    funky: counter as f64 * 0.1234,
                });
                println!("  send response: {:?}", *response);
                response.send()?;
            }

            // when an active_request is dropped it marks the connection so
            // that the corresponding pending response sees that no more
            // responses are arriving
            drop(active_request);
        }

        counter += 1;
    }

    println!("exit");

    Ok(())
}
