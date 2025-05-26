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
use crate::DataStream;
use crate::InboundStream;
use crate::OutboundStream;
use crate::PropagationError;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::publisher::PublisherCreateError;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::port::subscriber::SubscriberCreateError;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::publish_subscribe::PublishSubscribeOpenOrCreateError;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::info;
use iceoryx2_bb_log::warn;
use iceoryx2_services_discovery::service_discovery::Discovery;
use iceoryx2_services_discovery::service_discovery::Tracker as IceoryxServiceTracker;

use zenoh::handlers::FifoChannel;
use zenoh::handlers::FifoChannelHandler;
use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::pubsub::Subscriber as ZenohSubscriber;
use zenoh::sample::Locality;
use zenoh::sample::Sample;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use std::collections::HashMap;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum RelayCreationError {
    FailureToCreateIceoryxService,
    FailureToCreateIceoryxPublisher,
    FailureToCreateIceoryxSubscriber,
    FailureToCreateZenohPublisher,
    FailureToCreateZenohSubscriber,
}

impl core::fmt::Display for RelayCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "CreationError::{:?}", self)
    }
}

impl core::error::Error for RelayCreationError {}

/// A bidirectional relay that handles data flow between iceoryx2 and Zenoh.
//
/// This struct manages two data streams:
/// - `outbound_stream`: Transfers data from iceoryx2 to Zenoh
/// - `inbound_stream`: Transfers data from Zenoh to iceoryx2
///
/// It creates a complete bidirectional communication channel between the two middleware systems,
/// allowing services defined in iceoryx2 to be accessible through Zenoh and vice versa.
struct BidirectionalRelay<'a, Service: iceoryx2::service::Service> {
    outbound_stream: OutboundStream<'a, Service>,
    inbound_stream: InboundStream<Service>,
}

impl<'a, Service: iceoryx2::service::Service> BidirectionalRelay<'a, Service> {
    pub fn new(
        iox_service_config: &StaticConfig,
        iox_node: &IceoryxNode<Service>,
        z_session: &ZenohSession,
    ) -> Result<Self, RelayCreationError> {
        let iox_service = iox_create_service::<Service>(iox_node, iox_service_config)
            .map_err(|_e| RelayCreationError::FailureToCreateIceoryxService)?;

        // Create Outbound Stream
        let iox_node_id = iox_node.id();
        let iox_subscriber = iox_create_subscriber::<Service>(&iox_service, iox_service_config)
            .map_err(|_e| RelayCreationError::FailureToCreateIceoryxSubscriber)?;
        let z_publisher = z_create_publisher(z_session, iox_service_config)
            .map_err(|_e| RelayCreationError::FailureToCreateIceoryxPublisher)?;
        let outbound_stream = OutboundStream::new(iox_node_id, iox_subscriber, z_publisher);

        // Create Inbound Stream
        let iox_publisher = iox_create_publisher::<Service>(&iox_service, iox_service_config)
            .map_err(|_e| RelayCreationError::FailureToCreateIceoryxPublisher)?;
        let z_subscriber = z_create_subscriber(z_session, iox_service_config)
            .map_err(|_e| RelayCreationError::FailureToCreateIceoryxSubscriber)?;
        let inbound_stream = InboundStream::new(
            iox_service_config
                .publish_subscribe()
                .message_type_details(),
            iox_publisher,
            z_subscriber,
        );

        Ok(Self {
            outbound_stream,
            inbound_stream,
        })
    }

    pub fn propagate(&self) -> Result<(), PropagationError> {
        self.outbound_stream.propagate()?;
        self.inbound_stream.propagate()?;

        Ok(())
    }
}

pub struct TunnelConfig {
    pub discovery_service: Option<String>,
}

impl Default for TunnelConfig {
    fn default() -> Self {
        Self {
            discovery_service: None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailureToCreateZenohSession,
    FailureToCreateIceoryxNode,
    InvalidNameForDiscoveryService,
    FailureToCreateIceoryxService,
    FailureToCreateIceoryxSubscriber,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "CreationError::{:?}", self)
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    FailedToReceiveUpdatesFromDiscoveryService,
    FailedToSynchronizeWithTracker,
    FailedToEstablishDataStream,
    FailedToReceiveUpdatesFromZenoh,
    FailedToAnnounceServiceOnZenoh,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "DiscoveryError::{:?}", self)
    }
}

impl core::error::Error for DiscoveryError {}

pub struct Tunnel<'a, Service: iceoryx2::service::Service> {
    z_session: ZenohSession,
    iox_config: IceoryxConfig,
    iox_node: IceoryxNode<Service>,
    iox_discovery_service: Option<IceoryxSubscriber<Service, Discovery, ()>>,
    iox_tracker: Option<IceoryxServiceTracker<Service>>,
    relays: HashMap<IceoryxServiceId, BidirectionalRelay<'a, Service>>,
}

impl<'a, Service: iceoryx2::service::Service> Tunnel<'a, Service> {
    /// Creates a new tunnel with the provided configuration.
    pub fn new(
        tunnel_config: &TunnelConfig,
        iox_config: &IceoryxConfig,
    ) -> Result<Self, CreationError> {
        info!("STARTING Zenoh Tunnel");

        // TODO: Take as agument
        let z_config = zenoh::config::Config::default();
        let z_session = zenoh::open(z_config)
            .wait()
            .map_err(|_e| CreationError::FailureToCreateZenohSession)?;

        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<Service>()
            .map_err(|_e| CreationError::FailureToCreateIceoryxNode)?;

        // Create either a discovery service subscriber or a service tracker based on configuration
        let (iox_discovery_service, iox_tracker) = match &tunnel_config.discovery_service {
            Some(value) => {
                let iox_service_name: ServiceName = value
                    .as_str()
                    .try_into()
                    .map_err(|_e| CreationError::InvalidNameForDiscoveryService)?;

                info!("CONFIGURE Discovery updates from service {}", value);
                let iox_service = iox_node
                    .service_builder(&iox_service_name)
                    .publish_subscribe::<Discovery>()
                    .open_or_create()
                    .map_err(|_e| CreationError::FailureToCreateIceoryxService)?;

                let iox_subscriber = iox_service
                    .subscriber_builder()
                    .create()
                    .map_err(|_e| CreationError::FailureToCreateIceoryxSubscriber)?;

                (Some(iox_subscriber), None)
            }
            None => {
                info!("CONFIGURE Internal discovery tracking");
                (None, Some(IceoryxServiceTracker::new()))
            }
        };

        let relays: HashMap<IceoryxServiceId, BidirectionalRelay<Service>> = HashMap::new();

        Ok(Self {
            z_session,
            iox_config: iox_config.clone(),
            iox_node,
            iox_discovery_service,
            iox_tracker,
            relays,
        })
    }

    /// Discover services locally and remotely.
    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        self.iox_discovery()?;
        self.z_discovery()?;

        return Ok(());
    }

    /// Propagates payloads between iceoryx2 and zenoh.
    pub fn propagate(&self) {
        for (id, relay) in &self.relays {
            if let Err(e) = relay.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }
    }

    /// Returns a list of all service IDs that are currently being tunneled.
    pub fn tunneled_services(&self) -> Vec<String> {
        self.relays
            .iter()
            .map(|(id, _)| id.as_str().to_string())
            .collect()
    }

    /// Discovers local services via iceoryx2.
    fn iox_discovery(&mut self) -> Result<(), DiscoveryError> {
        let mut on_discovered = |iox_service_config: &StaticConfig| -> Result<(), DiscoveryError> {
            let iox_service_id = iox_service_config.service_id();

            if !self.relays.contains_key(&iox_service_id) {
                info!(
                    "DISCOVERED (iceoryx2): {} [{}]",
                    iox_service_config.service_id().as_str(),
                    iox_service_config.name()
                );

                // Set up relay
                let relay =
                    BidirectionalRelay::new(iox_service_config, &self.iox_node, &self.z_session)
                        .map_err(|_e| DiscoveryError::FailedToEstablishDataStream)?;
                self.relays.insert(iox_service_id.clone(), relay);

                // Announce Service to Zenoh
                z_announce_service(&self.z_session, iox_service_config)
                    .map_err(|_e| DiscoveryError::FailedToAnnounceServiceOnZenoh)?;
            }
            return Ok(());
        };

        if let Some(iox_discovery_service) = &self.iox_discovery_service {
            // Discovery via discovery service
            loop {
                match iox_discovery_service.receive() {
                    Ok(result) => match result {
                        Some(iox_sample) => {
                            if let Discovery::Added(iox_service_config) = iox_sample.payload() {
                                if let MessagingPattern::PublishSubscribe(_) =
                                    iox_service_config.messaging_pattern()
                                {
                                    on_discovered(&iox_service_config)?;
                                }
                            }
                        }
                        None => break,
                    },
                    Err(_e) => {
                        return Err(DiscoveryError::FailedToReceiveUpdatesFromDiscoveryService);
                    }
                }
            }
        } else if let Some(iox_tracker) = &mut self.iox_tracker {
            // Discovery via service tracker
            let (added, _removed) = iox_tracker
                .sync(&self.iox_config)
                .map_err(|_e| DiscoveryError::FailedToSynchronizeWithTracker)?;

            for iox_service_id in added {
                if let Some(iox_service_details) = iox_tracker.get(&iox_service_id) {
                    let iox_service_config = &iox_service_details.static_details;

                    if let MessagingPattern::PublishSubscribe(_) =
                        iox_service_details.static_details.messaging_pattern()
                    {
                        on_discovered(&iox_service_config)?;
                    }
                }
            }
        } else {
            // Should never happen
            warn!("Unable to discovery iceoryx2 services as neither the service discovery service nor a service tracker are set up.");
        }

        Ok(())
    }

    /// Discovers remote services via zenoh.
    fn z_discovery(&mut self) -> Result<(), DiscoveryError> {
        let iox_service_configs = z_query_services(&self.z_session)
            .map_err(|_e| DiscoveryError::FailedToReceiveUpdatesFromZenoh)?;
        for iox_service_config in iox_service_configs {
            info!(
                "DISCOVERED (zenoh): {} [{}]",
                iox_service_config.service_id().as_str(),
                iox_service_config.name()
            );

            let iox_service_id = iox_service_config.service_id();
            if !self.relays.contains_key(&iox_service_id) {
                let relay =
                    BidirectionalRelay::new(&iox_service_config, &self.iox_node, &self.z_session)
                        .map_err(|_e| DiscoveryError::FailedToEstablishDataStream)?;

                self.relays.insert(iox_service_id.clone(), relay);
            }
        }

        Ok(())
    }
}

fn iox_create_service<Service: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<Service>,
    iox_service_config: &StaticConfig,
) -> Result<
    PortFactory<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublishSubscribeOpenOrCreateError,
> {
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
            .open_or_create()?
    };

    Ok(iox_service)
}

fn iox_create_publisher<Service: iceoryx2::service::Service>(
    iox_service: &PortFactory<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    iox_service_config: &StaticConfig,
) -> Result<
    IceoryxPublisher<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    PublisherCreateError,
> {
    let iox_publisher = iox_service
        .publisher_builder()
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()?;

    info!(
        "NEW PUBLISHER (iceoryx2): {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_publisher)
}

fn iox_create_subscriber<Service: iceoryx2::service::Service>(
    iox_service: &PortFactory<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    iox_service_config: &StaticConfig,
) -> Result<
    IceoryxSubscriber<Service, [CustomPayloadMarker], CustomHeaderMarker>,
    SubscriberCreateError,
> {
    let iox_subscriber = iox_service.subscriber_builder().create()?;

    info!(
        "NEW SUBSCRIBER (iceoryx2): {} [{}]",
        iox_service_config.service_id().as_str(),
        iox_service_config.name()
    );

    Ok(iox_subscriber)
}

fn z_create_publisher<'a>(
    z_session: &ZenohSession,
    iox_service_config: &StaticConfig,
) -> Result<ZenohPublisher<'a>, zenoh::Error> {
    let z_key = keys::data_stream(iox_service_config.service_id());
    info!("NEW PUBLISHER (zenoh): {}", z_key.clone());
    let z_publisher = z_session.declare_publisher(z_key).wait()?;

    Ok(z_publisher)
}

fn z_create_subscriber(
    z_session: &ZenohSession,
    iox_service_config: &StaticConfig,
) -> Result<ZenohSubscriber<FifoChannelHandler<Sample>>, zenoh::Error> {
    let z_key = keys::data_stream(iox_service_config.service_id());
    let z_subscriber = z_session
        .declare_subscriber(z_key)
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()?;

    Ok(z_subscriber)
}

fn z_announce_service(
    z_session: &ZenohSession,
    iox_service_config: &StaticConfig,
) -> Result<(), zenoh::Error> {
    let z_key = keys::service(iox_service_config.service_id());
    match serde_json::to_string(&iox_service_config) {
        Ok(iox_static_details_json) => {
            z_session
                .declare_queryable(z_key.clone())
                .callback(move |query| {
                    if let Err(e) = query
                        .reply(query.key_expr().clone(), &iox_static_details_json)
                        .wait()
                    {
                        error!("Failed to reply to query for service info: {}", e);
                    }
                })
                .background()
                .wait()?;

            info!("ANNOUNCING (zenoh): {}", z_key);
        }
        Err(e) => {
            error!("Failed to serialize static details to JSON: {}", e);
        }
    }
    Ok(())
}

fn z_query_services(z_session: &ZenohSession) -> Result<Vec<StaticConfig>, zenoh::Error> {
    let mut iox_remote_static_details = Vec::new();

    let replies = z_session
        .get(keys::all_services())
        .allowed_destination(Locality::Remote)
        .wait()?;

    while let Ok(reply) = replies.try_recv() {
        match reply {
            // Case: Reply contains a sample (actual data from a service)
            Some(sample) => match sample.result() {
                // Case: Sample contains valid data that can be processed
                Ok(sample) => {
                    match serde_json::from_slice::<StaticConfig>(&sample.payload().to_bytes()) {
                        Ok(iox_static_details) => {
                            if !iox_remote_static_details
                                .iter()
                                .any(|details: &StaticConfig| {
                                    details.service_id() == iox_static_details.service_id()
                                })
                            {
                                iox_remote_static_details.push(iox_static_details.clone());
                            }
                        }
                        Err(e) => {
                            error!("Failed to deserialize static details: {}", e);
                        }
                    }
                }
                // Case: Sample contains an error (e.g., malformed data)
                Err(e) => {
                    error!("Invalid sample: {}", e);
                }
            },
            // Case: Reply exists but contains no sample (empty response)
            None => { /* Nothing to do */ }
        }
    }

    Ok(iox_remote_static_details)
}
