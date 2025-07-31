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

use crate::cli::{CliTypeVariant, DataRepresentation, PublishOptions};
use anyhow::Result;
use core::mem::MaybeUninit;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::prelude::*;
use iceoryx2::sample_mut_uninit::SampleMutUninit;
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::service::static_config::message_type_details::{
    TypeDetail, TypeNameString, TypeVariant,
};
use iceoryx2_cli::Format;
use iceoryx2_userland_record_and_replay::hex_conversion::hex_string_to_bytes;
use std::ptr::copy_nonoverlapping;
use std::time::Duration;

fn loan(
    len: usize,
    publisher: &Publisher<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker>,
    options: &PublishOptions,
) -> Result<SampleMutUninit<ipc::Service, [MaybeUninit<CustomPayloadMarker>], CustomHeaderMarker>> {
    match options.type_variant {
        CliTypeVariant::Dynamic => unsafe {
            publisher
                .loan_custom_payload(len)
                .map_err(|e| anyhow::anyhow!("failed to loan sample ({e:?})"))
        },
        CliTypeVariant::FixedSize => {
            let sample = unsafe { publisher.loan_custom_payload(1) }
                .map_err(|e| anyhow::anyhow!("failed to loan sample ({e:?})"))?;
            if sample.payload().len() != len {
                Err(anyhow::anyhow!(
                    "raw message size of {} does not fit required type size of {}",
                    len,
                    sample.payload().len()
                ))
            } else {
                Ok(sample)
            }
        }
    }
}

fn send_message(
    user_header: &[u8],
    payload: &[u8],
    publisher: &Publisher<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker>,
    options: &PublishOptions,
) -> Result<()> {
    let mut sample = loan(payload.len(), publisher, options)?;
    unsafe {
        copy_nonoverlapping(
            payload.as_ptr(),
            sample.payload_mut().as_mut_ptr().cast(),
            payload.len(),
        )
    }

    if options.header_type_size != 0 {
        if user_header.len() != options.header_type_size {
            return Err(anyhow::anyhow!(
                "raw user header size of {} does not fit required user header type size of {}",
                user_header.len(),
                options.header_type_size
            ));
        }

        unsafe {
            copy_nonoverlapping(
                user_header.as_ptr(),
                (sample.user_header_mut() as *mut CustomHeaderMarker).cast(),
                options.header_type_size,
            );
        }
    }

    let sample = unsafe { sample.assume_init() };
    sample.send()?;
    std::thread::sleep(Duration::from_millis(options.time_between_messages as _));

    Ok(())
}

fn read_cli_msg_into_buffer(
    message_buffer: &mut Vec<(Vec<u8>, Vec<u8>)>,
    options: &PublishOptions,
) -> Result<()> {
    for message in &options.message {
        match options.data_representation {
            DataRepresentation::Iox2Dump => {
                message_buffer.push((vec![], message.as_bytes().to_vec()))
            }
            DataRepresentation::HumanReadable => {
                let payload = hex_string_to_bytes(message)?;
                message_buffer.push((vec![], payload));
            }
        }
    }

    Ok(())
}

pub fn publish(options: PublishOptions, _format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let mut payload_type = TypeDetail::new::<()>(match options.type_variant {
        CliTypeVariant::Dynamic => TypeVariant::Dynamic,
        CliTypeVariant::FixedSize => TypeVariant::FixedSize,
    });
    iceoryx2::testing::type_detail_set_size(&mut payload_type, options.type_size);
    iceoryx2::testing::type_detail_set_alignment(&mut payload_type, options.type_alignment);
    iceoryx2::testing::type_detail_set_name(
        &mut payload_type,
        TypeNameString::from_str_truncated(options.type_name.as_str()),
    );

    let mut header_type = TypeDetail::new::<()>(TypeVariant::FixedSize);
    iceoryx2::testing::type_detail_set_size(&mut header_type, options.header_type_size);
    iceoryx2::testing::type_detail_set_alignment(&mut header_type, options.header_type_alignment);
    iceoryx2::testing::type_detail_set_name(
        &mut header_type,
        TypeNameString::from_str_truncated(options.header_type_name.as_str()),
    );

    let service = unsafe {
        node.service_builder(&ServiceName::new(&options.service)?)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&payload_type)
            .__internal_set_user_header_type_details(&header_type)
            .open_or_create()?
    };

    let publisher = match options.type_variant {
        CliTypeVariant::FixedSize => service.publisher_builder().create()?,
        CliTypeVariant::Dynamic => service
            .publisher_builder()
            .initial_max_slice_len(4096)
            .allocation_strategy(AllocationStrategy::PowerOfTwo)
            .create()?,
    };

    let mut message_buffer = vec![];

    read_cli_msg_into_buffer(&mut message_buffer, &options)?;

    let mut counter = 0;
    loop {
        for (header, payload) in &message_buffer {
            send_message(header.as_slice(), payload.as_slice(), &publisher, &options)?;
        }

        counter += 1;
        if counter == options.repetitions {
            break;
        }
    }

    Ok(())
}
