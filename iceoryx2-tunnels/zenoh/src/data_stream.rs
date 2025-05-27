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
use iceoryx2::node::NodeId as IceoryxNodeId;
use iceoryx2::port::notifier::Notifier as IceoryxNotifier;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::event::PortFactory as IceoryxEventService;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPublishSubscribeService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::error;

use iceoryx2_bb_log::info;
use zenoh::bytes::ZBytes;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Sample;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use crate::iox_create_event_service;
use crate::iox_create_notifier;
use crate::iox_create_publish_subscribe_service;
use crate::iox_create_publisher;
use crate::iox_create_subscriber;
use crate::z_create_publisher;
use crate::z_create_subscriber;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailureToCreateIceoryxService,
    FailureToCreateIceoryxPublisher,
    FailureToCreateIceoryxSubscriber,
    FailureToCreateIceoryxNotifier,
    FailureToCreateZenohPublisher,
    FailureToCreateZenohSubscriber,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "CreationError::{:?}", self)
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {
    FailureToReceiveFromIceoryx,
    FailureToPublishToIceoryx,
    FailureToNotifyIceoryx,
    FailureToReceiveFromZenoh,
    FailureToPublishToZenoh,
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

pub(crate) struct OutboundStream<'a, ServiceType: iceoryx2::service::Service> {
    iox_node_id: IceoryxNodeId,
    iox_service_config: IceoryxServiceConfig,
    iox_subscriber: IceoryxSubscriber<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
    z_publisher: ZenohPublisher<'a>,
}

impl<'a, ServiceType: iceoryx2::service::Service> DataStream for OutboundStream<'a, ServiceType> {
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
                        error!("Failed to propagate payload to zenoh: {}", e);
                        return Err(PropagationError::FailureToPublishToZenoh);
                    }

                    info!(
                        "PROPAGATED (iceoryx2->zenoh): {} [{}]",
                        self.iox_service_config.service_id().as_str(),
                        self.iox_service_config.name()
                    );
                }
                Ok(None) => break, // No more samples available
                Err(e) => {
                    error!("Failed to receive custom payload from iceoryx: {}", e);
                    return Err(PropagationError::FailureToReceiveFromIceoryx);
                }
            }
        }

        Ok(())
    }
}

impl<'a, ServiceType: iceoryx2::service::Service> OutboundStream<'a, ServiceType> {
    pub fn create(
        iox_node_id: &IceoryxNodeId,
        iox_service_config: &IceoryxServiceConfig,
        iox_service: &IceoryxPublishSubscribeService<
            ServiceType,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        let iox_subscriber = iox_create_subscriber::<ServiceType>(iox_service, iox_service_config)
            .map_err(|_e| CreationError::FailureToCreateIceoryxSubscriber)?;
        let z_publisher = z_create_publisher(z_session, iox_service_config)
            .map_err(|_e| CreationError::FailureToCreateZenohPublisher)?;

        Ok(Self {
            iox_node_id: iox_node_id.clone(),
            iox_service_config: iox_service_config.clone(),
            iox_subscriber,
            z_publisher,
        })
    }
}

pub(crate) struct InboundStream<ServiceType: iceoryx2::service::Service> {
    iox_service_config: IceoryxServiceConfig,
    iox_publisher: IceoryxPublisher<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
    iox_notifier: IceoryxNotifier<ServiceType>,
    z_subscriber: ZenohSubscriber<FifoChannelHandler<Sample>>,
}

impl<ServiceType: iceoryx2::service::Service> DataStream for InboundStream<ServiceType> {
    fn propagate(&self) -> Result<(), PropagationError> {
        let mut propagated = false;
        loop {
            match self.z_subscriber.try_recv() {
                Ok(Some(z_sample)) => {
                    let iox_message_type_details = self
                        .iox_service_config
                        .publish_subscribe()
                        .message_type_details();
                    let iox_payload_size = iox_message_type_details.payload.size;
                    let _iox_payload_alignment = iox_message_type_details.payload.alignment;

                    // TODO(correctness): verify size and alignment
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
                                    error!(
                                        "Failed to publish sample ({}): {}",
                                        self.iox_service_config.name(),
                                        e
                                    );
                                    return Err(PropagationError::FailureToPublishToIceoryx);
                                }
                                propagated = true;
                                info!(
                                    "PROPAGATED (iceoryx2<-zenoh): {} [{}]",
                                    self.iox_service_config.service_id().as_str(),
                                    self.iox_service_config.name()
                                );
                            }
                            Err(e) => {
                                error!(
                                    "Failed to loan sample ({}): {}",
                                    self.iox_service_config.name(),
                                    e
                                );
                                return Err(PropagationError::FailureToPublishToIceoryx);
                            }
                        }
                    }
                }
                Ok(None) => break, // No more samples available
                Err(e) => {
                    error!("Failed to receive payload from Zenoh: {}", e);
                    return Err(PropagationError::FailureToReceiveFromZenoh);
                }
            }
        }

        if propagated {
            if let Err(e) = self.iox_notifier.notify() {
                error!(
                    "Failed to notify service ({}): {}",
                    self.iox_service_config.name(),
                    e
                );
                return Err(PropagationError::FailureToNotifyIceoryx);
            }
        }

        Ok(())
    }
}

impl<ServiceType: iceoryx2::service::Service> InboundStream<ServiceType> {
    pub fn create(
        iox_service_config: &IceoryxServiceConfig,
        iox_publish_subscribe_service: &IceoryxPublishSubscribeService<
            ServiceType,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
        iox_event_service: &IceoryxEventService<ServiceType>,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        let iox_publisher =
            iox_create_publisher::<ServiceType>(iox_publish_subscribe_service, iox_service_config)
                .map_err(|_e| CreationError::FailureToCreateIceoryxPublisher)?;
        let iox_notifier = iox_create_notifier(iox_event_service, iox_service_config)
            .map_err(|_e| CreationError::FailureToCreateIceoryxNotifier)?;
        let z_subscriber = z_create_subscriber(z_session, iox_service_config)
            .map_err(|_e| CreationError::FailureToCreateZenohSubscriber)?;

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_publisher,
            iox_notifier,
            z_subscriber,
        })
    }
}

pub(crate) struct BidirectionalStream<'a, ServiceType: iceoryx2::service::Service> {
    outbound_stream: OutboundStream<'a, ServiceType>,
    inbound_stream: InboundStream<ServiceType>,
}

impl<'a, ServiceType: iceoryx2::service::Service> DataStream
    for BidirectionalStream<'a, ServiceType>
{
    fn propagate(&self) -> Result<(), PropagationError> {
        self.outbound_stream.propagate()?;
        self.inbound_stream.propagate()?;

        Ok(())
    }
}

impl<'a, ServiceType: iceoryx2::service::Service> BidirectionalStream<'a, ServiceType> {
    pub fn create(
        iox_node: &IceoryxNode<ServiceType>,
        z_session: &ZenohSession,
        iox_service_config: &IceoryxServiceConfig,
    ) -> Result<Self, CreationError> {
        let iox_publish_subscribe_service =
            iox_create_publish_subscribe_service::<ServiceType>(iox_node, iox_service_config)
                .map_err(|_e| CreationError::FailureToCreateIceoryxService)?;
        let iox_event_service = iox_create_event_service(iox_node, iox_service_config)
            .map_err(|_e| CreationError::FailureToCreateIceoryxService)?;

        let outbound_stream = OutboundStream::create(
            &iox_node.id(),
            iox_service_config,
            &iox_publish_subscribe_service,
            z_session,
        )?;
        let inbound_stream = InboundStream::create(
            iox_service_config,
            &iox_publish_subscribe_service,
            &iox_event_service,
            z_session,
        )?;

        Ok(Self {
            outbound_stream,
            inbound_stream,
        })
    }
}
