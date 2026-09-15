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
use iceoryx2_link_backend::relay::{EventRelay, RelayBuilder, RelayFactory};
use iceoryx2_link_backend::service_description::EventDescription;
use iceoryx2_link_backend::{Backend, RemoteDescription};
use iceoryx2_log::{fail, origin};

use crate::bridge::{BridgeError, Bridged, Counters, OpenError};
use crate::ports::EventPorts;

/// An event service's local ports paired with the backend's relay for
/// them.
pub(super) struct EventBridge<S: Service, B: Backend<S>> {
    ports: EventPorts<S>,
    relay: B::EventRelay,
    counters: Counters,
}

impl<S: Service, B: Backend<S>> Bridged for EventBridge<S, B> {
    type Service = S;
    type Backend = B;
    type Pattern<'a> = EventDescription<'a>;

    fn open(
        node: &Node<S>,
        backend: &mut B,
        description: EventDescription<'_>,
        remote: &RemoteDescription<S, B>,
    ) -> Result<Self, OpenError> {
        let origin = origin!("EventBridge::open");

        let ports = fail!(
            from origin,
            when EventPorts::open(node, &description.name(), description.settings()),
            with OpenError::Ports,
            "Failed to open the local ports of {}", description.name()
        );

        let mut factory = backend.relay_factory();
        let relay = fail!(
            from origin,
            when factory.event(description, remote).create(),
            with OpenError::Relay,
            "Failed to create the relay of {}", description.name()
        );

        Ok(Self {
            ports,
            relay,
            counters: Counters::default(),
        })
    }

    /// The link's own notifications never reach its listener, so nothing
    /// is filtered by node.
    fn propagate(&mut self, _: &UniqueNodeId) -> Result<(), BridgeError> {
        let origin = origin!("EventBridge::propagate");

        let propagated = fail!(
            from origin,
            when self.ports.receive(|id| self.relay.send(id)),
            with BridgeError::Propagation,
            "Failed to propagate notifications"
        );
        self.counters.outbound += propagated;

        let ingested = fail!(
            from origin,
            when self.ports.send(|| self.relay.receive()),
            with BridgeError::Ingestion,
            "Failed to ingest notifications from the opposing side"
        );
        self.counters.inbound += ingested;

        Ok(())
    }

    fn counters(&mut self) -> &mut Counters {
        &mut self.counters
    }
}
