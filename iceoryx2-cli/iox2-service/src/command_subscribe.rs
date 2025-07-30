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
use iceoryx2::service::header::publish_subscribe::Header;
use iceoryx2::service::static_config::message_type_details::TypeDetail;
use iceoryx2::service::static_config::message_type_details::TypeVariant;
use iceoryx2_cli::Format;
use iceoryx2_userland_record_and_replay::record::RawRecord;
use iceoryx2_userland_record_and_replay::recorder::RecorderBuilder;
use iceoryx2_userland_record_and_replay::recorder::ServiceTypes;
use std::time::Duration;
use std::time::Instant;

#[derive(serde::Serialize)]
struct Message {
    system_header_len: usize,
    system_header: String,
    user_header_len: usize,
    user_header: String,
    payload_len: usize,
    payload: String,
}

fn raw_data_to_hex_string(raw_data: &[u8]) -> String {
    use std::fmt::Write;

    let mut ret_val = String::with_capacity(3 * raw_data.len());
    for byte in raw_data {
        let _ = write!(&mut ret_val, "{byte:0>2x} ");
    }

    ret_val
}

fn get_service_types(
    options: &SubscribeOptions,
    node: &Node<ipc::Service>,
) -> Result<ServiceTypes> {
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

    let user_header = unsafe {
        service_details
            .static_details
            .messaging_pattern()
            .publish_subscribe()
            .message_type_details()
            .user_header
            .clone()
    };

    let payload = unsafe {
        service_details
            .static_details
            .messaging_pattern()
            .publish_subscribe()
            .message_type_details()
            .payload
            .clone()
    };

    let system_header = TypeDetail::new::<Header>(TypeVariant::FixedSize);

    Ok(ServiceTypes {
        payload,
        user_header,
        system_header,
    })
}

fn extract_payload<'a>(
    sample: &'a Sample<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker>,
    user_header_type: &TypeDetail,
) -> (&'a [u8], &'a [u8], &'a [u8]) {
    let system_header = unsafe {
        core::slice::from_raw_parts(
            (sample.header() as *const Header).cast(),
            core::mem::size_of::<Header>(),
        )
    };
    let user_header = unsafe {
        core::slice::from_raw_parts(
            (sample.user_header() as *const CustomHeaderMarker).cast(),
            user_header_type.size(),
        )
    };
    let payload = unsafe {
        core::slice::from_raw_parts(sample.payload().as_ptr().cast(), sample.payload().len())
    };

    (system_header, user_header, payload)
}

fn print_hex_dump(
    system_header: &[u8],
    user_header: &[u8],
    payload: &[u8],
    options: &SubscribeOptions,
    format: Format,
) -> Result<()> {
    if options.quiet {
        return Ok(());
    }

    let msg = Message {
        system_header_len: system_header.len(),
        system_header: raw_data_to_hex_string(system_header),
        user_header_len: user_header.len(),
        user_header: raw_data_to_hex_string(user_header),
        payload_len: payload.len(),
        payload: raw_data_to_hex_string(payload),
    };

    println!(
        "{}",
        format
            .as_string(&msg)
            .unwrap_or("Failed to format message".to_string())
    );

    Ok(())
}

fn print_iox2_dump(
    system_header: &[u8],
    user_header: &[u8],
    payload: &[u8],
    options: &SubscribeOptions,
    format: Format,
) -> Result<()> {
    if options.quiet {
        return Ok(());
    }

    let msg = Message {
        system_header_len: system_header.len(),
        system_header: raw_data_to_hex_string(system_header),
        user_header_len: user_header.len(),
        user_header: raw_data_to_hex_string(user_header),
        payload_len: payload.len(),
        payload: String::from_utf8_lossy(payload).to_string(),
    };

    println!(
        "{}",
        format
            .as_string(&msg)
            .unwrap_or("Failed to format message".to_string())
    );

    Ok(())
}

pub fn subscribe(options: SubscribeOptions, format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service_name = ServiceName::new(&options.service)?;
    let service_types = get_service_types(&options, &node)?;

    let mut file = match &options.output_file {
        Some(v) => Some(
            RecorderBuilder::new(&service_types)
                .data_representation(options.data_representation.into())
                .messaging_pattern(MessagingPattern::PublishSubscribe)
                .create(&FilePath::new(v.as_bytes())?)?,
        ),
        None => None,
    };

    let service = unsafe {
        node.service_builder(&service_name)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&service_types.payload)
            .__internal_set_user_header_type_details(&service_types.user_header)
            .open_or_create()?
    };

    let subscriber = service.subscriber_builder().create()?;
    let cycle_time = Duration::from_millis(1);

    let start = Instant::now();
    let mut msg_counter = 0u64;
    'node_loop: while node.wait(cycle_time).is_ok() {
        while let Some(sample) = unsafe { subscriber.receive_custom_payload()? } {
            let (system_header, user_header, payload) =
                extract_payload(&sample, &service_types.user_header);

            let mut record_to_file = |system_header, user_header, payload| -> Result<()> {
                if let Some(file) = &mut file {
                    file.write(RawRecord {
                        timestamp: start.elapsed(),
                        system_header,
                        user_header,
                        payload,
                    })?;
                }

                Ok(())
            };

            match options.data_representation {
                DataRepresentation::Iox2Dump => {
                    print_iox2_dump(system_header, user_header, payload, &options, format)?;
                    record_to_file(system_header, user_header, payload)?;
                }
                DataRepresentation::HumanReadable => {
                    print_hex_dump(system_header, user_header, payload, &options, format)?;
                    record_to_file(system_header, user_header, payload)?;
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
