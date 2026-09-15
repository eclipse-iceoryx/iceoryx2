// Copyright (c) 2026 Contributors to the Eclipse Foundation
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

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::Node;
use iceoryx2::service::Service;
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, RelayBuilder, RelayFactory};
use iceoryx2_link_backend::service_description::PublishSubscribeDescription;
use iceoryx2_link_backend::{Backend, RemoteDescription};
use iceoryx2_log::{fail, origin};

use crate::bridge::{BridgeError, Bridged, Counters, OpenError};
use crate::ports::PublishSubscribePorts;

/// A publish-subscribe service's local ports paired with the backend's
/// relay for them.
pub(super) struct PublishSubscribeBridge<S: Service, B: Backend<S>> {
    ports: PublishSubscribePorts<S>,
    relay: B::PublishSubscribeRelay,
    counters: Counters,
}

impl<S: Service, B: Backend<S>> Bridged for PublishSubscribeBridge<S, B> {
    type Service = S;
    type Backend = B;
    type Pattern<'a> = PublishSubscribeDescription<'a>;

    fn open(
        node: &Node<S>,
        backend: &mut B,
        description: PublishSubscribeDescription<'_>,
        remote: &RemoteDescription<S, B>,
    ) -> Result<Self, OpenError> {
        let origin = origin!("PublishSubscribeBridge::open");

        let ports = fail!(
            from origin,
            when PublishSubscribePorts::open(node, &description.name(), description.settings(), description.types()),
            with OpenError::Ports,
            "Failed to open the local ports of {}", description.name()
        );

        let mut factory = backend.relay_factory();
        let relay = fail!(
            from origin,
            when factory.publish_subscribe(description, remote).create(),
            with OpenError::Relay,
            "Failed to create the relay of {}", description.name()
        );

        Ok(Self {
            ports,
            relay,
            counters: Counters::default(),
        })
    }

    fn propagate(&mut self, own_node: &UniqueNodeId) -> Result<(), BridgeError> {
        let origin = origin!("PublishSubscribeBridge::propagate");

        let propagated = fail!(
            from origin,
            when self.ports.receive(own_node, |sample| self.relay.send(&sample)),
            with BridgeError::Propagation,
            "Failed to propagate samples"
        );
        self.counters.outbound += propagated;

        let ingested = fail!(
            from origin,
            when self.ports.send(|loan| self.relay.receive(loan)),
            with BridgeError::Ingestion,
            "Failed to ingest samples from the opposing side"
        );
        self.counters.inbound += ingested;
        Ok(())
    }

    fn counters(&mut self) -> &mut Counters {
        &mut self.counters
    }
}
