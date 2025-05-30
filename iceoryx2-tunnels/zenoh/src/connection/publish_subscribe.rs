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

use super::Connection;
use super::PropagationError;
use crate::iox_create_publish_subscribe_service;
use crate::iox_create_publisher;
use crate::iox_create_subscriber;
use crate::z_announce_service;
use crate::z_create_publisher;
use crate::z_create_subscriber;

use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::node::NodeId as IceoryxNodeId;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
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

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    IceoryxService,
    IceoryxPublisher,
    IceoryxSubscriber,
    ZenohPublisher,
    ZenohSubscriber,
    ZenohAnnouncement,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "CreationError::{:?}", self)
    }
}

impl core::error::Error for CreationError {}

/// A connection for propagating `iceoryx2` publish-subscribe payloads to remote hosts.
pub(crate) struct OutboundPublishSubscribeConnection<'a, ServiceType: iceoryx2::service::Service> {
    iox_node_id: IceoryxNodeId,
    iox_service_config: IceoryxServiceConfig,
    iox_subscriber: IceoryxSubscriber<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
    z_publisher: ZenohPublisher<'a>,
}

impl<ServiceType: iceoryx2::service::Service> Connection
    for OutboundPublishSubscribeConnection<'_, ServiceType>
{
    /// Propagate local payloads to remote hosts.
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
                        return Err(PropagationError::Outbound);
                    }

                    info!(
                        "PROPAGATED(iceoryx2->zenoh): PUBLISH_SUBSCRIBE {} [{}]",
                        self.iox_service_config.service_id().as_str(),
                        self.iox_service_config.name()
                    );
                }
                Ok(None) => break, // No more samples available
                Err(e) => {
                    error!("Failed to receive custom payload from iceoryx: {}", e);
                    return Err(PropagationError::Outbound);
                }
            }
        }

        Ok(())
    }
}

impl<ServiceType: iceoryx2::service::Service> OutboundPublishSubscribeConnection<'_, ServiceType> {
    // Creates an outbound connection to remote hosts for publish-subscribe payloads for a
    // particular service.
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
            .map_err(|_e| CreationError::IceoryxSubscriber)?;

        let z_publisher = z_create_publisher(z_session, iox_service_config)
            .map_err(|_e| CreationError::ZenohPublisher)?;

        Ok(Self {
            iox_node_id: *iox_node_id,
            iox_service_config: iox_service_config.clone(),
            iox_subscriber,
            z_publisher,
        })
    }
}

/// A connection for propagating `iceoryx2` publish-subscribe payloads from remote hosts.
pub(crate) struct InboundPublishSubscribeConnection<ServiceType: iceoryx2::service::Service> {
    iox_service_config: IceoryxServiceConfig,
    iox_publisher: IceoryxPublisher<ServiceType, [CustomPayloadMarker], CustomHeaderMarker>,
    z_subscriber: ZenohSubscriber<FifoChannelHandler<Sample>>,
}

impl<ServiceType: iceoryx2::service::Service> Connection
    for InboundPublishSubscribeConnection<ServiceType>
{
    /// Propagate remote payloads to the local host.
    fn propagate(&self) -> Result<(), PropagationError> {
        for z_sample in self.z_subscriber.drain() {
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
                            return Err(PropagationError::Inbound);
                        }
                        info!(
                            "PROPAGATED(iceoryx2<-zenoh): PUBLISH_SUBSCRIBE {} [{}]",
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
                        return Err(PropagationError::Inbound);
                    }
                }
            }
        }

        Ok(())
    }
}

impl<ServiceType: iceoryx2::service::Service> InboundPublishSubscribeConnection<ServiceType> {
    // Creates an inbound connection to remote hosts for publish-subscribe payloads for a
    // particular service.
    pub fn create(
        iox_service_config: &IceoryxServiceConfig,
        iox_publish_subscribe_service: &IceoryxPublishSubscribeService<
            ServiceType,
            [CustomPayloadMarker],
            CustomHeaderMarker,
        >,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        let iox_publisher =
            iox_create_publisher::<ServiceType>(iox_publish_subscribe_service, iox_service_config)
                .map_err(|_e| CreationError::IceoryxPublisher)?;
        let z_subscriber = z_create_subscriber(z_session, iox_service_config)
            .map_err(|_e| CreationError::ZenohSubscriber)?;

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_publisher,
            z_subscriber,
        })
    }
}

/// Couples the outbound and inbound connection for a particular iceoryx2 service.
pub(crate) struct BidirectionalPublishSubscribeConnection<
    'a,
    ServiceType: iceoryx2::service::Service,
> {
    outbound_connection: OutboundPublishSubscribeConnection<'a, ServiceType>,
    inbound_connection: InboundPublishSubscribeConnection<ServiceType>,
}

impl<ServiceType: iceoryx2::service::Service> Connection
    for BidirectionalPublishSubscribeConnection<'_, ServiceType>
{
    /// Propagate local payloads to remote host and remote payloads to the local host.
    fn propagate(&self) -> Result<(), PropagationError> {
        self.outbound_connection.propagate()?;
        self.inbound_connection.propagate()?;

        Ok(())
    }
}

impl<ServiceType: iceoryx2::service::Service>
    BidirectionalPublishSubscribeConnection<'_, ServiceType>
{
    /// Create a bi-directional connection to propagate payloads for a particular iceoryx2 service
    /// to and from remote iceoryx2 instances via Zenoh.
    pub fn create(
        iox_node: &IceoryxNode<ServiceType>,
        z_session: &ZenohSession,
        iox_service_config: &IceoryxServiceConfig,
    ) -> Result<Self, CreationError> {
        let iox_publish_subscribe_service =
            iox_create_publish_subscribe_service::<ServiceType>(iox_node, iox_service_config)
                .map_err(|_e| CreationError::IceoryxService)?;

        z_announce_service(&z_session, iox_service_config)
            .map_err(|_e| CreationError::ZenohAnnouncement)?;

        let outbound_connection = OutboundPublishSubscribeConnection::create(
            iox_node.id(),
            iox_service_config,
            &iox_publish_subscribe_service,
            z_session,
        )?;
        let inbound_connection = InboundPublishSubscribeConnection::create(
            iox_service_config,
            &iox_publish_subscribe_service,
            z_session,
        )?;

        Ok(Self {
            outbound_connection,
            inbound_connection,
        })
    }
}
