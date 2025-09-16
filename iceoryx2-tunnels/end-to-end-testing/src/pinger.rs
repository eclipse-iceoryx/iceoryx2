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

mod cli;
mod config;
mod testing;

use std::rc::Rc;

use crate::cli::*;
use crate::config::*;
use crate::testing::*;
use clap::Parser;
use iceoryx2::prelude::{ipc, NodeBuilder, WaitSetAttachmentId, WaitSetBuilder};

fn run_pinger<P: PayloadWriter>() -> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let ping_publisher = node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .publish_subscribe::<P::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .publisher_builder()
        .create()?;

    let ping_notifier = node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?
        .notifier_builder()
        .create()?;

    let pong_subscriber = node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .publish_subscribe::<P::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .subscriber_builder()
        .create()?;

    let pong_listener = node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?
        .listener_builder()
        .create()?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;

    let pong_guard = waitset.attach_notification(&pong_listener)?;
    let pong_id = WaitSetAttachmentId::from_guard(&pong_guard);

    let timeout_guard = waitset.attach_interval(TIMEOUT_DURATION)?;
    let timeout_id = WaitSetAttachmentId::from_guard(&timeout_guard);

    // Create the payload on the heap
    let mut payload_bytes = vec![0u8; std::mem::size_of::<P::PayloadType>()];
    let payload_ptr = payload_bytes.as_mut_ptr() as *mut P::PayloadType;
    unsafe {
        P::write_payload(&mut *payload_ptr);
    }

    // Relinquish ownership of the bytes
    std::mem::forget(payload_bytes);

    // Claim ownership with a Box and to use with Rc API
    let payload_box = unsafe { Box::from_raw(payload_ptr) };

    // Wrap in Rc since on_event required to be FnMut as closure technically can run N times
    let payload = Rc::from(payload_box);

    let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
        if id == pong_id {
            match pong_subscriber.receive() {
                Ok(sample) => match sample {
                    Some(pong_sample) => {
                        if *pong_sample.payload() == *payload {
                            pass_test();
                        } else {
                            fail_test(&format!(
                                "Unexpected sample received at subscriber. Sent: {:?}, Received: {:?}",
                                *payload,
                                *pong_sample.payload()
                            ));
                        }
                    }
                    None => {
                        fail_test("None sample at Pong Subscriber");
                    }
                },
                Err(e) => {
                    fail_test(&format!("Error receiving from Pong Subscriber: {e}"));
                }
            }
        }
        if id == timeout_id {
            fail_test("Timed out");
        }

        fail_test("Unexpected Event");
    };

    let ping_sample = ping_publisher.loan_uninit()?;

    // The bytes of the payload are copied directly into shared memory, by-passing stack
    unsafe {
        std::ptr::copy_nonoverlapping(
            payload.as_ref() as *const P::PayloadType as *const u8,
            ping_sample.payload().as_ptr() as *mut u8,
            std::mem::size_of::<P::PayloadType>(),
        );
    }

    let ping_sample = unsafe { ping_sample.assume_init() };
    ping_sample.send()?;
    ping_notifier.notify()?;

    waitset.wait_and_process(on_event)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let args = Args::parse();

    println!("Running with payload type: {:?}", args.payload_type);

    match args.payload_type {
        PayloadType::Primitive => run_pinger::<PrimitivePayload>(),
        PayloadType::Complex => run_pinger::<ComplexPayload>(),
    }
}
