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
use iceoryx2_link_backend::origin;
use iceoryx2_link_backend::{Epoch, RemoteDescription};
use iceoryx2_log::{debug, fail};

use crate::bridge::{Bridged, OpenError};

/// One bridge, stamped with the resolution it was opened for and the
/// reconcile that last saw it.
struct Bridge<B> {
    bridged: B,
    name: ServiceName,
    pattern: MessagingPattern,
    /// The epoch the service was resolved in when the bridge was opened.
    resolved: Epoch,
    /// The epoch of the reconcile that last saw the service bridgeable.
    seen: Epoch,
}

/// The bridges of one pattern, keyed by service.
pub(super) struct BridgeTable<B> {
    bridges: BTreeMap<ServiceHash, Bridge<B>>,
}

impl<B> Default for BridgeTable<B> {
    fn default() -> Self {
        Self {
            bridges: BTreeMap::new(),
        }
    }
}

impl<B: Bridged> BridgeTable<B> {
    /// Stamps the bridge of a bridgeable service, opening it first if the
    /// service has none or was resolved anew since its bridge was opened.
    pub(super) fn reconcile(
        &mut self,
        node: &Node<B::Service>,
        backend: &B::Backend,
        description: B::Pattern<'_>,
        remote: &RemoteDescription<B::Service, B::Backend>,
        resolved: Epoch,
        epoch: Epoch,
    ) -> Result<(), OpenError> {
        let origin = origin!("BridgeTable::reconcile");

        let hash = description.hash();
        if let Some(bridge) = self.bridges.get_mut(&hash) {
            if bridge.resolved == resolved {
                bridge.seen = epoch;
                return Ok(());
            }
            debug!(from origin, "Closing the {:?} bridge of {}, its resolution changed", bridge.pattern, bridge.name);
            self.bridges.remove(&hash);
        }
        let pattern = description.settings().pattern.messaging_pattern();
        let bridged = fail!(
            from origin,
            when B::open(node, backend, description, remote),
            "Failed to open the {:?} bridge of {}", pattern, description.name()
        );
        let bridge = Bridge {
            bridged,
            name: description.name(),
            pattern,
            resolved,
            seen: epoch,
        };
        debug!(from origin, "Opened {:?} bridge of {}", bridge.pattern, bridge.name);
        self.bridges.insert(hash, bridge);
        Ok(())
    }

    /// Closes the bridges not seen in `epoch`.
    pub(super) fn retain(&mut self, epoch: Epoch) {
        let origin = origin!("BridgeTable::retain");

        self.bridges.retain(|_, bridge| {
            let keep = bridge.seen == epoch;
            if !keep {
                debug!(from origin, "Closing the {:?} bridge of {}", bridge.pattern, bridge.name);
            }
            keep
        });
    }

    pub(super) fn contains(&self, hash: &ServiceHash) -> bool {
        self.bridges.contains_key(hash)
    }

    pub(super) fn is_empty(&self) -> bool {
        self.bridges.is_empty()
    }

    /// Iterates the bridges in hash order.
    pub(super) fn bridged(&self) -> impl Iterator<Item = &B> {
        self.bridges.values().map(|bridge| &bridge.bridged)
    }
}
