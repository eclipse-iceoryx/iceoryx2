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

const TIMEOUT: Duration = Duration::from_secs(2);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let server_event_service = node
        .service_builder(&"example/request_response_with_events/server".try_into()?)
        .event()
        .open_or_create()?;
    let server_listener = server_event_service.listener_builder().create()?;

    let client_event_service = node
        .service_builder(&"example/request_response_with_events/client".try_into()?)
        .event()
        .open_or_create()?;
    let client_notifier = client_event_service.notifier_builder().create()?;

    let service = node
        .service_builder(&"example/request_response_with_events".try_into()?)
        .request_response::<u64, TransmissionData>()
        .request_user_header::<u128>()
        .open_or_create()?;

    let server = service.server_builder().create()?;

    coutln!("Server ready to receive requests!");

    let mut counter = 0;
    while node.wait(Duration::ZERO).is_ok() {
        let events_received = server_listener.timed_wait(|_| {}, TIMEOUT)?;
        if events_received == 0 {
            coutln!("Timeout while waiting for clients");
            continue;
        }

        while let Some(active_request) = server.receive()? {
            coutln!("received request: {:?}", *active_request);

            let response = TransmissionData {
                x: 5 + counter,
                y: 6 * counter,
                funky: 7.77,
            };
            coutln!("  send response: {response:?}");
            // use copy API for example simplicity
            active_request.send_copy(response)?;
            let &listerner_id = active_request.user_header();
            client_notifier.for_each_listener(|monofier, listener_details| {
                if listener_details.listener_id.value() == listerner_id {
                    monofier.notify().expect("Listener notified");
                    CallbackProgression::Stop
                } else {
                    CallbackProgression::Continue
                }
            });
        }

        counter += 1;
    }

    coutln!("exit");

    Ok(())
}
