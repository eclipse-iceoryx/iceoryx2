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

use iceoryx2_bb_concurrency::atomic::{AtomicU64, Ordering};

use examples_common::TransmissionData;
use iceoryx2::{port::UnableToDeliverAction, prelude::*};

const CYCLE_TIME: Duration = Duration::from_millis(500);

fn main() -> Result<(), Box<dyn core::error::Error>> {
    set_log_level_from_env_or(LogLevel::Info);

    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&"My/Funk/ServiceName".try_into()?)
        .publish_subscribe::<TransmissionData>()
        .enable_safe_overflow(false)
        .open_or_create()?;
    service.service_hash();

    let counter = alloc::sync::Arc::new(AtomicU64::new(0));

    let publisher = service
        .publisher_builder()
        .unable_to_deliver_strategy(UnableToDeliverStrategy::DeferToHandler)
        .set_unable_to_deliver_handler({
            let counter = counter.clone();
            move |info| {
                println!(
                    "Could not deliver sample {}  from publisher sender id {:?} to subscriber receiver id {:?}",
                    counter.load(Ordering::SeqCst),
                    info.sender_port_id,
                    info.receiver_port_id
                );
                println!("    Discarding sample and failing");
                UnableToDeliverAction::AbortDeliveryAndFail
            }
        })
        .create()?;

    while node.wait(CYCLE_TIME).is_ok() {
        counter.fetch_add(1, Ordering::SeqCst);
        let sample = publisher.loan_uninit()?;

        let counter_val = counter.load(Ordering::SeqCst) as i32;
        let sample = sample.write_payload(TransmissionData {
            x: counter_val,
            y: counter_val * 3,
            funky: counter_val as f64 * 812.12,
        });

        coutln!("Sending sample {counter_val} ...");
        if let Err(e) = sample.send() {
            coutln!("Error: {:?}", e);
        }
    }

    coutln!("exit");

    Ok(())
}
