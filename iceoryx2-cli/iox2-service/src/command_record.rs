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

use crate::helper_functions::extract_pubsub_payload;
use crate::{cli::RecordOptions, helper_functions::get_pubsub_service_types};
use anyhow::Result;
use core::time::Duration;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2_cli::Format;
use iceoryx2_userland_record_and_replay::prelude::*;
use std::io::Write;
use std::time::Instant;

pub fn record(options: RecordOptions, _format: Format) -> Result<()> {
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
    let mut recorder = RecorderBuilder::new(&service_types)
        .data_representation(options.data_representation.into())
        .messaging_pattern(options.messaging_pattern.into())
        .create(&FilePath::new(options.output.as_bytes())?, &service_name)?;

    println!("Start recording data on \"{}\".", options.service);

    let start = Instant::now();
    let mut msg_counter = 0u64;
    let cycle_time = Duration::from_millis(options.cycle_time_in_ms);
    'node_loop: loop {
        while let Some(sample) = unsafe { subscriber.receive_custom_payload()? } {
            let (system_header, user_header, payload) =
                extract_pubsub_payload(&sample, &service_types.user_header);

            let elapsed = start.elapsed();
            recorder.write(RawRecord {
                timestamp: elapsed,
                system_header,
                user_header,
                payload,
            })?;

            print!(".");
            std::io::stdout().flush()?;
            msg_counter += 1;
            if let Some(max_messages) = options.max_messages {
                if msg_counter >= max_messages {
                    break 'node_loop;
                }
            }

            if let Some(timeout) = options.timeout_in_sec {
                if start.elapsed().as_secs() >= timeout as _ {
                    break 'node_loop;
                }
            }
        }

        if node.wait(cycle_time).is_err() {
            break 'node_loop;
        }
    }
    println!(" ");

    Ok(())
}
