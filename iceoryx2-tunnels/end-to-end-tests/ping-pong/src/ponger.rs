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

use clap::Parser;
use iceoryx2::prelude::{
    ipc, set_log_level_from_env_or, CallbackProgression, LogLevel, NodeBuilder,
    WaitSetAttachmentId, WaitSetBuilder,
};
use iceoryx2_bb_log::info;
use iceoryx2_tunnels_end_to_end_tests::cli::*;
use iceoryx2_tunnels_end_to_end_tests::config::*;
use iceoryx2_tunnels_end_to_end_tests::payload::*;

fn run_ponger<P: PayloadWriter>() -> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let ping_subscriber = node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .publish_subscribe::<P::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .subscriber_builder()
        .create()?;

    let ping_listener = node
        .service_builder(&PING_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?
        .listener_builder()
        .create()?;

    let pong_publisher = node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .publish_subscribe::<P::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .publisher_builder()
        .create()?;

    let pong_notifier = node
        .service_builder(&PONG_SERVICE_NAME.try_into()?)
        .event()
        .open_or_create()?
        .notifier_builder()
        .create()?;

    let waitset = WaitSetBuilder::new().create::<ipc::Service>()?;
    let ping_guard = waitset.attach_notification(&ping_listener)?;
    let ping_id = WaitSetAttachmentId::from_guard(&ping_guard);

    let on_event = |id: WaitSetAttachmentId<ipc::Service>| {
        if id == ping_id {
            ping_listener.try_wait_all(|_| {}).unwrap();

            while let Ok(Some(ping_sample)) = ping_subscriber.receive() {
                let pong_sample = pong_publisher.loan_uninit().unwrap();

                // Copy the received ping payload directly into the pong payload, by-passing stack
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        ping_sample.payload() as *const P::PayloadType as *const u8,
                        pong_sample.payload().as_ptr() as *mut u8,
                        std::mem::size_of::<P::PayloadType>(),
                    );
                }

                let pong_sample = unsafe { pong_sample.assume_init() };
                pong_sample.send().unwrap();
                pong_notifier.notify().unwrap();
            }
        }
        CallbackProgression::Continue
    };

    waitset.wait_and_process(on_event)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Warn);

    let args = Args::parse();

    info!("Running with payload type: {:?}", args.payload_type);

    match args.payload_type {
        PayloadType::Primitive => run_ponger::<PrimitivePayload>(),
        PayloadType::Complex => run_ponger::<ComplexPayload>(),
    }
}
