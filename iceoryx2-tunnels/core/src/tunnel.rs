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

use std::collections::HashMap;

use crate::{Relay, Transport};

use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::listener::Listener;
use iceoryx2::port::notifier::Notifier;
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::service::builder::CustomHeaderMarker;
use iceoryx2::service::builder::CustomPayloadMarker;
use iceoryx2::service::service_id::ServiceId;
use iceoryx2::service::Service;
use iceoryx2_bb_log::fail;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    Error,
}

#[derive(Default)]
pub struct TunnelConfig {}

enum Ports<S: Service> {
    PublishSubscribe(
        Publisher<S, [CustomPayloadMarker], CustomHeaderMarker>,
        Subscriber<S, [CustomPayloadMarker], CustomHeaderMarker>,
    ),
    Event(Notifier<S>, Listener<S>),
}

/// A generic tunnel implementation that works with any implemented Transport.
pub struct Tunnel<S: Service, T: Transport> {
    node: Node<S>,
    transport: T,
    ports: HashMap<ServiceId, Ports<S>>,
    relays: HashMap<ServiceId, Box<dyn Relay>>,
}

impl<S: Service, T: Transport> Tunnel<S, T> {
    /// create a new tunnel instance using the given transport
    pub fn create(
        tunnel_config: &TunnelConfig,
        iceoryx_config: &iceoryx2::config::Config,
        transport_config: &T::TransportConfig,
    ) -> Result<Self, Error> {
        let node = NodeBuilder::new().config(iceoryx_config).create::<S>();
        let node = fail!(
            from "Tunnel::<S, T>::create",
            when node,
            with Error::Error,
            "failed to create node"
        );

        let transport = Transport::create(transport_config);
        let transport = fail!(
            from "Tunnel::<S, T>::create",
            when transport,
            with Error::Error,
            "failed to instantiate the transport"
        );

        Ok(Self {
            node,
            transport,
            ports: HashMap::new(),
            relays: HashMap::new(),
        })
    }

    /// discover services across the extended system
    pub fn discover(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// propagate payloads data over all relays
    pub fn propagate(&mut self) -> Result<(), Error> {
        for (id, ports) in &self.ports {
            let relay = &self.relays.get(id);

            match ports {
                Ports::PublishSubscribe(publisher, subscriber) => todo!(),
                Ports::Event(notifier, listener) => todo!(),
            }
        }

        Ok(())
    }
}
