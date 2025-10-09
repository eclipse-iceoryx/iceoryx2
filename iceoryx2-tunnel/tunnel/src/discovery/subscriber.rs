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
use iceoryx2::prelude::ServiceName;
use iceoryx2::{port::subscriber::Subscriber, service::Service};
use iceoryx2_bb_log::fail;
use iceoryx2_services_discovery::service_discovery::Discovery as DiscoveryEvent;

use iceoryx2_tunnel_backend::traits::Discovery;
use iceoryx2_tunnel_backend::types::discovery::ProcessDiscoveryFn;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum CreationError {
    Service,
    Subscriber,
}

impl core::fmt::Display for CreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "CreationError::{self:?}")
    }
}

impl core::error::Error for CreationError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum DiscoveryError {
    ReceivingFromIceoryx,
    DiscoveryProcessing,
}

impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "DiscoveryError::{self:?}")
    }
}

impl core::error::Error for DiscoveryError {}

#[derive(Debug)]
pub struct DiscoverySubscriber<S: Service>(pub Subscriber<S, DiscoveryEvent, ()>);

impl<S: Service> DiscoverySubscriber<S> {
    pub fn create(node: &Node<S>, service_name: ServiceName) -> Result<Self, CreationError> {
        let service = fail!(
            from "Tunnel::<S, T>::create_discovery_subscriber",
            when node.service_builder(&service_name)
                    .publish_subscribe::<DiscoveryEvent>()
                    .open(),
            with CreationError::Service,
            "Failed to open discovery service with name {}", service_name
        );

        let subscriber = fail!(
            from "Tunnel::<S, T>::create_discovery_subscriber",
            when service.subscriber_builder().create(),
            with CreationError::Subscriber,
            "Failed to create subscriber for discovery service with name {}", service_name
        );

        Ok(Self(subscriber))
    }
}

impl<S: Service> Discovery for DiscoverySubscriber<S> {
    type DiscoveryError = DiscoveryError;

    fn discover<ProcessDiscoveryError>(
        &self,
        process_discovery: &mut ProcessDiscoveryFn<ProcessDiscoveryError>,
    ) -> Result<(), Self::DiscoveryError> {
        let subscriber = &self.0;
        loop {
            match subscriber.receive() {
                Ok(Some(sample)) => {
                    if let DiscoveryEvent::Added(static_config) = sample.payload() {
                        fail!(from "DiscoverySubscriber::discover",
                            when process_discovery(static_config),
                            with DiscoveryError::DiscoveryProcessing,
                            "Failed to process discovery event"
                        );
                    }
                }
                Ok(None) => break Ok(()),
                Err(_) => {
                    fail!(from "DiscoverySubscriber::discover",
                        with DiscoveryError::ReceivingFromIceoryx,
                        "Failed to receive from discovery subscriber"
                    );
                }
            }
        }
    }
}
