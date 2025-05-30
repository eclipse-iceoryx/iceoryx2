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

use crate::discovery::Discovery;
use crate::discovery::ZenohDiscovery;
use crate::BidirectionalEventConnection;
use crate::BidirectionalPublishSubscribeConnection;
use crate::Connection;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::prelude::*;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::info;
use iceoryx2_services_discovery::service_discovery::Discovery as IceoryxDiscovery;
use iceoryx2_services_discovery::service_discovery::Tracker as IceoryxServiceTracker;

use zenoh::Config as ZenohConfig;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use std::collections::HashMap;

#[derive(Default)]
pub struct TunnelConfig {
    pub discovery_service: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailureToCreateZenohSession,
    FailureToCreateZenohDiscovery,
    FailureToCreateZenohQuery,
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
    FailureToDiscoverServicesLocally,
    FailureToDiscoverServicesRemotely,
    FailureToAnnounceServiceRemotely,
    FailureToEstablishConnection,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "DiscoveryError::{:?}", self)
    }
}

impl core::error::Error for DiscoveryError {}

/// A tunnel for propagating iceoryx2 payloads across hosts via the Zenoh network middleware.
pub struct Tunnel<'a, Service: iceoryx2::service::Service> {
    z_session: ZenohSession,
    z_discovery: ZenohDiscovery<'a, Service>,
    iox_config: IceoryxConfig,
    iox_node: IceoryxNode<Service>,
    iox_discovery_subscriber: Option<IceoryxSubscriber<Service, IceoryxDiscovery, ()>>,
    iox_discovery_tracker: Option<IceoryxServiceTracker<Service>>,
    publish_subscribe_connectons:
        HashMap<IceoryxServiceId, BidirectionalPublishSubscribeConnection<'a, Service>>,
    event_connections: HashMap<IceoryxServiceId, BidirectionalEventConnection<'a, Service>>,
}

impl<'a, Service: iceoryx2::service::Service> Tunnel<'a, Service> {
    /// Creates a new tunnel with the provided configuration.
    pub fn create(
        tunnel_config: &TunnelConfig,
        iox_config: &IceoryxConfig,
        z_config: &ZenohConfig,
    ) -> Result<Self, CreationError> {
        info!("STARTING Zenoh Tunnel");

        let z_session = zenoh::open(z_config.clone())
            .wait()
            .map_err(|_e| CreationError::FailureToCreateZenohSession)?;
        let z_discovery = ZenohDiscovery::create(&z_session)
            .map_err(|_e| CreationError::FailureToCreateZenohDiscovery)?;

        let iox_node = NodeBuilder::new()
            .config(iox_config)
            .create::<Service>()
            .map_err(|_e| CreationError::FailureToCreateZenohQuery)?;

        // Create either a discovery service subscriber or a service tracker based on configuration
        let (iox_discovery_subscriber, iox_discovery_tracker) =
            match &tunnel_config.discovery_service {
                Some(value) => {
                    let iox_service_name: ServiceName = value
                        .as_str()
                        .try_into()
                        .map_err(|_e| CreationError::InvalidNameForDiscoveryService)?;

                    info!("CONFIGURED Discovery updates from service {}", value);
                    let iox_service = iox_node
                        .service_builder(&iox_service_name)
                        .publish_subscribe::<IceoryxDiscovery>()
                        .open_or_create()
                        .map_err(|_e| CreationError::FailureToCreateIceoryxService)?;

                    let iox_subscriber = iox_service
                        .subscriber_builder()
                        .create()
                        .map_err(|_e| CreationError::FailureToCreateIceoryxSubscriber)?;

                    (Some(iox_subscriber), None)
                }
                None => {
                    info!("CONFIGURED Internal discovery tracking");
                    (None, Some(IceoryxServiceTracker::new()))
                }
            };

        let publish_subscribe_connectons: HashMap<
            IceoryxServiceId,
            BidirectionalPublishSubscribeConnection<Service>,
        > = HashMap::new();
        let event_connections: HashMap<IceoryxServiceId, BidirectionalEventConnection<Service>> =
            HashMap::new();

        Ok(Self {
            z_session,
            z_discovery,
            iox_config: iox_config.clone(),
            iox_node,
            iox_discovery_subscriber,
            iox_discovery_tracker,
            publish_subscribe_connectons,
            event_connections,
        })
    }

    /// Discover services locally and remotely.
    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        self.local_discovery()?;
        self.remote_discovery()?;

        Ok(())
    }

    /// Propagates payloads between iceoryx2 and zenoh.
    pub fn propagate(&self) {
        for (id, connection) in &self.publish_subscribe_connectons {
            if let Err(e) = connection.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }

        for (id, connection) in &self.event_connections {
            if let Err(e) = connection.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }
    }

    /// Returns a list of all service IDs that are currently being tunneled.
    pub fn tunneled_services(&self) -> Vec<String> {
        self.publish_subscribe_connectons
            .keys()
            .chain(self.event_connections.keys())
            .map(|id| id.as_str().to_string())
            .collect()
    }

    /// Discover local services via iceoryx2.
    fn local_discovery(&mut self) -> Result<(), DiscoveryError> {
        // TODO(correctness): Handle event services - need to open corresponding service with same
        //                    properties
        let mut on_pub_sub =
            |iox_service_config: &IceoryxServiceConfig| -> Result<(), DiscoveryError> {
                let iox_service_id = iox_service_config.service_id();

                if !self
                    .publish_subscribe_connectons
                    .contains_key(iox_service_id)
                {
                    info!(
                        "DISCOVERED (iceoryx2): PublishSubscribe {} [{}]",
                        iox_service_id.as_str(),
                        iox_service_config.name()
                    );

                    let connection = BidirectionalPublishSubscribeConnection::create(
                        &self.iox_node,
                        &self.z_session,
                        iox_service_config,
                    )
                    .map_err(|_e| DiscoveryError::FailureToEstablishConnection)?;

                    self.publish_subscribe_connectons
                        .insert(iox_service_id.clone(), connection);
                }
                Ok(())
            };
        let mut on_event =
            |iox_service_config: &IceoryxServiceConfig| -> Result<(), DiscoveryError> {
                let iox_service_id = iox_service_config.service_id();
                if !self.event_connections.contains_key(iox_service_id) {
                    info!(
                        "DISCOVERED (iceoryx2): Event {} [{}]",
                        iox_service_id.as_str(),
                        iox_service_config.name()
                    );

                    let connection = BidirectionalEventConnection::create(
                        &self.iox_node,
                        &self.z_session,
                        iox_service_config,
                    )
                    .map_err(|_e| DiscoveryError::FailureToEstablishConnection)?;

                    self.event_connections
                        .insert(iox_service_id.clone(), connection);
                }

                Ok(())
            };

        // Discovery via discovery service
        if let Some(iox_discovery_subscriber) = &self.iox_discovery_subscriber {
            loop {
                match iox_discovery_subscriber.receive() {
                    Ok(result) => match result {
                        Some(iox_sample) => {
                            if let IceoryxDiscovery::Added(iox_service_details) =
                                iox_sample.payload()
                            {
                                match iox_service_details.messaging_pattern() {
                                    MessagingPattern::RequestResponse(_) => todo!(),
                                    MessagingPattern::PublishSubscribe(_) => {
                                        on_pub_sub(iox_service_details)?;
                                    }
                                    MessagingPattern::Event(_) => {
                                        on_event(iox_service_details)?;
                                    }
                                    _ => todo!(),
                                }
                            }
                        }
                        None => break,
                    },
                    Err(_e) => {
                        return Err(DiscoveryError::FailureToDiscoverServicesLocally);
                    }
                }
            }
        }
        // OR Discovery via service tracker
        else if let Some(iox_discovery_tracker) = &mut self.iox_discovery_tracker {
            let (added, _removed) = iox_discovery_tracker
                .sync(&self.iox_config)
                .map_err(|_e| DiscoveryError::FailureToDiscoverServicesLocally)?;

            for iox_service_id in added {
                if let Some(iox_service_details) = iox_discovery_tracker.get(&iox_service_id) {
                    let iox_service_details = &iox_service_details.static_details;

                    match iox_service_details.messaging_pattern() {
                        MessagingPattern::RequestResponse(_) => todo!(),
                        MessagingPattern::PublishSubscribe(_) => {
                            on_pub_sub(iox_service_details)?;
                        }
                        MessagingPattern::Event(_) => {
                            on_event(iox_service_details)?;
                        }
                        _ => todo!(),
                    }
                }
            }
        } else {
            // Should never happen
            panic!("Unable to discover iceoryx2 services as neither the service discovery service nor a service tracker are set up.");
        }

        Ok(())
    }

    /// Discover remote services via Zenoh.
    fn remote_discovery(&mut self) -> Result<(), DiscoveryError> {
        let mut on_discovered = |iox_service_config: &IceoryxServiceConfig| {
            let iox_service_id = iox_service_config.service_id();
            match iox_service_config.messaging_pattern() {
                MessagingPattern::PublishSubscribe(_) => {
                    if !self
                        .publish_subscribe_connectons
                        .contains_key(iox_service_id)
                    {
                        info!(
                            "DISCOVERED (zenoh): PublishSubscribe {} [{}]",
                            iox_service_id.as_str(),
                            iox_service_config.name()
                        );

                        let connection = BidirectionalPublishSubscribeConnection::create(
                            &self.iox_node,
                            &self.z_session,
                            &iox_service_config,
                        )
                        .unwrap();

                        self.publish_subscribe_connectons
                            .insert(iox_service_id.clone(), connection);
                    }
                }
                MessagingPattern::Event(_) => {
                    if !self.event_connections.contains_key(iox_service_id) {
                        info!(
                            "DISCOVERED (zenoh): Event {} [{}]",
                            iox_service_id.as_str(),
                            iox_service_config.name()
                        );

                        let connection = BidirectionalEventConnection::create(
                            &self.iox_node,
                            &self.z_session,
                            &iox_service_config,
                        )
                        .unwrap();

                        self.event_connections
                            .insert(iox_service_id.clone(), connection);
                    }
                }
                _ => { /* Not supported. Nothing to do. */ }
            }
        };

        self.z_discovery.discover(&mut on_discovered).unwrap();

        Ok(())
    }
}
