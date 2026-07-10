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

use alloc::format;

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::Node;
use iceoryx2::service::Service;
use iceoryx2_log::{fail, info};
use iceoryx2_services_tunnel_backend::traits::{
    Backend, EventRelay, PublishSubscribeRelay, RelayBuilder, RelayFactory,
};
use iceoryx2_services_tunnel_backend::types::publish_subscribe::LoanFn;
use iceoryx2_services_tunnel_backend::types::service_description::{
    PatternDescription, ServiceDescription,
};

use crate::ports::event::EventPorts;
use crate::ports::publish_subscribe::PublishSubscribePorts;
use crate::tunnel::{DiscoveryError, PropagateError};

/// A bidirectional bridge for a single service: the local iceoryx2 ports on one
/// side and the backend relay on the other.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum Bridge<S: Service, B: Backend<S>> {
    PublishSubscribe {
        ports: PublishSubscribePorts<S>,
        relay: B::PublishSubscribeRelay,
    },
    Event {
        ports: EventPorts<S>,
        relay: B::EventRelay,
    },
}

impl<S: Service, B: Backend<S>> Bridge<S, B> {
    /// Creates the ports and relay matching the messaging pattern of
    /// `description`.
    pub(crate) fn open(
        node: &Node<S>,
        backend: &B,
        description: &ServiceDescription,
    ) -> Result<Self, DiscoveryError> {
        let origin = "Bridge::open";

        match &description.pattern {
            PatternDescription::PublishSubscribe(pattern_description) => {
                let ports = fail!(
                    from origin,
                    when PublishSubscribePorts::new(&description.name, pattern_description, node),
                    with DiscoveryError::PublishSubscribePortCreation,
                    "Failed to create publish-subscribe ports"
                );
                let relay = fail!(
                    from origin,
                    when backend.relay_builder().publish_subscribe(description).create(),
                    with DiscoveryError::PublishSubscribeRelayCreation,
                    "Failed to create publish-subscribe relay"
                );
                Ok(Bridge::PublishSubscribe { ports, relay })
            }
            PatternDescription::Event(pattern_description) => {
                let ports = fail!(
                    from origin,
                    when EventPorts::new(&description.name, pattern_description, node),
                    with DiscoveryError::EventPortsCreation,
                    "Failed to create event ports"
                );
                let relay = fail!(
                    from origin,
                    when backend.relay_builder().event(description).create(),
                    with DiscoveryError::EventRelayCreation,
                    "Failed to create event relay"
                );
                Ok(Bridge::Event { ports, relay })
            }
        }
    }

    /// Propagates payloads/events in both directions for this bridge.
    pub(crate) fn propagate(&self, node_id: &UniqueNodeId) -> Result<(), PropagateError> {
        match self {
            Bridge::PublishSubscribe { ports, relay } => {
                propagate_publish_subscribe_payloads::<S, B>(node_id, ports, relay)
            }
            Bridge::Event { ports, relay } => propagate_events::<S, B>(node_id, ports, relay),
        }
    }
}

fn propagate_publish_subscribe_payloads<S: Service, B: Backend<S>>(
    node_id: &UniqueNodeId,
    port: &PublishSubscribePorts<S>,
    relay: &B::PublishSubscribeRelay,
) -> Result<(), PropagateError> {
    let origin = format!("Bridge({node_id})::propagate_publish_subscribe_payloads");

    let propagated = fail!(
        from origin,
        when port.receive(node_id, |sample| {
            relay.send(sample)
        }),
        with PropagateError::PayloadPropagation,
        "Failed to receive publish-subscribe payload for propagation"
    );
    if propagated {
        info!(from origin, "Propagated PublishSubscribe({})", port.name);
    }

    let ingested = fail!(
        from origin,
        when port.send(|loan: &mut LoanFn<_, _>| {
            relay.receive::<_>(&mut |size| {
            loan(size)})
        }),
        with PropagateError::PayloadIngestion,
        "Failed to ingest publish-subscribe payload received from backend"
    );
    if ingested {
        info!(from origin, "Ingested PublishSubscribe({})", port.name);
    }

    Ok(())
}

fn propagate_events<S: Service, B: Backend<S>>(
    node_id: &UniqueNodeId,
    port: &EventPorts<S>,
    relay: &B::EventRelay,
) -> Result<(), PropagateError> {
    let origin = format!("Bridge({node_id})::propagate_events");

    let propagated = fail!(
        from origin,
        when port.receive(|id| {
            relay.send(id)
        }),
        with PropagateError::EventPropagation,
        "Failed to receive events for propagation"
    );
    if propagated {
        info!(from origin, "Propagated Event({})", port.name);
    }

    let ingested = fail!(
        from origin,
        when port.send(|| {
            relay.receive()
        }),
        with PropagateError::EventIngestion,
        "Failed to ingest event received from backend"
    );
    if ingested {
        info!(from origin, "Ingested Event({})", port.name);
    }

    Ok(())
}
