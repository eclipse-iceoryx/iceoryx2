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

use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2_bb_log::info;

use zenoh::bytes::ZBytes;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::Session as ZenohSession;
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
        z_session: &ZenohSession,
        iox_node: &IceoryxNode<ipc::Service>,
        iox_service_details: &ServiceDetails<ipc::Service>,
    ) -> Self {
        let iox_subscriber = Self::create_iox_subscriber(iox_node, iox_service_details);
        let z_publisher = Self::create_zenoh_publisher(z_session, iox_service_details);

        Self::Outbound {
            iox_service_id: iox_service_details.static_details.service_id().clone(),
            iox_subscriber,
            z_publisher,
        }
    }

    pub fn propagate(&self) {
        match self {
            DataStream::Outbound {
                iox_service_id,
                iox_subscriber,
                z_publisher,
            } => {
                info!("PROPAGATING (iceoryx2->zenoh): {}", iox_service_id.as_str());

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

    fn create_iox_subscriber(
        iox_node: &IceoryxNode<ipc::Service>,
        iox_service_details: &ServiceDetails<ipc::Service>,
    ) -> IceoryxSubscriber<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker> {
        unsafe {
            let iox_service = iox_node
                .service_builder(iox_service_details.static_details.name())
                .publish_subscribe::<[CustomPayloadMarker]>()
                .user_header::<CustomHeaderMarker>()
                .__internal_set_user_header_type_details(
                    &iox_service_details
                        .static_details
                        .publish_subscribe()
                        .message_type_details()
                        .user_header,
                )
                .__internal_set_payload_type_details(
                    &iox_service_details
                        .static_details
                        .publish_subscribe()
                        .message_type_details()
                        .payload,
                )
                .open_or_create()
                .unwrap();

            let iox_subscriber = iox_service.subscriber_builder().create().unwrap();
            info!(
                "ADD SUBSCRIBER (iceoryx2): {} [{}]",
                iox_service_details.static_details.name(),
                iox_service_details.static_details.service_id().as_str()
            );

            iox_subscriber
        }
    }

    fn create_zenoh_publisher(
        z_session: &ZenohSession,
        iox_service_details: &ServiceDetails<ipc::Service>,
    ) -> ZenohPublisher<'a> {
        let z_key = &format!(
            "iox2/{}",
            iox_service_details.static_details.service_id().as_str()
        );
        let z_publisher = z_session.declare_publisher(z_key.clone()).wait().unwrap();

        info!("ADD PUBLISHER (zenoh): {}", z_key);

        z_publisher
    }
}
