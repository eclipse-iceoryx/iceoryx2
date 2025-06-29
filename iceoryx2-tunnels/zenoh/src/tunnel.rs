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
use crate::discovery::IceoryxDiscovery;
use crate::discovery::ZenohDiscovery;
use crate::middleware;
use crate::Channel;
use crate::ListenerChannel;
use crate::NotifierChannel;
use crate::PublisherChannel;
use crate::SubscriberChannel;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::node::NodeBuilder;
use iceoryx2::service::service_id::ServiceId as IceoryxServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig as IceoryxServiceConfig;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::info;

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
    Error,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    Error,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        core::write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

// TODO: Use this for propagation too
/// Defines the operational scope for tunnel services.
///
/// This enum specifies which environment to use for tunnel operations:
/// - `Iceoryx`: Only operate within the local Iceoryx environment
/// - `Zenoh`: Only operate through the Zenoh network
/// - `Both`: Operate in both Iceoryx and Zenoh environments
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Scope {
    Iceoryx,
    Zenoh,
    Both,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ChannelInfo {
    Publisher(String),
    Subscriber(String),
    Notifier(String),
    Listener(String),
}

/// A tunnel for propagating iceoryx2 payloads across hosts via the Zenoh network middleware.
pub struct Tunnel<'a, ServiceType: iceoryx2::service::Service> {
    z_session: ZenohSession,
    z_discovery: ZenohDiscovery<'a, ServiceType>,
    iox_node: IceoryxNode<ServiceType>,
    iox_discovery: IceoryxDiscovery<ServiceType>,
    publisher_channels: HashMap<IceoryxServiceId, PublisherChannel<'a, ServiceType>>,
    subscriber_channels: HashMap<IceoryxServiceId, SubscriberChannel<ServiceType>>,
    notifier_channels: HashMap<IceoryxServiceId, NotifierChannel<'a, ServiceType>>,
    listener_channels: HashMap<IceoryxServiceId, ListenerChannel<ServiceType>>,
}

impl<Service: iceoryx2::service::Service> Tunnel<'_, Service> {
    /// Creates a new tunnel with the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `tunnel_config` - Tunnel configuration
    /// * `iox_config` - Iceoryx configuration to be used
    /// * `z_config` - Zenoh configuration to be used
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - A new tunnel instance if creation was successful
    /// * `Err(CreationError)` - If any part of the tunnel creation failed
    pub fn create(
        tunnel_config: &TunnelConfig,
        iox_config: &IceoryxConfig,
        z_config: &ZenohConfig,
    ) -> Result<Self, CreationError> {
        info!("STARTING Zenoh Tunnel");

        let z_session = zenoh::open(z_config.clone())
            .wait()
            .map_err(|_e| CreationError::Error)?;
        let z_discovery = ZenohDiscovery::create(&z_session).map_err(|_e| CreationError::Error)?;

        let iox_node = NodeBuilder::new()
            .config(iox_config)
            .create::<Service>()
            .map_err(|_e| CreationError::Error)?;
        let iox_discovery =
            IceoryxDiscovery::create(iox_config, &iox_node, &tunnel_config.discovery_service)
                .map_err(|_e| CreationError::Error)?;

        Ok(Self {
            z_session,
            z_discovery,
            iox_node,
            iox_discovery,
            publisher_channels: HashMap::new(),
            subscriber_channels: HashMap::new(),
            notifier_channels: HashMap::new(),
            listener_channels: HashMap::new(),
        })
    }

    /// Discover iceoryx services across all connected hosts.
    ///
    /// # Arguments
    ///
    /// * `scope` - Determines the discovery scope
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If discovery was successful
    /// * `Err(DiscoveryError)` - If discovery failed
    pub fn discover(&mut self, scope: Scope) -> Result<(), DiscoveryError> {
        if scope == Scope::Iceoryx || scope == Scope::Both {
            self.iox_discovery
                .discover(&mut |iox_service_config| {
                    on_discovery(
                        iox_service_config,
                        &self.iox_node,
                        &self.z_session,
                        &mut self.publisher_channels,
                        &mut self.subscriber_channels,
                        &mut self.notifier_channels,
                        &mut self.listener_channels,
                    )
                })
                .map_err(|_e| DiscoveryError::Error)?;
        }

        if scope == Scope::Zenoh || scope == Scope::Both {
            self.z_discovery
                .discover(&mut |iox_service_config| {
                    on_discovery(
                        iox_service_config,
                        &self.iox_node,
                        &self.z_session,
                        &mut self.publisher_channels,
                        &mut self.subscriber_channels,
                        &mut self.notifier_channels,
                        &mut self.listener_channels,
                    )
                })
                .map_err(|_e| DiscoveryError::Error)?;
        }

        Ok(())
    }

    /// Propagates payloads between all connected hosts.
    pub fn propagate(&self) {
        // TODO(correctioness): consolidate and forward errors
        for (id, channel) in &self.subscriber_channels {
            if let Err(e) = channel.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }
        for (id, channel) in &self.publisher_channels {
            if let Err(e) = channel.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }
        for (id, channel) in &self.notifier_channels {
            if let Err(e) = channel.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }
        for (id, channel) in &self.listener_channels {
            if let Err(e) = channel.propagate() {
                error!("Failed to propagate ({:?}): {}", id, e);
            }
        }
    }

    /// Returns all currently open channels in the tunnel.
    pub fn active_channels(&self) -> Vec<ChannelInfo> {
        let mut ports = Vec::new();

        for id in self.publisher_channels.keys() {
            ports.push(ChannelInfo::Publisher(id.as_str().to_string()));
        }
        for id in self.subscriber_channels.keys() {
            ports.push(ChannelInfo::Subscriber(id.as_str().to_string()));
        }
        for id in self.notifier_channels.keys() {
            ports.push(ChannelInfo::Notifier(id.as_str().to_string()));
        }
        for id in self.listener_channels.keys() {
            ports.push(ChannelInfo::Listener(id.as_str().to_string()));
        }

        ports
    }
}

// TODO(correctness): Proper error handling and clean-up in error cases

/// Handles the discovery of a service and creates appropriate channels for it.
///
/// This function is called whenever a new service is discovered, either from the local Iceoryx
/// environment or from the Zenoh network. It creates the necessary channels based on the
/// messaging pattern of the discovered service.
///
/// # Arguments
///
/// * `iox_service_config` - Configuration of the discovered Iceoryx service
/// * `iox_node` - The local Iceoryx node to create services on
/// * `z_session` - The Zenoh session for network communication
/// * `publisher_channels` - Map of existing publisher channels, updated if a new one is created
/// * `subscriber_channels` - Map of existing subscriber channels, updated if a new one is created
/// * `notifier_channels` - Map of existing notifier channels, updated if a new one is created
/// * `listener_channels` - Map of existing listener channels, updated if a new one is created
fn on_discovery<'a, ServiceType: iceoryx2::service::Service>(
    iox_service_config: &IceoryxServiceConfig,
    iox_node: &IceoryxNode<ServiceType>,
    z_session: &ZenohSession,
    publisher_channels: &mut HashMap<IceoryxServiceId, PublisherChannel<'a, ServiceType>>,
    subscriber_channels: &mut HashMap<IceoryxServiceId, SubscriberChannel<ServiceType>>,
    notifier_channels: &mut HashMap<IceoryxServiceId, NotifierChannel<'a, ServiceType>>,
    listener_channels: &mut HashMap<IceoryxServiceId, ListenerChannel<ServiceType>>,
) {
    match iox_service_config.messaging_pattern() {
        MessagingPattern::PublishSubscribe(_) => {
            on_publish_subscribe_service(
                iox_node,
                iox_service_config,
                z_session,
                publisher_channels,
                subscriber_channels,
            );
        }
        MessagingPattern::Event(_) => {
            on_event_service(
                iox_node,
                iox_service_config,
                z_session,
                notifier_channels,
                listener_channels,
            );
        }
        _ => { /* Not supported. Nothing to do. */ }
    }
}

/// Handles the publish-subscribe messaging pattern during service discovery.
fn on_publish_subscribe_service<'a, ServiceType: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<ServiceType>,
    iox_service_config: &IceoryxServiceConfig,
    z_session: &ZenohSession,
    publisher_channels: &mut HashMap<IceoryxServiceId, PublisherChannel<'a, ServiceType>>,
    subscriber_channels: &mut HashMap<IceoryxServiceId, SubscriberChannel<ServiceType>>,
) {
    let iox_service_id = iox_service_config.service_id();
    let needs_publisher = !publisher_channels.contains_key(iox_service_id);
    let needs_subscriber = !subscriber_channels.contains_key(iox_service_id);

    if needs_publisher || needs_subscriber {
        let iox_service = middleware::iceoryx::create_publish_subscribe_service::<ServiceType>(
            iox_node,
            iox_service_config,
        )
        .map_err(|_e| CreationError::Error)
        .unwrap();

        if needs_publisher {
            let publisher_tunnel = PublisherChannel::create(
                iox_node.id(),
                iox_service_config,
                &iox_service,
                z_session,
            )
            .unwrap();
            publisher_channels.insert(iox_service_id.clone(), publisher_tunnel);

            info!(
                "CHANNEL: Publisher {} [{}]",
                iox_service_id.as_str(),
                iox_service_config.name()
            );
        }

        if needs_subscriber {
            let subscriber_tunnel =
                SubscriberChannel::create(iox_service_config, &iox_service, z_session).unwrap();
            subscriber_channels.insert(iox_service_id.clone(), subscriber_tunnel);

            info!(
                "CHANNEL: Subscriber {} [{}]",
                iox_service_id.as_str(),
                iox_service_config.name()
            );
        }

        middleware::zenoh::announce_service(z_session, iox_service_config)
            .map_err(|_e| CreationError::Error)
            .unwrap();
    }
}

/// Handles the event messaging pattern during service discovery.
fn on_event_service<'a, ServiceType: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<ServiceType>,
    iox_service_config: &IceoryxServiceConfig,
    z_session: &ZenohSession,
    notifier_channels: &mut HashMap<IceoryxServiceId, NotifierChannel<'a, ServiceType>>,
    listener_channels: &mut HashMap<IceoryxServiceId, ListenerChannel<ServiceType>>,
) {
    let iox_service_id = iox_service_config.service_id();
    let needs_notifier = !notifier_channels.contains_key(iox_service_id);
    let needs_listener = !listener_channels.contains_key(iox_service_id);

    if needs_notifier || needs_listener {
        let iox_service =
            middleware::iceoryx::create_event_service::<ServiceType>(iox_node, iox_service_config)
                .unwrap();

        if needs_notifier {
            let notifier_tunnel =
                NotifierChannel::create(iox_service_config, &iox_service, z_session).unwrap();
            notifier_channels.insert(iox_service_id.clone(), notifier_tunnel);

            info!(
                "CHANNEL: Notifier {} [{}]",
                iox_service_id.as_str(),
                iox_service_config.name()
            );
        }

        if needs_listener {
            let listener_tunnel =
                ListenerChannel::create(iox_service_config, &iox_service, z_session).unwrap();
            listener_channels.insert(iox_service_id.clone(), listener_tunnel);

            info!(
                "CHANNEL: Listener {} [{}]",
                iox_service_id.as_str(),
                iox_service_config.name()
            );
        }

        middleware::zenoh::announce_service(z_session, iox_service_config)
            .map_err(|_e| CreationError::Error)
            .unwrap();
    }
}
