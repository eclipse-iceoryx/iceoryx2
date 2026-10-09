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

use alloc::collections::BTreeMap;

use iceoryx2::node::Node;
use iceoryx2::service::messaging_pattern::MessagingPattern;
use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2::service::service_name::ServiceName;
use iceoryx2_bb_elementary::epoch::Epoch;
use iceoryx2_link_backend::RemoteDescription;
use iceoryx2_log::{fail, origin, trace};

use crate::bridge::{Bridge, BridgeError, OpenError, PropagateError};
use crate::diagnostics::PropagationFailure;

/// A service's bridge, stamped with the resolution it was opened for and
/// the reconcile that last saw it.
struct BridgedService<B> {
    bridge: B,
    name: ServiceName,
    pattern: MessagingPattern,
    /// The epoch the service was resolved in when the bridge was opened.
    resolved: Epoch,
    /// The epoch of the reconcile that last saw the service bridgeable.
    seen: Epoch,
    /// How the bridge's last propagation ended.
    failure: Option<BridgeError>,
}

/// The bridges of one pattern.
pub(super) struct BridgeTable<B> {
    services: BTreeMap<ServiceHash, BridgedService<B>>,
}

impl<B> Default for BridgeTable<B> {
    fn default() -> Self {
        Self {
            services: BTreeMap::new(),
        }
    }
}

impl<B: Bridge> BridgeTable<B> {
    /// Stamps the bridge of a bridgeable service, opening it first if the
    /// service has none or was resolved anew since its bridge was opened.
    pub(super) fn reconcile(
        &mut self,
        node: &Node<B::Service>,
        backend: &mut B::Backend,
        local_description: B::LocalDescription<'_>,
        remote_description: &RemoteDescription<B::Service, B::Backend>,
        resolved: Epoch,
        epoch: Epoch,
    ) -> Result<(), OpenError> {
        let origin = origin!("BridgeTable::reconcile");

        let hash = local_description.hash();
        if let Some(service) = self.services.get_mut(&hash) {
            if service.resolved == resolved {
                service.seen = epoch;
                return Ok(());
            }
            trace!(from origin, "Closing the {:?} bridge of \"{}\", its resolution changed", service.pattern, service.name);
            self.services.remove(&hash);
        }
        let pattern = local_description.settings().pattern.messaging_pattern();
        let bridge = fail!(
            from origin,
            when B::open(node, backend, local_description, remote_description),
            "Failed to open the {:?} bridge of \"{}\"", pattern, local_description.name()
        );
        let service = BridgedService {
            bridge,
            name: local_description.name(),
            pattern,
            resolved,
            seen: epoch,
            failure: None,
        };

        trace!(from origin, "Opened {:?} bridge of \"{}\"", service.pattern, service.name);

        self.services.insert(hash, service);
        Ok(())
    }

    /// Closes the bridges not seen in `epoch`.
    pub(super) fn retain(&mut self, epoch: Epoch) {
        let origin = origin!("BridgeTable::retain");

        self.services.retain(|_, service| {
            let keep = service.seen == epoch;
            if !keep {
                trace!(from origin, "Closing the {:?} bridge of \"{}\"", service.pattern, service.name);
            }
            keep
        });
    }

    pub(super) fn contains(&self, hash: &ServiceHash) -> bool {
        self.services.contains_key(hash)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.services.is_empty()
    }

    /// Moves what is pending on every bridge in both directions, in hash
    /// order. A failing bridge does not stop the others, the number of
    /// failures are returned in the error.
    pub(super) fn propagate(
        &mut self,
        node: &Node<B::Service>,
        notify: bool,
    ) -> Result<(), PropagateError> {
        let mut failures = 0;
        for service in self.services.values_mut() {
            service.failure = service.bridge.propagate(node, notify).err();
            failures += usize::from(service.failure.is_some());
        }
        match failures {
            0 => Ok(()),
            n => Err(PropagateError::Bridges(n)),
        }
    }

    /// The bridges whose last propagation failed, with why.
    pub(super) fn failed(&self) -> impl Iterator<Item = (&ServiceHash, PropagationFailure)> {
        self.services.iter().filter_map(|(hash, service)| {
            service.failure.map(|error| {
                let failure = PropagationFailure {
                    name: service.name,
                    pattern: service.pattern,
                    error,
                };
                (hash, failure)
            })
        })
    }

    pub(super) fn len(&self) -> usize {
        self.services.len()
    }

    /// Logs what every bridge moved since the last report.
    pub(super) fn report(&mut self) {
        let origin = origin!("BridgeTable::report");

        for service in self.services.values_mut() {
            let (outbound, inbound) = service.bridge.monitoring().take();
            if outbound != 0 || inbound != 0 {
                trace!(
                    from origin,
                    "Relayed {:?} bridge of \"{}\" ({} out, {} in)",
                    service.pattern, service.name, outbound, inbound
                );
            }
        }
    }
}
