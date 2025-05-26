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
use iceoryx2_bb_log::error;

use zenoh::bytes::ZBytes;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Sample;
use zenoh::Wait;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {
    /// Error that occurs when receiving data from an Iceoryx subscriber fails
    IceoryxReceiveFailure,
    /// Error that occurs when publishing data to an Iceoryx publisher fails
    IceoryxPublishFailure,
    /// Error that occurs when receiving data from a Zenoh subscriber fails
    ZenohReceiveFailure,
    /// Error that occurs when publishing data to a Zenoh publisher fails
    ZenohPublishFailure,
}

impl core::fmt::Display for PropagationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "PropagationError::{:?}", self)
    }
}

impl core::error::Error for PropagationError {}

pub trait DataStream {
    fn propagate(&self) -> Result<(), PropagationError>;
}

pub struct OutboundStream<'a, Service: iceoryx2::service::Service> {
    iox_node_id: NodeId,
    iox_subscriber: IceoryxSubscriber<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    z_publisher: ZenohPublisher<'a>,
}

impl<'a, Service: iceoryx2::service::Service> DataStream for OutboundStream<'a, Service> {
    fn propagate(&self) -> Result<(), PropagationError> {
        loop {
            match unsafe { self.iox_subscriber.receive_custom_payload() } {
                Ok(Some(sample)) => {
                    if sample.header().node_id() == self.iox_node_id {
                        // Ignore samples published by the gateway itself to prevent loopback.
                        continue;
                    }

                    let ptr = sample.payload().as_ptr() as *const u8;
                    let len = sample.len();
                    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };

                    let z_payload = ZBytes::from(bytes);
                    if let Err(e) = self.z_publisher.put(z_payload).wait() {
                        error!("failed to propagate payload to zenoh: {}", e);
                        return Err(PropagationError::ZenohPublishFailure);
                    }
                }
                Ok(None) => break, // No more samples available
                Err(e) => {
                    error!("failed to receive custom payload from iceoryx: {}", e);
                    return Err(PropagationError::IceoryxReceiveFailure);
                }
            }
        }

        Ok(())
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
    fn propagate(&self) -> Result<(), PropagationError> {
        loop {
            match self.z_subscriber.try_recv() {
                Ok(Some(z_sample)) => {
                    let iox_payload_size = self.iox_message_type_details.payload.size;
                    let _iox_payload_alignment = self.iox_message_type_details.payload.alignment;

                    // TODO: verify size and alignment
                    let z_payload = z_sample.payload();

                    let number_of_elements = z_payload.len() / iox_payload_size;
                    unsafe {
                        match self.iox_publisher.loan_custom_payload(number_of_elements) {
                            Ok(mut iox_sample) => {
                                core::ptr::copy_nonoverlapping(
                                    z_payload.to_bytes().as_ptr(),
                                    iox_sample.payload_mut().as_mut_ptr() as *mut u8,
                                    z_payload.len(),
                                );
                                let iox_sample = iox_sample.assume_init();
                                if let Err(e) = iox_sample.send() {
                                    error!("failed to send custom payload to iceoryx: {}", e);
                                    return Err(PropagationError::IceoryxPublishFailure);
                                }
                            }
                            Err(e) => {
                                error!("failed to loan custom payload from iceoryx: {}", e);
                                return Err(PropagationError::IceoryxPublishFailure);
                            }
                        }
                    }
                }
                Ok(None) => break, // No more samples available
                Err(e) => {
                    error!("failed to receive payload from Zenoh: {}", e);
                    return Err(PropagationError::ZenohReceiveFailure);
                }
            }
        }

        Ok(())
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
