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

use crate::channel::Channel;
use crate::channel::ListenerChannel;
use crate::channel::NotifierChannel;
use crate::channel::PropagationError;
use crate::channel::PublisherChannel;
use crate::channel::SubscriberChannel;
use crate::discovery::Discovery;
use crate::discovery::DiscoveryError;
use crate::discovery::IceoryxDiscovery;
use crate::discovery::ZenohDiscovery;
use crate::middleware;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::node::NodeBuilder;
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::static_config::StaticConfig as ServiceConfig;
use iceoryx2_bb_log::error;
use iceoryx2_bb_log::fail;

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

/// Defines the operational scope for tunnel services.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Scope {
    Iceoryx,
    Zenoh,
    Both,
}

/// Represents information about an active communication channel in the tunnel.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ChannelInfo {
    Publisher(String),
    Subscriber(String),
    Notifier(String),
    Listener(String),
}

/// A tunnel for propagating iceoryx2 payloads across hosts via the Zenoh network middleware.
#[derive(Debug)]
pub struct Tunnel<'a, ServiceType: iceoryx2::service::Service> {
    z_session: ZenohSession,
    z_discovery: ZenohDiscovery<'a, ServiceType>,
    iox_node: IceoryxNode<ServiceType>,
    iox_discovery: IceoryxDiscovery<ServiceType>,
    publisher_channels: HashMap<ServiceId, PublisherChannel<'a, ServiceType>>,
    subscriber_channels: HashMap<ServiceId, SubscriberChannel<ServiceType>>,
    notifier_channels: HashMap<ServiceId, NotifierChannel<'a, ServiceType>>,
    listener_channels: HashMap<ServiceId, ListenerChannel<ServiceType>>,
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
        let z_session = zenoh::open(z_config.clone()).wait();
        let z_session = fail!(
            from "Tunnel::create()",
            when z_session,
            with CreationError::Error,
            "failed to open zenoh session"
        );
        let z_discovery = ZenohDiscovery::create(&z_session);
        let z_discovery = fail!(
            from "Tunnel::create()",
            when z_discovery,
            with CreationError::Error,
            "failed to create zenoh discovery"
        );

        let iox_node = NodeBuilder::new().config(iox_config).create::<Service>();
        let iox_node = fail!(
            from "Tunnel::create()",
            when iox_node,
            with CreationError::Error,
            "failed to create node"
        );

        let iox_discovery =
            IceoryxDiscovery::create(iox_config, &iox_node, &tunnel_config.discovery_service);
        let iox_discovery = fail!(
            from "Tunnel::create()",
            when iox_discovery,
            with CreationError::Error,
            "failed to create iceoryx discovery"
        );

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
            fail!(
                from &self,
                when self.iox_discovery.discover(&mut |iox_service_config| {
                    on_discovery(
                        iox_service_config,
                        &self.iox_node,
                        &self.z_session,
                        &mut self.publisher_channels,
                        &mut self.subscriber_channels,
                        &mut self.notifier_channels,
                        &mut self.listener_channels,
                    )
                }),
                "failed to discover services via iceoryx"
            );
        }

        if scope == Scope::Zenoh || scope == Scope::Both {
            fail!(
                from &self,
                when self.z_discovery.discover(&mut |iox_service_config| {
                    on_discovery(
                        iox_service_config,
                        &self.iox_node,
                        &self.z_session,
                        &mut self.publisher_channels,
                        &mut self.subscriber_channels,
                        &mut self.notifier_channels,
                        &mut self.listener_channels,
                    )
                }),
                "failed to discover services via zenoh"
            );
        }

        Ok(())
    }

    /// Propagates payloads between all connected hosts.
    pub fn propagate(&self) -> Result<(), PropagationError> {
        // Attempted to propagate all channels. Continue to next channel if error encountered.
        let mut propagation_failure = false;
        for (id, channel) in &self.subscriber_channels {
            let _ = channel.propagate().inspect_err(|e| {
                error!("Failed to propagate data through subscriber channel with id {id:?}: {e}");
                propagation_failure = true;
            });
        }
        for (id, channel) in &self.publisher_channels {
            let _ = channel.propagate().inspect_err(|e| {
                error!("Failed to propagate data through publisher channel with id {id:?}: {e}");
                propagation_failure = true;
            });
        }
        for (id, channel) in &self.notifier_channels {
            let _ = channel.propagate().inspect_err(|e| {
                error!("Failed to propagate data through notifier channel with id {id:?}: {e}");
                propagation_failure = true;
            });
        }
        for (id, channel) in &self.listener_channels {
            let _ = channel.propagate().inspect_err(|e| {
                error!("Failed to propagate data through listener channel with id {id:?}: {e}");
                propagation_failure = true;
            });
        }

        if propagation_failure {
            fail!(from self,
                with PropagationError::Incomplete,
                "failure to propagate over all channels");
        }

        Ok(())
    }

    /// Returns all currently active channels in the tunnel.
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

// TODO(correctness): Proper clean-up in error cases

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
    iox_service_config: &ServiceConfig,
    iox_node: &IceoryxNode<ServiceType>,
    z_session: &ZenohSession,
    publisher_channels: &mut HashMap<ServiceId, PublisherChannel<'a, ServiceType>>,
    subscriber_channels: &mut HashMap<ServiceId, SubscriberChannel<ServiceType>>,
    notifier_channels: &mut HashMap<ServiceId, NotifierChannel<'a, ServiceType>>,
    listener_channels: &mut HashMap<ServiceId, ListenerChannel<ServiceType>>,
) -> Result<(), DiscoveryError> {
    match iox_service_config.messaging_pattern() {
        MessagingPattern::PublishSubscribe(_) => {
            fail!(
                from "on_discovery()",
                when on_publish_subscribe_service(
                    iox_node,
                    iox_service_config,
                    z_session,
                    publisher_channels,
                    subscriber_channels,
                ),
                "failed to process discovered publish-subscribe service"
            );
        }
        MessagingPattern::Event(_) => {
            fail!(
                from "on_discovery()",
                when on_event_service(
                    iox_node,
                    iox_service_config,
                    z_session,
                    notifier_channels,
                    listener_channels,
                ),
                "failed to process discovered event service"
            );
        }
        _ => { /* Not supported. Nothing to do. */ }
    }

    Ok(())
}

/// Handles the publish-subscribe messaging pattern during service discovery.
fn on_publish_subscribe_service<'a, ServiceType: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<ServiceType>,
    iox_service_config: &ServiceConfig,
    z_session: &ZenohSession,
    publisher_channels: &mut HashMap<ServiceId, PublisherChannel<'a, ServiceType>>,
    subscriber_channels: &mut HashMap<ServiceId, SubscriberChannel<ServiceType>>,
) -> Result<(), DiscoveryError> {
    let iox_service_id = iox_service_config.service_id();
    let needs_publisher = !publisher_channels.contains_key(iox_service_id);
    let needs_subscriber = !subscriber_channels.contains_key(iox_service_id);

    if needs_publisher || needs_subscriber {
        let iox_service = fail!(
            from "on_publish_subscribe_service()",
            when middleware::iceoryx::create_publish_subscribe_service::<ServiceType>(
                    iox_node,
                    iox_service_config,
                ),
            with DiscoveryError::ServiceCreation,
            "failed to open or create discovered publish-subscribe service"
        );

        if needs_publisher {
            let publisher_channel = fail!(
                from "on_publish_subscribe_service()",
                when PublisherChannel::create(
                    iox_node.id(),
                    iox_service_config,
                    &iox_service,
                    z_session,
                ),
                with DiscoveryError::PortCreation,
                "failed to create publisher channel for discovered service"
            );

            publisher_channels.insert(iox_service_id.clone(), publisher_channel);
        }
        if needs_subscriber {
            let subscriber_channel = fail!(
                from "on_publish_subscribe_service()",
                when SubscriberChannel::create(
                    iox_service_config, &iox_service, z_session
                ),
                with DiscoveryError::PortCreation,
                "failed to create subscriber channel for discovered service"
            );

            subscriber_channels.insert(iox_service_id.clone(), subscriber_channel);
        }

        fail!(
            from "on_publish_subscribe_service()",
            when middleware::zenoh::announce_service(z_session, iox_service_config),
            with DiscoveryError::ServiceAnnouncement,
            "failed to announce discovered publish-subscribe service to zenoh network"
        );
    }

    Ok(())
}

/// Handles the event messaging pattern during service discovery.
fn on_event_service<'a, ServiceType: iceoryx2::service::Service>(
    iox_node: &IceoryxNode<ServiceType>,
    iox_service_config: &ServiceConfig,
    z_session: &ZenohSession,
    notifier_channels: &mut HashMap<ServiceId, NotifierChannel<'a, ServiceType>>,
    listener_channels: &mut HashMap<ServiceId, ListenerChannel<ServiceType>>,
) -> Result<(), DiscoveryError> {
    let iox_service_id = iox_service_config.service_id();
    let needs_notifier = !notifier_channels.contains_key(iox_service_id);
    let needs_listener = !listener_channels.contains_key(iox_service_id);

    if needs_notifier || needs_listener {
        let iox_service = fail!(
            from "on_event_service()",
            when middleware::iceoryx::create_event_service::<ServiceType>(iox_node, iox_service_config),
            with DiscoveryError::ServiceCreation,
            "failed to open or create discovered event service"
        );
        if needs_notifier {
            let notifier_channel = fail!(
                from "on_event_service()",
                when NotifierChannel::create(iox_service_config, &iox_service, z_session),
                with DiscoveryError::PortCreation,
                "failed to create notifier channel for discovered service"
            );
            notifier_channels.insert(iox_service_id.clone(), notifier_channel);
        }
        if needs_listener {
            let listener_channel = fail!(
                from "on_event_service()",
                when ListenerChannel::create(iox_service_config, &iox_service, z_session),
                with DiscoveryError::PortCreation,
                "failed to create listener channel for discovered service"
            );
            listener_channels.insert(iox_service_id.clone(), listener_channel);
        }
        fail!(
            from "on_event_service()",
            when middleware::zenoh::announce_service(z_session, iox_service_config),
            with DiscoveryError::ServiceAnnouncement,
            "failed to announce discovered event service to zenoh network"
        );
    }

    Ok(())
}
