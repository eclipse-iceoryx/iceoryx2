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

use iceoryx2::prelude::*;
use iceoryx2_bb_log::info;

fn log(message: &str) {
    info!("<< PONGER >> {}", message);
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    const PING_SERVICE_NAME: &str = "tunnel-end-to-end-test/ping";
    const PONG_SERVICE_NAME: &str = "tunnel-end-to-end-test/pong";
    type PayloadType = u64;

    log("STARTING Ponger");

    let ponger_node = NodeBuilder::new().create::<ipc::Service>()?;
    log("CREATED Ponger Node");

    let ping_service = ponger_node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .publish_subscribe::<PayloadType>()
        .history_size(10)
        .open_or_create()?;
    let ping_subscriber = ping_service.subscriber_builder().create()?;
    log("CREATED Ping Subscriber");

    let ping_service = ponger_node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?;
    let ping_listener = ping_service.listener_builder().create()?;
    log("CREATED Ping Listener");

    let pong_service = ponger_node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .publish_subscribe::<PayloadType>()
        .history_size(10)
        .open_or_create()?;
    let pong_publisher = pong_service.publisher_builder().create()?;
    log("CREATED Pong Publisher");

    let pong_service = ponger_node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?;
    let pong_notifier = pong_service.notifier_builder().create()?;
    log("CREATED Pong Notifier");

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let ping_guard = waitset.attach_notification(&ping_listener)?;
    let ping_id = WaitSetAttachmentId::from_guard(&ping_guard);

    let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
        if id == ping_id {
            log("RECEIVED Ping Notification");
            ping_listener.try_wait_all(|_| {}).unwrap();

            while let Ok(Some(sample)) = ping_subscriber.receive() {
                log("RECEIVED Ping Payload");

                let pong_sample = pong_publisher.loan_uninit().unwrap();
                let pong_sample = pong_sample.write_payload(*sample.payload());
                pong_sample.send().unwrap();
                pong_notifier.notify().unwrap();

                log("SENT Pong");
            }
        }
        CallbackProgression::Continue
    };

    log("CREATED Waitset");
    log("STARTED Ponger");

    waitset.wait_and_process(on_event)?;

    Ok(())
}
