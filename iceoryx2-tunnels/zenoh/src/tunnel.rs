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

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::publisher::Publisher as IceoryxPublisher;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::ipc::Service as IceoryxService;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2_bb_log::info;
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

/// A bidirectional relay that handles data flow between iceoryx2 and Zenoh.
///
/// This struct manages two data streams:
/// - `outbound_stream`: Transfers data from iceoryx2 to Zenoh
/// - `inbound_stream`: Transfers data from Zenoh to iceoryx2
///
/// It creates a complete bidirectional communication channel between the two middleware systems,
/// allowing services defined in iceoryx2 to be accessible through Zenoh and vice versa.
struct BidirectionalRelay<'a> {
    outbound_stream: OutboundStream<'a>,
    inbound_stream: InboundStream,
}

impl<'a> BidirectionalRelay<'a> {
    pub fn new(
        iox_service_details: &ServiceDetails<ipc::Service>,
        iox_node: &IceoryxNode<ipc::Service>,
        z_session: &ZenohSession,
    ) -> Self {
        let iox_service = iox_create_service(iox_node, iox_service_details);

        // Create Outbound Stream
        let iox_subscriber = iox_create_subscriber(&iox_service, iox_service_details);
        let z_publisher = zenoh_create_publisher(z_session, iox_service_details);
        let outbound_stream = OutboundStream::new(iox_subscriber, z_publisher);

        // Create Inbound Stream
        let iox_publisher = iox_create_publisher(&iox_service, iox_service_details);
        let z_subscriber = zenoh_create_subscriber(z_session, iox_service_details);
        let inbound_stream = InboundStream::new(iox_publisher, z_subscriber);

        Self {
            outbound_stream,
            inbound_stream,
        }
    }

    pub fn propagate(&self) {
        self.outbound_stream.propagate();
        self.inbound_stream.propagate();
    }
}

pub struct Tunnel<'a> {
    z_session: ZenohSession,
    iox_config: IceoryxConfig,
    iox_node: IceoryxNode<ipc::Service>,
    iox_tracker: IceoryxServiceTracker<ipc::Service>,
    relays: HashMap<IceoryxServiceId, BidirectionalRelay<'a>>,
}

impl<'a> Tunnel<'a> {
    pub fn new(iox_config: IceoryxConfig) -> Self {
        let mut z_config = zenoh::config::Config::default();
        z_config.insert_json5("adminspace/enabled", "true").unwrap(); // this is mandatory
        let z_session = zenoh::open(z_config).wait().unwrap();

        let iox_node = NodeBuilder::new()
            .config(&iox_config)
            .create::<ipc::Service>()
            .unwrap();
        let iox_tracker = IceoryxServiceTracker::new();

        let relays: HashMap<IceoryxServiceId, BidirectionalRelay> = HashMap::new();

        Self {
            z_session,
            iox_config,
            iox_node,
            iox_tracker,
            relays,
        }
    }

    pub fn initialize(&mut self) {
        info!("Zenoh Tunnel UP");
    }

    pub fn discover(&mut self) {
        self.iox_discovery();
    }

    pub fn propagate(&self) {
        for (_, relay) in &self.relays {
            relay.propagate();
        }
    }

    pub fn shutdown(&mut self) {
        info!("Zenoh Tunnel DOWN");
    }

    /// Returns a list of all service IDs that are currently being tunneled.
    ///
    /// This method provides a way to inspect which iceoryx2 services are currently
    /// being bridged to the Zenoh network. It returns the string representation
    /// of each service ID that has an active relay.
    ///
    /// # Returns
    ///
    /// A vector of strings containing the service IDs of all tunneled services.
    pub fn tunneled_services(&self) -> Vec<String> {
        self.relays
            .iter()
            .map(|(id, _)| id.as_str().to_string())
            .collect()
    }

    /// Discovers iceoryx2 services and creates corresponding streams to propagate data to zenoh.
    ///
    /// This method synchronizes with the iceoryx2 service tracker to find newly added services.
    /// For each discovered publish-subscribe service, it creates an iceoryx2 subscriber and a
    /// corresponding Zenoh publisher, then announces the service to the Zenoh network.
    ///
    /// The discovered services are stored in the internal streams collection for later propagation.
    fn iox_discovery(&mut self) {
        let (added, _removed) = self.iox_tracker.sync(&self.iox_config).unwrap();

        for iox_service_id in added {
            let iox_service_details = self.iox_tracker.get(&iox_service_id).unwrap();

            if let MessagingPattern::PublishSubscribe(_) =
                iox_service_details.static_details.messaging_pattern()
            {
                info!(
                    "DISCOVERED (iceoryx2): {} [{}]",
                    iox_service_details.static_details.service_id().as_str(),
                    iox_service_details.static_details.name()
                );

                if !self.relays.contains_key(&iox_service_id) {
                    // Set up relay
                    self.relays.insert(
                        iox_service_id.clone(),
                        BidirectionalRelay::new(
                            &iox_service_details,
                            &self.iox_node,
                            &self.z_session,
                        ),
                    );

                    // Announce Service to Zenoh
                    zenoh_announce_service(&self.z_session, iox_service_details);
                }
            }
        }
    }
}

fn iox_create_service(
    iox_node: &IceoryxNode<ipc::Service>,
    iox_service_details: &ServiceDetails<ipc::Service>,
) -> PortFactory<IceoryxService, [CustomPayloadMarker], CustomHeaderMarker> {
    let iox_service = unsafe {
        iox_node
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
            .unwrap()
    };

    iox_service
}

fn iox_create_publisher(
    iox_service: &PortFactory<IceoryxService, [CustomPayloadMarker], CustomHeaderMarker>,
    iox_service_details: &ServiceDetails<ipc::Service>,
) -> IceoryxPublisher<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker> {
    let iox_publisher = iox_service
        .publisher_builder()
        .allocation_strategy(AllocationStrategy::PowerOfTwo)
        .create()
        .unwrap();
    info!(
        "NEW PUBLISHER (iceoryx2): {} [{}]",
        iox_service_details.static_details.service_id().as_str(),
        iox_service_details.static_details.name()
    );

    iox_publisher
}

/// Creates an iceoryx2 subscriber for a given service.
///
/// This method creates a subscriber for the specified iceoryx2 service using custom payload
/// and header markers. It sets up the necessary type details from the service details.
///
/// # Arguments
///
/// * `iox_node` - The iceoryx2 node used to build the service
/// * `iox_service_details` - The details of the service to subscribe to
///
/// # Returns
///
/// An iceoryx2 subscriber configured with custom payload and header markers
fn iox_create_subscriber(
    iox_service: &PortFactory<IceoryxService, [CustomPayloadMarker], CustomHeaderMarker>,
    iox_service_details: &ServiceDetails<ipc::Service>,
) -> IceoryxSubscriber<ipc::Service, [CustomPayloadMarker], CustomHeaderMarker> {
    let iox_subscriber = iox_service.subscriber_builder().create().unwrap();
    info!(
        "NEW SUBSCRIBER (iceoryx2): {} [{}]",
        iox_service_details.static_details.service_id().as_str(),
        iox_service_details.static_details.name()
    );

    iox_subscriber
}

/// Creates a Zenoh publisher for an iceoryx2 service.
///
/// This method creates a Zenoh publisher at the key expression derived from the service ID.
/// The publisher is used to send data from iceoryx2 to the Zenoh network.
///
/// # Arguments
///
/// * `z_session` - The Zenoh session used to declare the publisher
/// * `iox_service_details` - The iceoryx2 service details containing the service ID
///
/// # Returns
///
/// A Zenoh publisher that can be used to publish data to the Zenoh network
fn zenoh_create_publisher<'a>(
    z_session: &ZenohSession,
    iox_service_details: &ServiceDetails<ipc::Service>,
) -> ZenohPublisher<'a> {
    let z_key = keys::data_stream(iox_service_details.static_details.service_id());
    info!("NEW PUBLISHER (zenoh): {}", z_key.clone());
    let z_publisher = z_session.declare_publisher(z_key).wait().unwrap();

    z_publisher
}

fn zenoh_create_subscriber(
    z_session: &ZenohSession,
    iox_service_details: &ServiceDetails<ipc::Service>,
) -> ZenohSubscriber<FifoChannelHandler<Sample>> {
    let z_key = keys::data_stream(iox_service_details.static_details.service_id());
    let z_subscriber = z_session
        .declare_subscriber(z_key)
        .with(FifoChannel::new(10))
        .allowed_origin(Locality::Remote)
        .wait()
        .unwrap();

    z_subscriber
}

/// Announces an iceoryx2 service to the Zenoh network.
///
/// This method creates a Zenoh queryable at the service's key expression that responds
/// with the service's static details serialized as JSON. This allows other components
/// to discover the service through Zenoh queries.
///
/// # Arguments
///
/// * `z_session` - The Zenoh session used to declare the queryable
/// * `iox_service_details` - The iceoryx2 service details to be announced
fn zenoh_announce_service(
    z_session: &ZenohSession,
    iox_service_details: &ServiceDetails<ipc::Service>,
) {
    let z_key = keys::service(iox_service_details.static_details.service_id());
    let iox_static_details_json =
        serde_json::to_string(&iox_service_details.static_details).unwrap();
    z_session
        .declare_queryable(z_key.clone())
        .callback(move |query| {
            query
                .reply(query.key_expr().clone(), &iox_static_details_json)
                .wait()
                .unwrap();
        })
        .background()
        .wait()
        .unwrap();

    info!("ANNOUNCING (zenoh): {}", z_key);
}
