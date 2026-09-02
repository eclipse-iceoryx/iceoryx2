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

/// All bridges opened by the gateway.
#[derive(Debug)]
pub(crate) struct Bridges<S: Service, B: Backend<S>> {
    publish_subscribe: BTreeMap<ServiceHash, PublishSubscribeBridge<S, B>>,
    event: BTreeMap<ServiceHash, EventBridge<S, B>>,
    /// Services whose bridge could not be established, with the description
    /// the attempt was made for.
    failed: BTreeMap<ServiceHash, ServiceDescription>,
}

impl<S: Service, B: Backend<S>> Default for Bridges<S, B> {
    fn default() -> Self {
        Self {
            publish_subscribe: BTreeMap::new(),
            event: BTreeMap::new(),
            failed: BTreeMap::new(),
        }
    }
}

impl<S: Service, B: Backend<S>> Bridges<S, B> {
    /// Opens the ports and relay matching the messaging pattern of
    /// `description`.
    pub(crate) fn open(&mut self, node: &Node<S>, backend: &B, description: &ServiceDescription) {
        let hash = description.service_hash;
        match &description.pattern {
            PatternDescription::PublishSubscribe(pattern_description) => {
                match PublishSubscribeBridge::open(node, backend, description, pattern_description)
                {
                    Ok(bridge) => {
                        self.publish_subscribe.insert(hash, bridge);
                    }
                    Err(error) => self.record_failure(description, error),
                }
            }
            PatternDescription::Event(pattern_description) => {
                match EventBridge::open(node, backend, description, pattern_description) {
                    Ok(bridge) => {
                        self.event.insert(hash, bridge);
                    }
                    Err(error) => self.record_failure(description, error),
                }
            }
        }
    }

    /// Whether an established bridge for the service exists.
    pub(crate) fn is_established(&self, hash: &ServiceHash) -> bool {
        let Bridges {
            publish_subscribe,
            event,
            failed: _,
        } = self;

        publish_subscribe.contains_key(hash) || event.contains_key(hash)
    }

    /// The description a failed establishment attempt was made for.
    pub(crate) fn failed_description(&self, hash: &ServiceHash) -> Option<&ServiceDescription> {
        self.failed.get(hash)
    }

    /// Drops the record of a failed establishment attempt.
    pub(crate) fn clear_failed(&mut self, hash: &ServiceHash) {
        self.failed.remove(hash);
    }

    /// Number of tracked bridges, established or failed.
    #[cfg(debug_assertions)]
    pub(crate) fn number_of_tracked_services(&self) -> usize {
        let Bridges {
            publish_subscribe,
            event,
            failed,
        } = self;

        publish_subscribe.len() + event.len() + failed.len()
    }

    /// Retains only the bridges for which `keep` returns true. `on_close` is
    /// invoked for every established bridge that is dropped.
    pub(crate) fn retain(
        &mut self,
        mut keep: impl FnMut(&ServiceHash) -> bool,
        mut on_close: impl FnMut(&ServiceHash),
    ) {
        let Bridges {
            publish_subscribe,
            event,
            failed,
        } = self;

        let mut keep_or_close = |hash: &ServiceHash| {
            let keep = keep(hash);
            if !keep {
                on_close(hash);
            }
            keep
        };

        publish_subscribe.retain(|hash, _| keep_or_close(hash));
        event.retain(|hash, _| keep_or_close(hash));
        failed.retain(|hash, _| keep(hash));
    }

    /// The hashes of all established bridges.
    pub(crate) fn established(&self) -> BTreeSet<ServiceHash> {
        let Bridges {
            publish_subscribe,
            event,
            failed: _,
        } = self;

        publish_subscribe
            .keys()
            .chain(event.keys())
            .copied()
            .collect()
    }

    /// Propagates payloads/events in both directions for all established
    /// bridges. Payload-carrying patterns propagate before events.
    ///
    // TODO(#1103): Retain ordering across the wire
    pub(crate) fn propagate(&self, node_id: &UniqueNodeId) -> Result<(), PropagateError> {
        let Bridges {
            publish_subscribe,
            event,
            failed: _,
        } = self;

        for bridge in publish_subscribe.values() {
            bridge.propagate(node_id)?;
        }
        for bridge in event.values() {
            bridge.propagate(node_id)?;
        }

        Ok(())
    }

    fn record_failure(&mut self, description: &ServiceDescription, error: DiscoveryError) {
        warn!(
            from "Bridges::open",
            "{}({}) will not be bridged: {}",
            description.pattern,
            description.name,
            error
        );
        self.failed
            .insert(description.service_hash, description.clone());
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
