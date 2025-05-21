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

use iceoryx2::node::NodeId;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::static_config::message_type_details::MessageTypeDetails;

use zenoh::bytes::ZBytes;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Sample;
use zenoh::Wait;

pub trait DataStream {
    fn propagate(&self);
}

pub struct OutboundStream<'a, Service: iceoryx2::service::Service> {
    iox_node_id: NodeId,
    iox_subscriber: IceoryxSubscriber<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    z_publisher: ZenohPublisher<'a>,
}

impl<'a, Service: iceoryx2::service::Service> DataStream for OutboundStream<'a, Service> {
    fn propagate(&self) {
        while let Ok(Some(sample)) = unsafe { self.iox_subscriber.receive_custom_payload() } {
            if sample.header().node_id() == self.iox_node_id {
                // Ignore samples published by the gateway itself to prevent loopback.
                continue;
            }

            let ptr = sample.payload().as_ptr() as *const u8;
            let len = sample.len();
            let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };

            let z_payload = ZBytes::from(bytes);
            self.z_publisher.put(z_payload).wait().unwrap();
        }
    }
}

impl<'a, Service: iceoryx2::service::Service> OutboundStream<'a, Service> {
    pub fn new(
        iox_node_id: &NodeId,
        iox_subscriber: IceoryxSubscriber<Service, [CustomPayloadMarker], CustomHeaderMarker>,
        z_publisher: ZenohPublisher<'a>,
    ) -> Self {
        Self {
            iox_node_id: iox_node_id.clone(),
            iox_subscriber,
            z_publisher,
        }
    }
}

pub struct InboundStream<Service: iceoryx2::service::Service> {
    iox_message_type_details: MessageTypeDetails,
    iox_publisher: IceoryxPublisher<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    z_subscriber: ZenohSubscriber<FifoChannelHandler<Sample>>,
}

impl<Service: iceoryx2::service::Service> DataStream for InboundStream<Service> {
    fn propagate(&self) {
        while let Ok(Some(z_sample)) = self.z_subscriber.try_recv() {
            let iox_payload_size = self.iox_message_type_details.payload.size;
            let _iox_payload_alignment = self.iox_message_type_details.payload.alignment;

            // TODO: verify size and alignment
            let z_payload = z_sample.payload();

            let number_of_elements = z_payload.len() / iox_payload_size;
            unsafe {
                let mut iox_sample = self
                    .iox_publisher
                    .loan_custom_payload(number_of_elements)
                    .unwrap();
                std::ptr::copy_nonoverlapping(
                    z_payload.to_bytes().as_ptr(),
                    iox_sample.payload_mut().as_mut_ptr() as *mut u8,
                    z_payload.len(),
                );
                let iox_sample = iox_sample.assume_init();
                iox_sample.send().unwrap();
            }
        }
    }
}

impl<Service: iceoryx2::service::Service> InboundStream<Service> {
    pub fn new(
        iox_message_type_details: &MessageTypeDetails,
        iox_publisher: IceoryxPublisher<Service, [CustomPayloadMarker], CustomHeaderMarker>,
        z_subscriber: ZenohSubscriber<FifoChannelHandler<Sample>>,
    ) -> Self {
        Self {
            iox_message_type_details: iox_message_type_details.clone(),
            iox_publisher,
            z_subscriber,
        }
    }
}
