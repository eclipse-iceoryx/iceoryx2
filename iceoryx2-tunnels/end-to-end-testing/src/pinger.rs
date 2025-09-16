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

mod config;
mod testing;

use crate::config::*;
use crate::testing::*;
use iceoryx2::prelude::{ipc, NodeBuilder, WaitSetAttachmentId, WaitSetBuilder};

fn run_pinger<C: Config>(config: &C) -> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let ping_publisher = node
        .service_builder(&config.ping_service_name().try_into()?)
        .publish_subscribe::<C::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .publisher_builder()
        .create()?;

    let ping_notifier = node
        .service_builder(&config.ping_service_name().try_into()?)
        .event()
        .open_or_create()?
        .notifier_builder()
        .create()?;

    let pong_subscriber = node
        .service_builder(&config.pong_service_name().try_into()?)
        .publish_subscribe::<C::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .subscriber_builder()
        .create()?;

    let pong_listener = node
        .service_builder(&config.pong_service_name().try_into()?)
        .event()
        .open_or_create()?
        .listener_builder()
        .create()?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    let pong_guard = waitset.attach_notification(&pong_listener)?;
    let pong_id = WaitSetAttachmentId::from_guard(&pong_guard);

    let timeout_guard = waitset.attach_interval(TIMEOUT_DURATION)?;
    let timeout_id = WaitSetAttachmentId::from_guard(&timeout_guard);

    let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
        if id == pong_id {
            match pong_subscriber.receive() {
                Ok(sample) => match sample {
                    Some(sample) => {
                        if *sample.payload() == config.payload() {
                            pass_test();
                        } else {
                            fail_test(&format!(
                                "FAILED Unexpected sample received at subscriber. Sent: {:?}, Received: {:?}",
                                config.payload(),
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

    let ping_sample = ping_publisher.loan_uninit()?;
    let ping_sample = ping_sample.write_payload(config.payload());
    ping_sample.send()?;
    ping_notifier.notify()?;

    waitset.wait_and_process(on_event)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    run_pinger(&PrimitiveType)
}
