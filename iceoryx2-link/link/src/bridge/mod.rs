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

mod event;
mod publish_subscribe;
mod table;

use core::ops::Deref;
use iceoryx2::identifiers::UniqueNodeId;
use iceoryx2::node::Node;
use iceoryx2::service::Service;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_elementary::epoch::Epoch;
use iceoryx2_link_backend::service_description::{MessagingPattern, ServiceDescription};
use iceoryx2_link_backend::{Backend, RemoteDescription};
use iceoryx2_log::{fail, origin};

use crate::bridge::event::EventBridge;
use crate::bridge::publish_subscribe::PublishSubscribeBridge;
use crate::bridge::table::BridgeTable;
use crate::diagnostics::{Diagnostics, Failure};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum OpenError {
    Ports,
    Relay,
}

impl core::fmt::Display for OpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OpenError::{self:?}")
    }
}

impl core::error::Error for OpenError {}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagateError {
    Propagation,
    Ingestion,
}

impl core::fmt::Display for PropagateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PropagateError::{self:?}")
    }
}

impl core::error::Error for PropagateError {}

/// How much a bridge moved in each direction since last taken.
#[derive(Default)]
struct Counters {
    outbound: u64,
    inbound: u64,
}

impl Counters {
    /// The outbound and inbound counts, reset to zero.
    fn take(&mut self) -> (u64, u64) {
        let counts = (self.outbound, self.inbound);
        *self = Self::default();
        counts
    }
}

/// The link's local ports of one pattern joined to the backend's relay
/// for them.
trait Bridged: Sized {
    type Service: Service;
    type Backend: Backend<Self::Service>;
    /// The description seen as this pattern.
    type Pattern<'a>: Copy + Deref<Target = ServiceDescription>;

    fn open(
        node: &Node<Self::Service>,
        backend: &mut Self::Backend,
        description: Self::Pattern<'_>,
        remote: &RemoteDescription<Self::Service, Self::Backend>,
    ) -> Result<Self, OpenError>;

    /// Moves what is pending in both directions.
    fn propagate(&mut self, own_node: &UniqueNodeId) -> Result<(), PropagateError>;

    /// What was moved since last taken.
    fn counters(&mut self) -> &mut Counters;
}

pub struct Bridges<S: Service, B: Backend<S>> {
    publish_subscribe: BridgeTable<PublishSubscribeBridge<S, B>>,
    event: BridgeTable<EventBridge<S, B>>,
    diagnostics: Diagnostics,
    /// Moved by every reconcile to track changes.
    epoch: Epoch,
}

impl<S: Service, B: Backend<S>> Default for Bridges<S, B> {
    fn default() -> Self {
        Self {
            publish_subscribe: BridgeTable::default(),
            event: BridgeTable::default(),
            diagnostics: Diagnostics::default(),
            epoch: Epoch::default(),
        }
    }
}

impl<S: Service, B: Backend<S>> Bridges<S, B> {
    /// Opens bridges for bridgeable services that have none or were
    /// resolved anew, and closes bridges of services no longer
    /// bridgeable. A bridge that fails to open is tried again on the next
    /// call, the failure reported once per description.
    pub(crate) fn reconcile<'a>(
        &mut self,
        node: &Node<S>,
        backend: &mut B,
        bridgeable: impl Iterator<
            Item = (
                &'a ServiceHash,
                &'a ServiceDescription,
                &'a RemoteDescription<S, B>,
                Epoch,
            ),
        >,
    ) where
        B: 'a,
    {
        self.epoch = self.epoch.next();
        let epoch = self.epoch;
        let mut failed = self.diagnostics.failed_bridges();

        for (hash, description, remote, resolved) in bridgeable {
            let opened = match description.messaging_pattern() {
                MessagingPattern::PublishSubscribe(description) => self
                    .publish_subscribe
                    .reconcile(node, backend, description, remote, resolved, epoch),
                MessagingPattern::Event(description) => {
                    self.event
                        .reconcile(node, backend, description, remote, resolved, epoch)
                }
            };
            if let Err(error) = opened {
                let failure = Failure {
                    description: description.clone(),
                    error,
                };
                failed.record(*hash, failure);
            }
        }

        self.publish_subscribe.retain(epoch);
        self.event.retain(epoch);
        failed.commit();
    }

    /// Whether a service is bridged.
    pub fn contains(&self, hash: &ServiceHash) -> bool {
        self.publish_subscribe.contains(hash) || self.event.contains(hash)
    }

    /// Whether no service is bridged.
    pub fn empty(&self) -> bool {
        self.publish_subscribe.is_empty() && self.event.is_empty()
    }

    /// Moves pending samples and notifications of every bridge in both
    /// directions, samples first so a notification about one never
    /// arrives before it. With `monitoring`, reports what each bridge
    /// moved.
    pub(crate) fn propagate(
        &mut self,
        own_node: &UniqueNodeId,
        monitoring: bool,
    ) -> Result<(), PropagateError> {
        let origin = origin!("Bridges::propagate");
        for bridge in self.publish_subscribe.bridged_mut() {
            fail!(
                from origin,
                when bridge.propagate(own_node),
                "Failed to propagate a publish-subscribe bridge"
            );
        }
        for bridge in self.event.bridged_mut() {
            fail!(
                from origin,
                when bridge.propagate(own_node),
                "Failed to propagate an event bridge"
            );
        }
        if monitoring {
            self.publish_subscribe.report();
            self.event.report();
        }
        Ok(())
    }
}
