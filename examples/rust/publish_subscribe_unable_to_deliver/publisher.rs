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
        .set_unable_to_deliver_handler({
            let counter = counter.clone();
            let delivery_incident_counter = alloc::sync::Arc::new(AtomicU64::new(0));
            move |info| {
                // only print the port IDs in the first iteration of the retry loop of each delivery incident
                if info.retries == 0 {
                    delivery_incident_counter.fetch_add(1, Ordering::SeqCst);
                    println!(
                        "Sample delivery interruption count {}",
                        delivery_incident_counter.load(Ordering::SeqCst)
                    );

                    println!(
                        "    Could not deliver sample {} from publisher sender id 0x{:02X} to subscriber receiver id 0x{:02X}",
                        counter.load(Ordering::SeqCst),
                        info.sender_port_id,
                        info.receiver_port_id
                    );
                }

                // there are multiple mitigation options available and to showcase these options,
                // the mitigation is selected based on the incident counter
                match delivery_incident_counter.load(Ordering::SeqCst) % 4 {
                    0 => {
                        // use the built-in sleeping strategy and keep retrying to send the sample
                        // for 10ms and then discard the sample for the receiver that caused the
                        // incident but continue to try delivering the sample to all other receiver
                        // to whom no attempt was taken to deliver the sample, yet;
                        // return with an error if the sample was not delivered to all receivers
                        if info.elapsed_time < Duration::from_millis(10) {
                            UnableToDeliverAction::Retry
                        } else {
                            println!("    Retried for 10ms! Discarding sample and failing");
                            UnableToDeliverAction::DiscardDataAndFail
                        }
                    }
                    1 => {
                        // instead of using the built-in sleeping strategy, the sleep time is defined
                        // by the handler and the sample is discarded after a specified amount of retries
                        // for the receiver that caused the incident but continue to try delivering
                        // the sample to all other receiver to whom no attempt was taken to deliver
                        // the sample, yet;
                        // return with an error if the sample was not delivered to all receivers
                        if info.retries < 10 {
                            println!("    Sleeping 100ms and retry");
                            std::thread::sleep(core::time::Duration::from_millis(100));
                            UnableToDeliverAction::Retry
                        } else {
                            UnableToDeliverAction::DiscardDataAndFail
                        }
                    }
                    2 => {
                        // just discard the sample for the receiver involved in the incident and
                        // continue to try delivering the sample to all other receiver to whom no
                        // attempt was taken to deliver the sample, yet
                        println!("    Discarding sample silently");
                        UnableToDeliverAction::DiscardData
                    }
                    _ => {
                        // just discard the sample for the receiver involved in the incident and
                        // continue to try delivering the sample to all other receiver to whom
                        // no attempt was taken to deliver the sample, yet;
                        // return with an error if the sample was not delivered to all receivers
                        println!("    Discarding sample and failing");
                        UnableToDeliverAction::DiscardDataAndFail
                    }
                }
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
