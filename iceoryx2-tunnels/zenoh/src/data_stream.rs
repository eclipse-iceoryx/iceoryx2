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

use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;

use zenoh::bytes::ZBytes;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::Wait;

pub enum DataStream<'a> {
    Outbound {
        iox_service_id: IceoryxServiceId,
        iox_subscriber: IceoryxSubscriber<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker>,
        z_publisher: ZenohPublisher<'a>,
    },
}

impl<'a> DataStream<'a> {
    pub fn new_outbound(
        iox_service_id: &IceoryxServiceId,
        iox_subscriber: IceoryxSubscriber<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker>,
        z_publisher: ZenohPublisher<'a>,
    ) -> Self {
        Self::Outbound {
            iox_service_id: iox_service_id.clone(),
            iox_subscriber,
            z_publisher,
        }
    }

    pub fn propagate(&self) {
        match self {
            DataStream::Outbound {
                iox_service_id: _,
                iox_subscriber,
                z_publisher,
            } => {
                while let Ok(Some(sample)) = unsafe { iox_subscriber.receive_custom_payload() } {
                    let ptr = sample.payload().as_ptr() as *const u8;
                    let len = sample.len();
                    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };

                    let z_payload = ZBytes::from(bytes);
                    z_publisher.put(z_payload).wait().unwrap();
                }
            }
        }
    }
}
