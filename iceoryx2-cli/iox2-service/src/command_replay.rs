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

use core::ptr::copy_nonoverlapping;
use core::time::Duration;
use std::io::Write;
use std::time::Instant;

use crate::{cli::ReplayOptions, helper_functions::get_pubsub_service_types};
use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_cli::Format;
use iceoryx2_userland_record_and_replay::prelude::*;
use iceoryx2_userland_record_and_replay::record_header::{
    RecordHeaderDetails, FILE_FORMAT_HUMAN_READABLE_VERSION, FILE_FORMAT_IOX2_DUMP_VERSION,
};

pub fn replay(options: ReplayOptions, _format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let replay = ReplayerOpener::new(&FilePath::new(options.input.as_bytes())?)
        .data_representation(options.data_representation.into())
        .open()?;

    let service_name = match options.service {
        Some(v) => ServiceName::new(&v)?,
        None => replay.header().service_name.clone(),
    };

    let required_header = RecordHeaderDetails {
        file_format_version: match options.data_representation {
            crate::cli::DataRepresentation::HumanReadable => FILE_FORMAT_HUMAN_READABLE_VERSION,
            crate::cli::DataRepresentation::Iox2Dump => FILE_FORMAT_IOX2_DUMP_VERSION,
        },
        types: get_pubsub_service_types(&service_name, &node)?,
        messaging_pattern: options.messaging_pattern.into(),
    };

    if required_header != replay.header().details {
        return Err(anyhow::anyhow!(
            "The expected header {required_header:?} does not match the actual header {:?}.",
            replay.header().details
        ));
    }

    let buffer = replay.read_into_buffer()?;

    let service = unsafe {
        node.service_builder(&service_name)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&required_header.types.payload)
            .__internal_set_user_header_type_details(&required_header.types.user_header)
            .open_or_create()?
    };

    let publisher = match required_header.types.payload.variant() {
        TypeVariant::FixedSize => service.publisher_builder().create()?,
        TypeVariant::Dynamic => service
            .publisher_builder()
            .initial_max_slice_len(4096)
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()?,
    };

    println!("Start replaying data on \"{service_name}\".");
    for n in 0..u64::MAX {
        let start = Instant::now();
        for data in &buffer {
            let sample = unsafe {
                let mut sample = publisher.loan_custom_payload(1)?;
                copy_nonoverlapping(
                    data.payload.as_ptr(),
                    sample.payload_mut().as_ptr() as *mut u8,
                    data.payload.len(),
                );
                if !data.user_header.is_empty() {
                    copy_nonoverlapping(
                        data.user_header.as_ptr(),
                        (sample.user_header_mut() as *mut CustomHeaderMarker) as *mut u8,
                        data.user_header.len(),
                    );
                }
                sample.assume_init()
            };

            let elapsed = start.elapsed().as_millis() as f64 * options.time_factor as f64;
            let timestamp = data.timestamp.as_millis() as f64 * options.time_factor as f64;
            if elapsed < timestamp {
                std::thread::sleep(Duration::from_millis((timestamp - elapsed) as u64));
            }

            sample.send()?;
            print!(".");
            std::io::stdout().flush()?;
        }

        if options.repetitions <= n {
            break;
        }
    }

    Ok(())
}
