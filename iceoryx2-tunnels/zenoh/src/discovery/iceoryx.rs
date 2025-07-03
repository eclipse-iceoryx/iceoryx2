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
use crate::discovery::DiscoveryError;

use iceoryx2::config::Config;
use iceoryx2::node::Node;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2::service::Service;
use iceoryx2_bb_log::fail;
use iceoryx2_bb_log::info;
use iceoryx2_services_discovery::service_discovery::Discovery as DiscoveryUpdate;
use iceoryx2_services_discovery::service_discovery::Tracker;

// TODO: More granularity in errors
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

#[derive(Debug)]
pub(crate) struct IceoryxDiscovery<ServiceType: iceoryx2::service::Service> {
    config: Config,
    discovery_subscriber: Option<Subscriber<ServiceType, DiscoveryUpdate, ()>>,
    discovery_tracker: Option<Tracker<ServiceType>>,
}

impl<ServiceType: iceoryx2::service::Service> IceoryxDiscovery<ServiceType> {
    pub fn create(
        config: &Config,
        node: &Node<ServiceType>,
        service_name: &Option<String>,
    ) -> Result<Self, CreationError> {
        let (discovery_service, discovery_tracker) = match service_name {
            Some(service_name) => {
                let service_name = fail!(
                    from "IceoryxDiscovery::create()",
                    when service_name.as_str().try_into(),
                    with CreationError::Error,
                    "failed to create service name for discovery service"
                );

                let service = fail!(
                    from "IceoryxDiscovery::create()",
                    when node.service_builder(&service_name)
                            .publish_subscribe::<DiscoveryUpdate>()
                            .open_or_create(),
                    with CreationError::Error,
                    "failed to open or create iceoryx discovery service"
                );
                let discovery_subscriber = fail!(
                    from "IceoryxDiscovery::create()",
                    when service.subscriber_builder().create(),
                    with CreationError::Error,
                    "failed to create subscriber to iceoryx discovery service"
                );

                info!("CONFIGURE DiscoveryService {}", service_name);
                (Some(discovery_subscriber), None)
            }
            None => {
                info!("CONFIGURE DiscoveryTracker");
                (None, Some(Tracker::<ServiceType>::new()))
            }
        };

        Ok(Self {
            config: config.clone(),
            discovery_subscriber: discovery_service,
            discovery_tracker,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Discovery<ServiceType>
    for IceoryxDiscovery<ServiceType>
{
    fn discover<
        OnDiscovered: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), DiscoveryError>,
    >(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), DiscoveryError> {
        match (&self.discovery_subscriber, &mut self.discovery_tracker) {
            (Some(subscriber), _) => discover_via_subscriber(subscriber, on_discovered),
            (_, Some(tracker)) => discover_via_tracker(&self.config, tracker, on_discovered),
            (None, None) => panic!("Unable to discover iceoryx services as neither the service discovery service nor a service tracker are set up"),
        }
    }
}

fn discover_via_subscriber<
    ServiceType: Service,
    OnDiscovered: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), DiscoveryError>,
>(
    subscriber: &Subscriber<ServiceType, DiscoveryUpdate, ()>,
    on_discovered: &mut OnDiscovered,
) -> Result<(), DiscoveryError> {
    loop {
        match subscriber.receive() {
            Ok(Some(sample)) => {
                if let DiscoveryUpdate::Added(service_config) = sample.payload() {
                    match service_config.messaging_pattern() {
                        MessagingPattern::PublishSubscribe(_) | MessagingPattern::Event(_) => {
                            fail!(
                                from "discovery_via_subscriber()",
                                when on_discovered(service_config),
                                "failed to process service discovered via subscriber to discovery service"
                            );
                        }
                        _ => {
                            // Not supported. Nothing to do.
                        }
                    }
                }
            }
            Ok(None) => break,
            Err(e) => fail!(
                from "discovery_via_subscriber()",
                when Err(e),
                with DiscoveryError::UpdateFromLocalPort,
                "failed to receive from discovery subscriber"
            ),
        }
    }

    Ok(())
}

fn discover_via_tracker<
    ServiceType: Service,
    OnDiscovered: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), DiscoveryError>,
>(
    config: &Config,
    tracker: &mut Tracker<ServiceType>,
    on_discovered: &mut OnDiscovered,
) -> Result<(), DiscoveryError> {
    let (added, _removed) = fail!(
        from "discovery_via_tracker()",
        when tracker.sync(config),
        with DiscoveryError::UpdateFromTracker,
        "failed to synchronize with service tracker"
    );

    for service_id in added {
        if let Some(service_details) = tracker.get(&service_id) {
            let service_config = &service_details.static_details;
            match service_config.messaging_pattern() {
                MessagingPattern::PublishSubscribe(_) | MessagingPattern::Event(_) => {
                    fail!(
                        from "discovery_via_tracker()",
                        when on_discovered(service_config),
                        "failed to process service discovered via tracker"
                    );
                }
                _ => {
                    // Not supported. Nothing to do.
                }
            }
        }
    }

    Ok(())
}
