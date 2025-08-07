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
use crate::helper_functions::{extract_pubsub_payload, get_pubsub_service_types};
use anyhow::Result;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2_cli::Format;
use iceoryx2_userland_record_and_replay::hex_conversion::bytes_to_hex_string;
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

fn print_hex_dump(
    system_header: &[u8],
    user_header: &[u8],
    payload: &[u8],
    format: Format,
) -> Result<()> {
    let msg = Message {
        system_header_len: system_header.len(),
        system_header: bytes_to_hex_string(system_header),
        user_header_len: user_header.len(),
        user_header: bytes_to_hex_string(user_header),
        payload_len: payload.len(),
        payload: bytes_to_hex_string(payload),
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
    format: Format,
) -> Result<()> {
    let msg = Message {
        system_header_len: system_header.len(),
        system_header: bytes_to_hex_string(system_header),
        user_header_len: user_header.len(),
        user_header: bytes_to_hex_string(user_header),
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
    let service_types = get_pubsub_service_types(&service_name, &node)?;

    let service = unsafe {
        node.service_builder(&service_name)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&service_types.payload)
            .__internal_set_user_header_type_details(&service_types.user_header)
            .open_or_create()?
    };

    let subscriber = service.subscriber_builder().create()?;
    let cycle_time = Duration::from_millis(100);

    let start = Instant::now();
    let mut msg_counter = 0u64;
    'node_loop: while node.wait(cycle_time).is_ok() {
        while let Some(sample) = unsafe { subscriber.receive_custom_payload()? } {
            let (system_header, user_header, payload) =
                extract_pubsub_payload(&sample, &service_types.user_header);

            match options.data_representation {
                DataRepresentation::Iox2Dump => {
                    print_iox2_dump(system_header, user_header, payload, format)?;
                }
                DataRepresentation::HumanReadable => {
                    print_hex_dump(system_header, user_header, payload, format)?;
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
