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

use crate::cli::{DataRepresentation, SubscribeOptions};
use anyhow::anyhow;
use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2::sample::Sample;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::TypeDetail;
use iceoryx2_cli::Format;
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

fn raw_data_to_hex_string(raw_data: &[u8]) -> String {
    let mut ret_val = String::with_capacity(2 * raw_data.len());
    for byte in raw_data {
        ret_val.push_str(&format!("{0:0>2x} ", byte));
    }

    ret_val
}

fn get_service_types(
    options: &SubscribeOptions,
    node: &Node<ipc::Service>,
) -> Result<(TypeDetail, TypeDetail)> {
    let service_name = ServiceName::new(&options.service)?;
    let service_details = match ipc::Service::details(
        &service_name,
        node.config(),
        MessagingPattern::PublishSubscribe,
    )? {
        Some(v) => v,
        None => {
            return Err(anyhow!(
                "unable to access service \"{}\", does it exist?",
                options.service
            ))
        }
    };

    let user_header_type = unsafe {
        &service_details
            .static_details
            .messaging_pattern()
            .publish_subscribe()
            .message_type_details()
            .user_header
    };

    let payload_type = unsafe {
        &service_details
            .static_details
            .messaging_pattern()
            .publish_subscribe()
            .message_type_details()
            .payload
    };

    Ok((user_header_type.clone(), payload_type.clone()))
}

fn extract_payload<'a>(
    sample: &'a Sample<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker>,
    user_header_type: &TypeDetail,
) -> (&'a [u8], &'a [u8]) {
    let user_header = unsafe {
        core::slice::from_raw_parts(
            (sample.user_header() as *const CustomHeaderMarker).cast(),
            user_header_type.size,
        )
    };
    let payload = unsafe {
        core::slice::from_raw_parts(sample.payload().as_ptr().cast(), sample.payload().len())
    };

    (user_header, payload)
}

fn output_iox2_dump_data(
    user_header: &[u8],
    payload: &[u8],
    options: &SubscribeOptions,
    file: &mut Option<File>,
) -> Result<()> {
    if !options.quiet {
        print!("header {{len = {}}}: ", user_header.len());
        std::io::stdout().write(user_header).ok();
        print!("\n");

        print!("payload {{len = {}}}: ", payload.len());
        std::io::stdout().write(payload).ok();
        print!("\n");
    }

    if let Some(ref mut file) = file {
        file.write_all(&(user_header.len() as u64).to_le_bytes())?;
        file.write_all(user_header)?;
        file.write_all(&(payload.len() as u64).to_le_bytes())?;
        file.write_all(payload)?;
    }

    Ok(())
}

fn output_hex_data(
    user_header: &[u8],
    payload: &[u8],
    options: &SubscribeOptions,
    file: &mut Option<File>,
) -> Result<()> {
    let hex_user_header = raw_data_to_hex_string(user_header);
    let hex_payload = raw_data_to_hex_string(payload);

    if !options.quiet {
        println!(
            "header {{len = {}}}: {}",
            user_header.len(),
            hex_user_header
        );

        println!("payload {{len = {}}}: {}", payload.len(), hex_payload);
    }

    if let Some(ref mut file) = file {
        writeln!(file, "{hex_user_header}")?;
        writeln!(file, "{hex_payload}")?;
    }

    Ok(())
}

pub fn subscribe(options: SubscribeOptions, _format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service_name = ServiceName::new(&options.service)?;
    let (user_header_type, payload_type) = get_service_types(&options, &node)?;

    let mut file = match &options.output_file {
        Some(v) => Some(
            std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(v)?,
        ),
        None => None,
    };

    let service = unsafe {
        node.service_builder(&service_name)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&payload_type)
            .__internal_set_user_header_type_details(&user_header_type)
            .open_or_create()?
    };

    let subscriber = service.subscriber_builder().create()?;
    let cycle_time = Duration::from_millis(10);

    let start = Instant::now();
    let mut msg_counter = 0u64;
    'node_loop: while node.wait(cycle_time).is_ok() {
        while let Some(sample) = unsafe { subscriber.receive_custom_payload()? } {
            let (user_header, payload) = extract_payload(&sample, &user_header_type);

            match options.data_representation {
                DataRepresentation::Iox2Dump => {
                    output_iox2_dump_data(user_header, payload, &options, &mut file)?
                }
                DataRepresentation::Hex => {
                    output_hex_data(user_header, payload, &options, &mut file)?
                }
            }

            msg_counter += 1;
            if let Some(max_messages) = options.max_messages {
                if msg_counter >= max_messages {
                    break 'node_loop;
                }
            }

            if let Some(timeout) = options.timeout {
                if start.elapsed().as_millis() >= timeout as _ {
                    break 'node_loop;
                }
            }
        }
    }

    Ok(())
}
