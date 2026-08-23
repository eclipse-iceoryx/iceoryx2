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

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;

use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::Node;
use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_gateway_backend::traits::{
    Backend, EventRelay, PublishSubscribeRelay, RelayBuilder, RelayFactory,
};
use iceoryx2_gateway_backend::types::publish_subscribe::LoanFn;
use iceoryx2_gateway_backend::types::service_description::{
    EventDescription, PatternDescription, PublishSubscribeDescription, ServiceDescription,
};
use iceoryx2_log::{fail, info, warn};

use crate::gateway::{DiscoveryError, PropagateError};
use crate::ports::event::EventPorts;
use crate::ports::publish_subscribe::PublishSubscribePorts;

/// Describes the outcome of opening up a bridge for a service.
///
/// Failed attempts are additionally tracked to prevent constantly retrying
/// to create a bridge that causes an error.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
enum BridgeState<T> {
    /// The bridge is established and relaying between both sides.
    Established(T),
    /// The bridge could not be established. No reattemtps to reestablish
    /// unless the service is reinstantiated.
    Failed,
}

impl<T> BridgeState<T> {
    // Returns the established bridge or None if there was a failure.
    fn established(&self) -> Option<&T> {
        match self {
            BridgeState::Established(bridge) => Some(bridge),
            BridgeState::Failed => None,
        }
    }

    fn is_established(&self) -> bool {
        self.established().is_some()
    }
}

/// All bridges opened by the gateway, stored per messaging pattern so that
/// propagation can process each pattern's bridges without filtering.
#[derive(Debug)]
pub(crate) struct Bridges<S: Service, B: Backend<S>> {
    publish_subscribe: BTreeMap<ServiceHash, BridgeState<PublishSubscribeBridge<S, B>>>,
    event: BTreeMap<ServiceHash, BridgeState<EventBridge<S, B>>>,
}

impl<S: Service, B: Backend<S>> Default for Bridges<S, B> {
    fn default() -> Self {
        Self {
            publish_subscribe: BTreeMap::new(),
            event: BTreeMap::new(),
        }
    }
}

impl<S: Service, B: Backend<S>> Bridges<S, B> {
    /// Opens the ports and relay matching the messaging pattern of
    /// `description` and records the outcome, including failures.
    pub(crate) fn open(&mut self, node: &Node<S>, backend: &B, description: &ServiceDescription) {
        let hash = description.service_hash;
        match &description.pattern {
            PatternDescription::PublishSubscribe(pattern_description) => {
                let state = into_state(
                    PublishSubscribeBridge::open(node, backend, description, pattern_description),
                    description,
                );
                self.publish_subscribe.insert(hash, state);
            }
            PatternDescription::Event(pattern_description) => {
                let state = into_state(
                    EventBridge::open(node, backend, description, pattern_description),
                    description,
                );
                self.event.insert(hash, state);
            }
        }
    }

    /// Whether a bridge for the service is tracked, established or failed.
    pub(crate) fn contains(&self, hash: &ServiceHash) -> bool {
        self.publish_subscribe.contains_key(hash) || self.event.contains_key(hash)
    }

    /// Number of tracked bridges, established or failed.
    pub(crate) fn len(&self) -> usize {
        self.publish_subscribe.len() + self.event.len()
    }

    /// Retains only the bridges for which `keep` returns true. The closure
    /// additionally receives whether the bridge is established.
    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&ServiceHash, bool) -> bool) {
        self.publish_subscribe
            .retain(|hash, state| keep(hash, state.is_established()));
        self.event
            .retain(|hash, state| keep(hash, state.is_established()));
    }

    /// The hashes of all established bridges.
    pub(crate) fn established(&self) -> BTreeSet<ServiceHash> {
        established_hashes(&self.publish_subscribe)
            .chain(established_hashes(&self.event))
            .collect()
    }

    /// Propagates payloads/events in both directions for all established
    /// bridges. Payload-carrying patterns propagate before events.
    ///
    /// The exhaustive destructuring forces a newly added messaging pattern to
    /// be given a position in the propagation order.
    // TODO(#1103): Retain ordering across the wire
    pub(crate) fn propagate(&self, node_id: &UniqueNodeId) -> Result<(), PropagateError> {
        let Bridges {
            publish_subscribe,
            event,
        } = self;

        for bridge in publish_subscribe
            .values()
            .filter_map(BridgeState::established)
        {
            bridge.propagate(node_id)?;
        }
        for bridge in event.values().filter_map(BridgeState::established) {
            bridge.propagate(node_id)?;
        }

        Ok(())
    }
}

fn established_hashes<T>(
    map: &BTreeMap<ServiceHash, BridgeState<T>>,
) -> impl Iterator<Item = ServiceHash> + '_ {
    map.iter()
        .filter(|(_, state)| state.is_established())
        .map(|(hash, _)| *hash)
}

fn into_state<T>(
    result: Result<T, DiscoveryError>,
    description: &ServiceDescription,
) -> BridgeState<T> {
    match result {
        Ok(bridge) => BridgeState::Established(bridge),
        Err(error) => {
            warn!(
                from "Bridges::open",
                "{}({}) will not be bridged: {}",
                description.pattern,
                description.name,
                error
            );
            BridgeState::Failed
        }
    }
}

/// A bidirectional bridge for a single publish-subscribe service.
#[derive(Debug)]
struct PublishSubscribeBridge<S: Service, B: Backend<S>> {
    ports: PublishSubscribePorts<S>,
    relay: B::PublishSubscribeRelay,
}

impl<S: Service, B: Backend<S>> PublishSubscribeBridge<S, B> {
    fn open(
        node: &Node<S>,
        backend: &B,
        description: &ServiceDescription,
        pattern_description: &PublishSubscribeDescription,
    ) -> Result<Self, DiscoveryError> {
        let origin = "PublishSubscribeBridge::open";

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
        Ok(Self { ports, relay })
    }

    fn propagate(&self, node_id: &UniqueNodeId) -> Result<(), PropagateError> {
        let origin = format!("PublishSubscribeBridge({node_id})::propagate");
        let port = &self.ports;
        let relay = &self.relay;

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
}

/// A bidirectional bridge for a single event service.
#[derive(Debug)]
struct EventBridge<S: Service, B: Backend<S>> {
    ports: EventPorts<S>,
    relay: B::EventRelay,
}

impl<S: Service, B: Backend<S>> EventBridge<S, B> {
    fn open(
        node: &Node<S>,
        backend: &B,
        description: &ServiceDescription,
        pattern_description: &EventDescription,
    ) -> Result<Self, DiscoveryError> {
        let origin = "EventBridge::open";

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
        Ok(Self { ports, relay })
    }

    fn propagate(&self, node_id: &UniqueNodeId) -> Result<(), PropagateError> {
        let origin = format!("EventBridge({node_id})::propagate");
        let port = &self.ports;
        let relay = &self.relay;

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
}
