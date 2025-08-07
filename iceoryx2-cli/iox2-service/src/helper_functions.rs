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

use anyhow::{anyhow, Result};
use iceoryx2::service::builder::{CustomHeaderMarker, CustomPayloadMarker};
use iceoryx2::{
    prelude::*,
    sample::Sample,
    service::{
        header::publish_subscribe::Header,
        static_config::message_type_details::{TypeDetail, TypeVariant},
    },
};
use iceoryx2_userland_record_and_replay::prelude::ServiceTypes;

pub fn get_pubsub_service_types(
    service_name: &ServiceName,
    node: &Node<ipc::Service>,
) -> Result<ServiceTypes> {
    let service_details = match ipc::Service::details(
        service_name,
        node.config(),
        MessagingPattern::PublishSubscribe,
    )? {
        Some(v) => v,
        None => {
            return Err(anyhow!(
                "unable to access service \"{service_name}\", does it exist?",
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

pub fn extract_pubsub_payload<'a>(
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
