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
use std::fs::{read_to_string, File};
use std::io::Read;
use std::ptr::copy_nonoverlapping;
use std::time::Duration;

fn hex_string_to_raw_data(hex_string: &str) -> Vec<u8> {
    let mut hex_string = hex_string.to_string();
    hex_string.retain(|c| !c.is_whitespace());
    (0..hex_string.len())
        .step_by(2)
        .map(|n| u8::from_str_radix(&hex_string[n..n + 2], 16).unwrap())
        .collect::<Vec<u8>>()
}

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

fn read_file_into_buffer(
    message_buffer: &mut Vec<(Vec<u8>, Vec<u8>)>,
    options: &PublishOptions,
) -> Result<()> {
    if let Some(ref file) = options.input_file {
        match options.data_representation {
            DataRepresentation::Hex => {
                let mut header = None;
                for line in read_to_string(file)?.lines() {
                    if header.is_none() {
                        header = Some(hex_string_to_raw_data(line));
                    } else {
                        message_buffer.push((
                            header.as_ref().unwrap().clone(),
                            hex_string_to_raw_data(line),
                        ));
                        header = None;
                    }
                }
            }
            DataRepresentation::Iox2Dump => {
                let mut file = File::open(file)?;

                let mut buffer = [0u8; 8];
                let mut read_buffer = || -> Result<()> {
                    file.read_exact(&mut buffer)?;
                    let header_len = u64::from_le_bytes(buffer);
                    let mut header = vec![0u8; header_len as usize];
                    file.read_exact(&mut header)?;

                    file.read_exact(&mut buffer)?;
                    let payload_len = u64::from_le_bytes(buffer);
                    let mut payload = vec![0u8; payload_len as usize];
                    file.read_exact(&mut payload)?;

                    message_buffer.push((header, payload));
                    Ok(())
                };

                while read_buffer().is_ok() {}
            }
        }
    }

    Ok(())
}

fn read_cli_msg_into_buffer(
    message_buffer: &mut Vec<(Vec<u8>, Vec<u8>)>,
    options: &PublishOptions,
) {
    for message in &options.message {
        match options.data_representation {
            DataRepresentation::Iox2Dump => {
                message_buffer.push((vec![], message.as_bytes().to_vec()))
            }
            DataRepresentation::Hex => {
                let payload = hex_string_to_raw_data(message);
                message_buffer.push((vec![], payload));
            }
        }
    }
}

pub fn publish(options: PublishOptions, _format: Format) -> Result<()> {
    let node = NodeBuilder::new()
        .name(&NodeName::new(&options.node_name)?)
        .create::<ipc::Service>()?;

    let service = unsafe {
        node.service_builder(&ServiceName::new(&options.service)?)
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_payload_type_details(&TypeDetail {
                variant: match options.type_variant {
                    CliTypeVariant::Dynamic => TypeVariant::Dynamic,
                    CliTypeVariant::FixedSize => TypeVariant::FixedSize,
                },
                type_name: TypeNameString::try_from(options.type_name.as_str())?,
                size: options.type_size,
                alignment: options.type_alignment,
            })
            .__internal_set_user_header_type_details(&TypeDetail {
                variant: TypeVariant::FixedSize,
                type_name: TypeNameString::try_from(options.header_type_name.as_str())?,
                size: options.header_type_size,
                alignment: options.header_type_alignment,
            })
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
    read_file_into_buffer(&mut message_buffer, &options)?;
    read_cli_msg_into_buffer(&mut message_buffer, &options);

    for _ in 0..options.repetitions {
        for (header, payload) in &message_buffer {
            send_message(header.as_slice(), payload.as_slice(), &publisher, &options)?;
        }
    }

    Ok(())
}
