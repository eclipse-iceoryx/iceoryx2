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

use std::time::Duration;

use iceoryx2::prelude::*;
use iceoryx2_bb_log::{error, info};

fn log(message: &str) {
    info!("<< PINGER >> {}", message);
}

fn pass_test() {
    info!("<< TEST >> SUCCESS");
    std::process::exit(0);
}

fn fail_test(message: &str) -> ! {
    error!("<< TEST >> {}", message);
    std::process::exit(-128);
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    const PING_SERVICE_NAME: &str = "tunnel-end-to-end-test/ping";
    const PONG_SERVICE_NAME: &str = "tunnel-end-to-end-test/pong";
    type PayloadType = u64;
    const PAYLOAD_DATA: PayloadType = 42;

    log("STARTING Pinger");

    let pinger_node = NodeBuilder::new().create::<ipc::Service>()?;
    log("CREATED Pinger Node");

    let ping_service = pinger_node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .publish_subscribe::<PayloadType>()
        .history_size(10)
        .open_or_create()?;
    let ping_publisher = ping_service.publisher_builder().create()?;
    log("CREATED Ping Publisher");

    let ping_service = pinger_node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?;
    let ping_notifier = ping_service.notifier_builder().create()?;
    log("CREATED Ping Notifier");

    let pong_service = pinger_node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .publish_subscribe::<PayloadType>()
        .history_size(10)
        .open_or_create()?;
    let pong_subscriber = pong_service.subscriber_builder().create()?;
    log("CREATED Pong Subscriber");

    let pong_service = pinger_node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?;
    let pong_listener = pong_service.listener_builder().create()?;
    log("CREATED Pong Listener");

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    let pong_guard = waitset.attach_notification(&pong_listener)?;
    let pong_id = WaitSetAttachmentId::from_guard(&pong_guard);

    let timeout_guard = waitset.attach_interval(Duration::from_secs(5))?;
    let timeout_id = WaitSetAttachmentId::from_guard(&timeout_guard);

    let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
        if id == pong_id {
            log("RECEIVED Pong Notification");

            match pong_subscriber.receive() {
                Ok(sample) => match sample {
                    Some(sample) => {
                        log("RECEIVED Pong Payload");
                        if *sample.payload() == PAYLOAD_DATA {
                            pass_test();
                        } else {
                            fail_test(&format!(
                                "FAILED Unexpected sample received at subscriber. Sent: {:?}, Received: {:?}",
                                PAYLOAD_DATA,
                                *sample.payload()
                            ));
                        }
                    }
                    None => {
                        fail_test("FAILED None sample at Pong Subscriber");
                    }
                },
                Err(e) => {
                    fail_test(&format!("FAILED Error receiving from Pong Subscriber: {e}"));
                }
            }
        }
        if id == timeout_id {
            fail_test("FAILED Timed out");
        }

        fail_test("FAILED Unexpected Event");
    };

    log("CREATED Waitset");
    log("STARTED Pinger");

    let ping_sample = ping_publisher.loan_uninit()?;
    let ping_sample = ping_sample.write_payload(PAYLOAD_DATA);
    ping_sample.send()?;
    ping_notifier.notify()?;

    log("SENT Ping");

    waitset.wait_and_process(on_event)?;

    Ok(())
}
