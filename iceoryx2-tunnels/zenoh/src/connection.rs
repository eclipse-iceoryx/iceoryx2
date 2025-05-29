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

use crate::keys;

use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::node::NodeId as IceoryxNodeId;
use iceoryx2::port::notifier::Notifier as IceoryxNotifier;
use iceoryx2::port::notifier::NotifierCreateError;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::port::subscriber::SubscriberCreateError;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::event::EventOpenOrCreateError;
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::event::PortFactory as IceoryxEventService;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory as IceoryxPublishSubscribeService;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::info;

use zenoh::bytes::ZBytes;
use zenoh::handlers::FifoChannel;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Locality;
use zenoh::sample::Sample;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    IceoryxService,
    IceoryxPublisher,
    IceoryxSubscriber,
    IceoryxNotifier,
    ZenohPublisher,
    ZenohSubscriber,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "CreationError::{:?}", self)
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagationError {
    ReceiveFromIceoryx,
    PublishToIceoryx,
    NotifyIceoryx,
    PublishToZenoh,
}

impl core::fmt::Display for PropagationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "PropagationError::{:?}", self)
    }
}

impl core::error::Error for PropagationError {}

pub trait Connection {
    fn propagate(&self) -> Result<(), PropagationError>;
}

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
                        return Err(PropagationError::PublishToZenoh);
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
                    return Err(PropagationError::ReceiveFromIceoryx);
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
    iox_notifier: IceoryxNotifier<ServiceType>,
    z_subscriber: ZenohSubscriber<FifoChannelHandler<Sample>>,
}

impl<ServiceType: iceoryx2::service::Service> Connection
    for InboundPublishSubscribeConnection<ServiceType>
{
    /// Propagate remote payloads to the local host.
    fn propagate(&self) -> Result<(), PropagationError> {
        let mut propagated = false;

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
                            return Err(PropagationError::PublishToIceoryx);
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
                        return Err(PropagationError::PublishToIceoryx);
                    }
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
                return Err(PropagationError::NotifyIceoryx);
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
        iox_event_service: &IceoryxEventService<ServiceType>,
        z_session: &ZenohSession,
    ) -> Result<Self, CreationError> {
        let iox_publisher =
            iox_create_publisher::<ServiceType>(iox_publish_subscribe_service, iox_service_config)
                .map_err(|_e| CreationError::IceoryxPublisher)?;
        let iox_notifier = iox_create_notifier(iox_event_service, iox_service_config)
            .map_err(|_e| CreationError::IceoryxNotifier)?;
        let z_subscriber = z_create_subscriber(z_session, iox_service_config)
            .map_err(|_e| CreationError::ZenohSubscriber)?;

        Ok(Self {
            iox_service_config: iox_service_config.clone(),
            iox_publisher,
            iox_notifier,
            z_subscriber,
        })
    }
}

/// Couples the outbound and inbound connection for a particular iceoryx2 service.
pub(crate) struct BidirectionalPublishSubscribeConnection<
    'a,
    ServiceType: iceoryx2::service::Service,
> {
    outbound_stream: OutboundPublishSubscribeConnection<'a, ServiceType>,
    inbound_stream: InboundPublishSubscribeConnection<ServiceType>,
}

impl<ServiceType: iceoryx2::service::Service> Connection
    for BidirectionalPublishSubscribeConnection<'_, ServiceType>
{
    /// Propagate local payloads to remote host and remote payloads to the local host.
    fn propagate(&self) -> Result<(), PropagationError> {
        self.outbound_stream.propagate()?;
        self.inbound_stream.propagate()?;

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
        let iox_event_service = iox_create_event_service(iox_node, iox_service_config)
            .map_err(|_e| CreationError::IceoryxService)?;

        let outbound_stream = OutboundPublishSubscribeConnection::create(
            iox_node.id(),
            iox_service_config,
            &iox_publish_subscribe_service,
            z_session,
        )?;
        let inbound_stream = InboundPublishSubscribeConnection::create(
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

/// Creates an iceoryx2 publish-subscribe service matching the provided service configuration.
pub(crate) fn iox_create_publish_subscribe_service<Service: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<Service>,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<
    IceoryxPublishSubscribeService<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublishSubscribeOpenOrCreateError,
> {
    let iox_publish_subscribe_config = iox_service_config.publish_subscribe();
    let iox_service = unsafe {
        iox_node
            .service_builder(iox_service_config.name())
            .publish_subscribe::<[CustomPayloadMarker]>()
            .user_header::<CustomHeaderMarker>()
            .__internal_set_user_header_type_details(
                &iox_service_config
                    .publish_subscribe()
                    .message_type_details()
                    .user_header,
            )
            .__internal_set_payload_type_details(
                &iox_service_config
                    .publish_subscribe()
                    .message_type_details()
                    .payload,
            )
            .enable_safe_overflow(iox_publish_subscribe_config.has_safe_overflow())
            .history_size(iox_publish_subscribe_config.history_size())
            .max_publishers(iox_publish_subscribe_config.max_publishers())
            .max_subscribers(iox_publish_subscribe_config.max_subscribers())
            .subscriber_max_buffer_size(iox_publish_subscribe_config.subscriber_max_buffer_size())
            .subscriber_max_buffer_size(
                iox_publish_subscribe_config.subscriber_max_borrowed_samples(),
            )
            .open_or_create()?
    };

    Ok(iox_service)
}

/// Creates an iceoryx2 event service matching the provided service configuration.
pub(crate) fn iox_create_event_service<Service: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<Service>,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<IceoryxEventService<Service>, EventOpenOrCreateError> {
    // TODO(correctness): Use properties of existing event services to prevent clashing
    let iox_service = iox_node
        .service_builder(iox_service_config.name())
        .event()
        .open_or_create()?;

    Ok(iox_service)
}

/// Creates an iceoryx2 publisher to the provided service.
pub(crate) fn iox_create_publisher<Service: iceoryx2::service::Service>(
    iox_publish_subscribe_service: &IceoryxPublishSubscribeService<
        Service,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<
    IceoryxPublisher<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublisherCreateError,
> {
    let iox_publisher = iox_publish_subscribe_service
        .publisher_builder()
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    info!(
        "CREATED PUBLISHER (iceoryx2): {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_publisher)
}

/// Creates an iceoryx2 subscriber to the provided service.
pub(crate) fn iox_create_subscriber<Service: iceoryx2::service::Service>(
    iox_publish_subscribe_service: &IceoryxPublishSubscribeService<
        Service,
        [CustomPayloadMarker],
        CustomHeaderMarker,
    >,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<
    IceoryxSubscriber<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    SubscriberCreateError,
> {
    let iox_subscriber = iox_publish_subscribe_service
        .subscriber_builder()
        .create()?;

    info!(
        "CREATED SUBSCRIBER (iceoryx2): {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_subscriber)
}

/// Creates an iceoryx2 notifier to the provided service.
pub(crate) fn iox_create_notifier<Service: iceoryx2::service::Service>(
    iox_event_service: &IceoryxEventService<Service>,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<IceoryxNotifier<Service>, NotifierCreateError> {
    let iox_notifier = iox_event_service.notifier_builder().create()?;

    info!(
        "CREATED NOTIFIER (iceoryx2): {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_notifier)
}

/// Creates a Zenoh publisher to send payloads from iceoryx2 services to remote hosts.
pub(crate) fn z_create_publisher<'a>(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<ZenohPublisher<'a>, zenoh::Error> {
    let z_key = keys::publish_subscribe(iox_service_config.service_id());
    let z_publisher = z_session
        .declare_publisher(z_key.clone())
        .allowed_destination(Locality::Remote)
        .wait()?;
    info!(
        "CREATED PUBLISHER (zenoh): {} [{}]",
        z_key,
        iox_service_config.name()
    );

    Ok(z_publisher)
}

/// Creates a Zenoh subscriber to receive payloads from remote hosts for a particular iceoryx2 service.
pub(crate) fn z_create_subscriber(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<ZenohSubscriber<FifoChannelHandler<Sample>>, zenoh::Error> {
    let z_key = keys::publish_subscribe(iox_service_config.service_id());
    let z_subscriber = z_session
        .declare_subscriber(z_key.clone())
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()?;
    info!(
        "CREATED SUBSCRIBER (zenoh): {} [{}]",
        z_key,
        iox_service_config.name()
    );

    Ok(z_subscriber)
}

/// Announces an iceoryx2 service over Zenoh to make it discoverable by remote hosts.
pub(crate) fn z_announce_service(
    z_session: &ZenohSession,
    iox_service_config: &IceoryxServiceConfig,
) -> Result<(), zenoh::Error> {
    let z_key = keys::service_details(iox_service_config.service_id());
    let iox_service_config_serialized = serde_json::to_string(&iox_service_config)?;

    info!(
        "ANNOUNCING (zenoh): {} [{}]",
        z_key,
        iox_service_config.name()
    );

    z_session
        .declare_queryable(z_key.clone())
        .callback(move |query| {
            if let Err(e) = query
                .reply(z_key.clone(), iox_service_config_serialized.clone())
                .wait()
            {
                error!("Failed to reply to query {}: {}", z_key, e);
            }
        })
        .allowed_origin(Locality::Remote)
        .background()
        .wait()?;

    Ok(())
}
