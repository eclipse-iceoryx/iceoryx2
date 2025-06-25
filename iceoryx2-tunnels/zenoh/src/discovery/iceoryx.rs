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

use super::Discovery;
use super::DiscoveryError;

use iceoryx2::config::Config as IceoryxConfig;
use iceoryx2::node::Node as IceoryxNode;
use iceoryx2::port::subscriber::Subscriber as IceoryxSubscriber;
use iceoryx2::prelude::ServiceName;
use iceoryx2::service::static_config::messaging_pattern::MessagingPattern;
use iceoryx2_bb_log::info;
use iceoryx2_services_discovery::service_discovery::Discovery as DiscoveryUpdate;
use iceoryx2_services_discovery::service_discovery::Tracker as IceoryxServiceTracker;

// TODO: More granularity in errors
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Error,
}

pub(crate) struct IceoryxDiscovery<ServiceType: iceoryx2::service::Service> {
    iox_config: IceoryxConfig,
    iox_discovery_subscriber: Option<IceoryxSubscriber<ServiceType, DiscoveryUpdate, ()>>,
    iox_discovery_tracker: Option<IceoryxServiceTracker<ServiceType>>,
}

impl<ServiceType: iceoryx2::service::Service> IceoryxDiscovery<ServiceType> {
    pub fn create(
        iox_config: &IceoryxConfig,
        iox_node: &IceoryxNode<ServiceType>,
        iox_service_name: &Option<String>,
    ) -> Result<Self, CreationError> {
        let (iox_discovery_subscriber, iox_discovery_tracker) = match iox_service_name {
            Some(value) => {
                let iox_service_name: ServiceName = value
                    .as_str()
                    .try_into()
                    .map_err(|_e| CreationError::Error)?;

                info!("CONFIGURED Discovery updates from service {}", value);
                let iox_service = iox_node
                    .service_builder(&iox_service_name)
                    .publish_subscribe::<DiscoveryUpdate>()
                    .open_or_create()
                    .map_err(|_e| CreationError::Error)?;

                let iox_subscriber = iox_service
                    .subscriber_builder()
                    .create()
                    .map_err(|_e| CreationError::Error)?;

                (Some(iox_subscriber), None)
            }
            None => {
                info!("CONFIGURED Internal discovery tracking");
                (None, Some(IceoryxServiceTracker::<ServiceType>::new()))
            }
        };

        Ok(Self {
            iox_config: iox_config.clone(),
            iox_discovery_subscriber,
            iox_discovery_tracker,
        })
    }
}

impl<ServiceType: iceoryx2::service::Service> Discovery<ServiceType>
    for IceoryxDiscovery<ServiceType>
{
    fn discover<OnDiscovered: FnMut(&iceoryx2::service::static_config::StaticConfig)>(
        &mut self,
        on_discovered: &mut OnDiscovered,
    ) -> Result<(), super::DiscoveryError> {
        // EITHER Discover via external discovery service
        if let Some(iox_discovery_subscriber) = &self.iox_discovery_subscriber {
            loop {
                match iox_discovery_subscriber.receive() {
                    Ok(result) => match result {
                        Some(iox_sample) => {
                            if let DiscoveryUpdate::Added(iox_service_details) =
                                iox_sample.payload()
                            {
                                match iox_service_details.messaging_pattern() {
                                    MessagingPattern::PublishSubscribe(_) => {
                                        on_discovered(iox_service_details);
                                    }
                                    MessagingPattern::Event(_) => {
                                        on_discovered(iox_service_details);
                                    }
                                    _ => { /* Not supported. Nothing to do. */ }
                                }
                            }
                        }
                        None => break,
                    },
                    Err(_e) => {
                        return Err(DiscoveryError::Error);
                    }
                }
            }
        }
        // OR Discover via internal service tracker
        else if let Some(iox_discovery_tracker) = &mut self.iox_discovery_tracker {
            let (added, _removed) = iox_discovery_tracker
                .sync(&self.iox_config)
                .map_err(|_e| DiscoveryError::Error)?;

            for iox_service_id in added {
                if let Some(iox_service_details) = iox_discovery_tracker.get(&iox_service_id) {
                    let iox_service_details = &iox_service_details.static_details;

                    match iox_service_details.messaging_pattern() {
                        MessagingPattern::PublishSubscribe(_) => {
                            on_discovered(iox_service_details);
                        }
                        MessagingPattern::Event(_) => {
                            on_discovered(iox_service_details);
                        }
                        _ => { /* Not supported. Nothing to do. */ }
                    }
                }
            }
        }
        // SHOULD NOT HAPPEN: Neither were configured
        else {
            panic!("Unable to discover iceoryx services as neither the service discovery service nor a service tracker are set up");
        }

        Ok(())
    }
}
