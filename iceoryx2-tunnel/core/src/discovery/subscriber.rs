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

use iceoryx2::node::Node;
use iceoryx2::port::ReceiveError;
use iceoryx2::{port::subscriber::Subscriber, service::Service};
use iceoryx2_bb_log::fail;
use iceoryx2_services_discovery::service_discovery::Discovery as DiscoveryEvent;

use iceoryx2_tunnel_traits::Discovery;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    FailedToCreateServiceName,
    FailedToCreateService,
    FailedToCreateSubscriber,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    FailedToReceiveDiscoveryEvent,
}

impl From<ReceiveError> for DiscoveryError {
    fn from(_: ReceiveError) -> Self {
        DiscoveryError::FailedToReceiveDiscoveryEvent
    }
}

pub struct DiscoverySubscriber<S: Service>(pub Subscriber<S, DiscoveryEvent, ()>);

impl<S: Service> DiscoverySubscriber<S> {
    pub fn create(node: &Node<S>, service_name: &str) -> Result<Self, CreationError> {
        let service_name = fail!(
            from "Tunnel::<S, T>::create_discovery_subscriber",
            when service_name.try_into(),
            with CreationError::FailedToCreateServiceName,
            "{}", &format!("Failed to create ServiceName '{}'", service_name)
        );

        let service = fail!(
            from "Tunnel::<S, T>::create_discovery_subscriber",
            when node.service_builder(&service_name)
                    .publish_subscribe::<DiscoveryEvent>()
                    .open_or_create(),
            with CreationError::FailedToCreateService,
            "{}", &format!("Failed to open DiscoveryService with ServiceName '{}'", service_name)
        );

        let subscriber = fail!(
            from "Tunnel::<S, T>::create_discovery_subscriber",
            when service.subscriber_builder().create(),
            with CreationError::FailedToCreateSubscriber,
            "{}", &format!("Failed to create DiscoverySubscriber with ServiceName '{}'", service_name)
        );

        Ok(Self(subscriber))
    }
}

impl<S: Service> Discovery for DiscoverySubscriber<S> {
    type DiscoveryError = DiscoveryError;

    fn discover<
        F: FnMut(&iceoryx2::service::static_config::StaticConfig) -> Result<(), Self::DiscoveryError>,
    >(
        &mut self,
        process_discovery: &mut F,
    ) -> Result<(), Self::DiscoveryError> {
        let subscriber = &self.0;
        loop {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    if let DiscoveryEvent::Added(static_config) = sample.payload() {
                        process_discovery(static_config)?;
                    }
                }
                Ok(None) => break Ok(()),
                Err(e) => {
                    fail!(from "DiscoverySubscriber<S>::discover",
                        with e.into(),
                        "Failed to receive from discovery subscriber"
                    );
                }
            }
        }
    }
}
