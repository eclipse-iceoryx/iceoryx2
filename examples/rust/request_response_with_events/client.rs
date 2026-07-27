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

use core::time::Duration;

extern crate alloc;
use alloc::boxed::Box;

use examples_common::TransmissionData;
use iceoryx2::prelude::*;

const CYCLE_TIME: Duration = Duration::from_secs(1);
const TIMEOUT: Duration = Duration::from_millis(100);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let server_event_service = node
        .service_builder(&"example/request_response_with_events/server".try_into()?)
        .event()
        .open_or_create()?;
    let server_notifier = server_event_service.notifier_builder().create()?;

    let client_event_service = node
        .service_builder(&"example/request_response_with_events/client".try_into()?)
        .event()
        .open_or_create()?;
    let client_listener = client_event_service.listener_builder().create()?;
    let listener_id = client_listener.id();

    let service = node
        .service_builder(&"example/request_response_with_events".try_into()?)
        .request_response::<u64, TransmissionData>()
        .request_user_header::<u128>()
        .open_or_create()?;

    let client = service.client_builder().create()?;

    let mut request_counter: u64 = 0;
    let mut response_counter: u64 = 0;

    while node.wait(CYCLE_TIME).is_ok() {
        coutln!("send request {request_counter} ...");
        let mut request = client.loan()?;
        *request.payload_mut() = request_counter;
        *request.user_header_mut() = listener_id.value();
        let pending_response = request.send()?;
        server_notifier.notify()?;

        let events_received = client_listener.timed_wait(|_| {}, TIMEOUT)?;
        if events_received == 0 {
            coutln!("Timeout while waiting for response from server");
            continue;
        }
        coutln!("  number of received notifications from server: {events_received}");

        // acquire all responses to our request from our buffer that were sent by the servers
        while let Some(response) = pending_response.receive()? {
            coutln!("  received response {response_counter}: {:?}", *response);
            response_counter += 1;
        }

        request_counter += 1;
    }

    coutln!("exit");

    Ok(())
}
