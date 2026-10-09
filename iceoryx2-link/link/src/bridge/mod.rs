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

//! A bridge relays samples between one local service and the opposing side.
//!
//! ```text
//!   application ◀──▶ ┌─────────┐ ◀──▶ ┌────────┐ ◀──▶ ┌───────┐ ◀──▶ opposing side
//!                    │ service │      │ bridge │      │ relay │
//!                    └─────────┘      └────────┘      └───────┘
//! ```

mod event;
mod publish_subscribe;
mod table;

use core::ops::Deref;
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
use crate::diagnostics::{Diagnostics, OpenFailure};
use crate::discovery::BridgeableService;

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

/// Why one bridge failed to propagate.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum BridgeError {
    Propagation,
    Ingestion,
}

impl core::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "BridgeError::{self:?}")
    }
}

impl core::error::Error for BridgeError {}

/// How many bridges failed to propagate in a cycle.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum PropagateError {
    Bridges(usize),
}

impl core::fmt::Display for PropagateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "PropagateError::{self:?}")
    }
}

impl core::error::Error for PropagateError {}

/// What a bridge moved in each direction since it was last reported.
#[derive(Default)]
struct Monitoring {
    outbound: u64,
    inbound: u64,
}

impl Monitoring {
    /// The outbound and inbound counts, reset to zero.
    fn take(&mut self) -> (u64, u64) {
        let counts = (self.outbound, self.inbound);
        *self = Self::default();
        counts
    }
}

/// The link's local ports of one pattern joined to the backend's relay
/// for them.
trait Bridge: Sized {
    type Service: Service;
    type Backend: Backend<Self::Service>;
    /// The local description seen as this bridge's pattern.
    type LocalDescription<'a>: Copy + Deref<Target = ServiceDescription>;

    fn open(
        node: &Node<Self::Service>,
        backend: &mut Self::Backend,
        local_description: Self::LocalDescription<'_>,
        remote_description: &RemoteDescription<Self::Service, Self::Backend>,
    ) -> Result<Self, OpenError>;

    /// Moves what is pending in both directions.
    fn propagate(&mut self, node: &Node<Self::Service>, notify: bool) -> Result<(), BridgeError>;

    /// What was moved since last taken.
    fn monitoring(&mut self) -> &mut Monitoring;
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
        bridgeable: impl Iterator<Item = BridgeableService<'a, RemoteDescription<S, B>>>,
    ) where
        B: 'a,
    {
        self.epoch = self.epoch.next();
        let epoch = self.epoch;
        let mut failed = self.diagnostics.failed_opens();

        for entry in bridgeable {
            let BridgeableService {
                hash,
                local_description,
                remote_description,
                resolved,
            } = entry;

            let opened = match local_description.messaging_pattern() {
                MessagingPattern::PublishSubscribe(local_description) => {
                    self.publish_subscribe.reconcile(
                        node,
                        backend,
                        local_description,
                        remote_description,
                        resolved,
                        epoch,
                    )
                }
                MessagingPattern::Event(local_description) => self.event.reconcile(
                    node,
                    backend,
                    local_description,
                    remote_description,
                    resolved,
                    epoch,
                ),
                MessagingPattern::RequestResponse(_) => {
                    unreachable!("the link does not bridge request-response yet")
                }
            };
            if let Err(error) = opened {
                let failure = OpenFailure {
                    description: local_description.clone(),
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
    /// arrives before it. Every bridge is attempted even when one fails,
    /// the failures are reported once each and counted in the error.
    pub(crate) fn propagate(
        &mut self,
        node: &Node<S>,
        monitoring: bool,
        notify: bool,
    ) -> Result<(), PropagateError> {
        let origin = origin!("Bridges::propagate");
        let publish_subscribe = self.publish_subscribe.propagate(node, notify);
        let event = self.event.propagate(node, notify);

        let mut failed = self.diagnostics.failed_propagations();
        for (hash, failure) in self.publish_subscribe.failed().chain(self.event.failed()) {
            failed.record(*hash, failure);
        }
        failed.commit();

        if monitoring {
            self.publish_subscribe.report();
            self.event.report();
        }

        let failures = failed_bridges(publish_subscribe) + failed_bridges(event);
        if failures != 0 {
            fail!(
                from origin,
                with PropagateError::Bridges(failures),
                "Failed to propagate {} of {} bridges", failures, self.len()
            );
        }

        Ok(())
    }

    fn len(&self) -> usize {
        self.publish_subscribe.len() + self.event.len()
    }
}

/// How many bridges a table's propagation failed on.
fn failed_bridges(outcome: Result<(), PropagateError>) -> usize {
    match outcome {
        Ok(()) => 0,
        Err(PropagateError::Bridges(failures)) => failures,
    }
}
