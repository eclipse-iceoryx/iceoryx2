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

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::prelude::*;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2_bb_log::info;
use iceoryx2_services_discovery::service_discovery::Tracker as IceoryxServiceTracker;

use zenoh::pubsub::Publisher as ZenohPublisher;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use std::collections::HashMap;

pub struct Tunnel<'a> {
    z_session: ZenohSession,
    iox_config: IceoryxConfig,
    iox_node: IceoryxNode<ipc::Service>,
    iox_tracker: IceoryxServiceTracker<ipc::Service>,
    streams: HashMap<IceoryxServiceId, DataStream<'a>>,
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

        let streams: HashMap<IceoryxServiceId, DataStream<'a>> = HashMap::new();

        Self {
            z_session,
            iox_config,
            iox_node,
            iox_tracker,
            streams,
        }
    }

    pub fn initialize(&mut self) {
        info!("Zenoh Tunnel UP");
    }

    pub fn discover(&mut self) {
        self.iox_discovery();
    }

    pub fn propagate(&self) {
        for (_id, stream) in &self.streams {
            match stream {
                DataStream::Outbound {
                    iox_service_id,
                    iox_subscriber: _,
                    z_publisher: _,
                } => {
                    info!("PROPAGATING (outbound): {}", iox_service_id.as_str());
                }
            }
            stream.propagate();
        }
    }

    pub fn shutdown(&mut self) {
        info!("Zenoh Tunnel DOWN");
    }

    pub fn stream_ids(&self) -> Vec<String> {
        self.streams
            .iter()
            .map(|(id, _)| id.as_str().to_string())
            .collect()
    }

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

                if !self.streams.contains_key(&iox_service_id) {
                    let iox_subscriber =
                        Self::iox_create_subscriber(&self.iox_node, iox_service_details);
                    let z_publisher =
                        Self::zenoh_create_publisher(&self.z_session, iox_service_details);

                    Self::zenoh_announce_service(&self.z_session, iox_service_details);

                    self.streams.insert(
                        iox_service_id.clone(),
                        DataStream::new_outbound(&iox_service_id, iox_subscriber, z_publisher),
                    );
                } else {
                    // TODO
                }
            }
        }
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
                "NEW SUBSCRIBER (iceoryx2): {} [{}]",
                iox_service_details.static_details.service_id().as_str(),
                iox_service_details.static_details.name()
            );

            iox_subscriber
        }
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
    fn zenoh_create_publisher(
        z_session: &ZenohSession,
        iox_service_details: &ServiceDetails<ipc::Service>,
    ) -> ZenohPublisher<'a> {
        let z_key = keys::data_stream(iox_service_details.static_details.service_id());
        let z_publisher = z_session.declare_publisher(z_key.clone()).wait().unwrap();
        info!("NEW PUBLISHER (zenoh): {}", z_key);

        z_publisher
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
}
