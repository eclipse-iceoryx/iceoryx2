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
use iceoryx2_link_backend::description::PublishSubscribeDescription;
use iceoryx2_link_backend::origin;
use iceoryx2_link_backend::relay::{PublishSubscribeRelay, RelayBuilder, RelayFactory};
use iceoryx2_link_backend::{Backend, RemoteDescription};
use iceoryx2_log::{debug, fail};

use crate::bridge::{Bridged, OpenError, PropagateError};
use crate::ports::PublishSubscribePorts;

/// A publish-subscribe service's local ports paired with the backend's
/// relay for them.
pub(super) struct PublishSubscribeBridge<S: Service, B: Backend<S>> {
    ports: PublishSubscribePorts<S>,
    relay: B::PublishSubscribeRelay,
}

impl<S: Service, B: Backend<S>> Bridged for PublishSubscribeBridge<S, B> {
    type Service = S;
    type Backend = B;
    type Pattern<'a> = PublishSubscribeDescription<'a>;

    fn open(
        node: &Node<S>,
        backend: &B,
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
        let relay = fail!(
            from origin,
            when backend.relay_factory().publish_subscribe(description, remote).create(),
            with OpenError::Relay,
            "Failed to create the relay of {}", description.name()
        );
        Ok(Self { ports, relay })
    }

    fn propagate(&self, own_node: &UniqueNodeId) -> Result<(), PropagateError> {
        let origin = origin!("PublishSubscribeBridge::propagate");
        fail!(
            from origin,
            when self.ports.receive(own_node, |sample| {
                debug!(from origin, "Relaying a sample of {}", self.ports.name());
                self.relay.send(&sample)
            }),
            with PropagateError::Propagation,
            "Failed to propagate samples"
        );
        fail!(
            from origin,
            when self.ports.send(|loan| self.relay.receive(loan)),
            with PropagateError::Ingestion,
            "Failed to ingest samples from the opposing side"
        );
        Ok(())
    }
}
