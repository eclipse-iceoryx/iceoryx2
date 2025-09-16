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

use crate::cli::*;
use crate::config::*;
use clap::Parser;
use iceoryx2::prelude::{
    ipc, CallbackProgression, NodeBuilder, WaitSetAttachmentId, WaitSetBuilder,
};

fn run_ponger<C: Config>(config: &C) -> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let ping_subscriber = node
        .service_builder(&config.ping_service_name().try_into()?)
        .publish_subscribe::<C::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .subscriber_builder()
        .create()?;

    let ping_listener = node
        .service_builder(&config.ping_service_name().try_into()?)
        .event()
        .open_or_create()?
        .listener_builder()
        .create()?;

    let pong_publisher = node
        .service_builder(&config.pong_service_name().try_into()?)
        .publish_subscribe::<C::PayloadType>()
        .history_size(HISTORY_SIZE)
        .open_or_create()?
        .publisher_builder()
        .create()?;

    let pong_notifier = node
        .service_builder(&config.pong_service_name().try_into()?)
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

                // copy the received ping payload to the pong payload
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        ping_sample.payload() as *const C::PayloadType as *const u8,
                        pong_sample.payload().as_ptr() as *mut u8,
                        std::mem::size_of::<C::PayloadType>(),
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
    let args = Args::parse();

    match args.payload_type {
        PayloadType::Primitive => run_ponger(&PrimitiveType),
        PayloadType::Complex => todo!(),
    }
}
