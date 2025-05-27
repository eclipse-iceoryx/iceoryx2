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

use crate::z_announce_service;
use crate::z_query_services;
use crate::BidirectionalStream;
use crate::DataStream;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::prelude::*;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::info;
use iceoryx2_services_discovery::service_discovery::Discovery;
use iceoryx2_services_discovery::service_discovery::Tracker as IceoryxServiceTracker;

use zenoh::Config as ZenohConfig;
use zenoh::Session as ZenohSession;
use zenoh::Wait;

use std::collections::HashMap;

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
    FailureToDiscoverServicesLocally,
    FailureToDiscoverServicesRemotely,
    FailureToAnnounceServiceRemotely,
    FailureToEstablishDataStream,
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
    streams: HashMap<IceoryxServiceId, BidirectionalStream<'a, Service>>,
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

                info!("CONFIGURED Discovery updates from service {}", value);
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
                info!("CONFIGURED Internal discovery tracking");
                (None, Some(IceoryxServiceTracker::new()))
            }
        };

        let streams: HashMap<IceoryxServiceId, BidirectionalStream<Service>> = HashMap::new();

        Ok(Self {
            z_session,
            iox_config: iox_config.clone(),
            iox_node,
            iox_discovery_service,
            iox_tracker,
            streams,
        })
    }

    /// Discover services locally and remotely.
    pub fn discover(&mut self) -> Result<(), DiscoveryError> {
        self.local_discovery()?;
        self.remote_discovery()?;

        return Ok(());
    }

    /// Propagates payloads between iceoryx2 and zenoh.
    pub fn propagate(&self) {
        for (id, stream) in &self.streams {
            if let Err(e) = stream.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }
    }

    /// Returns a list of all service IDs that are currently being tunneled.
    pub fn tunneled_services(&self) -> Vec<String> {
        self.streams
            .iter()
            .map(|(id, _)| id.as_str().to_string())
            .collect()
    }

    /// Discovers local services via iceoryx2.
    fn local_discovery(&mut self) -> Result<(), DiscoveryError> {
        let mut on_discovered =
            |iox_service_config: &IceoryxServiceConfig| -> Result<(), DiscoveryError> {
                let iox_service_id = iox_service_config.service_id();

                if !self.streams.contains_key(&iox_service_id) {
                    info!(
                        "DISCOVERED (iceoryx2): {} [{}]",
                        iox_service_id.as_str(),
                        iox_service_config.name()
                    );

                    // Set up stream
                    let stream = BidirectionalStream::create(
                        &self.iox_node,
                        &self.z_session,
                        iox_service_config,
                    )
                    .map_err(|_e| DiscoveryError::FailureToEstablishDataStream)?;
                    self.streams.insert(iox_service_id.clone(), stream);

                    // Announce Service to Zenoh
                    z_announce_service(&self.z_session, iox_service_config)
                        .map_err(|_e| DiscoveryError::FailureToAnnounceServiceRemotely)?;
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
                        return Err(DiscoveryError::FailureToDiscoverServicesLocally);
                    }
                }
            }
        } else if let Some(iox_tracker) = &mut self.iox_tracker {
            // Discovery via service tracker
            let (added, _removed) = iox_tracker
                .sync(&self.iox_config)
                .map_err(|_e| DiscoveryError::FailureToDiscoverServicesLocally)?;

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
            panic!("Unable to discover iceoryx2 services as neither the service discovery service nor a service tracker are set up.");
        }

        Ok(())
    }

    /// Discovers remote services via zenoh.
    fn remote_discovery(&mut self) -> Result<(), DiscoveryError> {
        // TODO(optimize): This is re-discoverying the same services every iteration
        let iox_service_configs = z_query_services(&self.z_session)
            .map_err(|_e| DiscoveryError::FailureToDiscoverServicesRemotely)?;

        for iox_service_config in iox_service_configs {
            let iox_service_id = iox_service_config.service_id();
            if !self.streams.contains_key(&iox_service_id) {
                info!(
                    "DISCOVERED (zenoh): {} [{}]",
                    iox_service_id.as_str(),
                    iox_service_config.name()
                );

                let stream = BidirectionalStream::create(
                    &self.iox_node,
                    &self.z_session,
                    &iox_service_config,
                )
                .map_err(|_e| DiscoveryError::FailureToEstablishDataStream)?;

                self.streams.insert(iox_service_id.clone(), stream);
            }
        }

        Ok(())
    }
}
