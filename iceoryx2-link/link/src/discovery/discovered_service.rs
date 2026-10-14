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
use alloc::collections::btree_map::Values;
use core::iter::Map;

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_elementary::epoch::Epoch;
use iceoryx2_link_backend::resolver::Resolution;
use iceoryx2_link_backend::service_description::ServiceDescription;

/// What identifies one creation of a local service. `UniqueServiceId`
/// once it can be generated outside iceoryx2.
pub(crate) type CreationId = u128;

/// What the last local update observed of one service.
pub(super) struct LocalObservation {
    pub(super) description: ServiceDescription,
    /// A recreated service has a new one.
    pub(super) id: CreationId,
    /// The epoch of the update that last saw the service.
    pub(super) seen: Epoch,
}

/// What the last remote update observed of one description.
pub(crate) struct RemoteObservation<RemoteDescription> {
    pub(super) description: RemoteDescription,
    /// The epoch of the update that last saw the description.
    pub(super) seen: Epoch,
}

/// The descriptions of the remotes of an entry, in id order.
pub(crate) type RemoteDescriptions<'a, RemoteId, RemoteDescription> = Map<
    Values<'a, RemoteId, RemoteObservation<RemoteDescription>>,
    fn(&'a RemoteObservation<RemoteDescription>) -> &'a RemoteDescription,
>;

/// A service discovered locally or remotely and how it was resolved.
pub(crate) struct DiscoveredService<RemoteId, RemoteDescription, Refusal> {
    pub(super) local: Option<LocalObservation>,
    pub(super) remotes: BTreeMap<RemoteId, RemoteObservation<RemoteDescription>>,
    pub(super) resolution: Resolution<RemoteDescription, Refusal>,
    /// Whether the resolution is stale, a side changed since it was decided.
    pub(super) stale: bool,
    /// The epoch the resolution was decided in.
    pub(super) resolved: Epoch,
}

/// A discovered service for which a bridge can be established.
pub(crate) struct BridgeableService<'a, RemoteDescription> {
    pub(crate) hash: &'a ServiceHash,
    pub(crate) local_description: &'a ServiceDescription,
    /// The remote the service is relayed against.
    pub(crate) remote_description: &'a RemoteDescription,
    /// The epoch the resolution was decided in.
    pub(crate) resolved: Epoch,
}

impl<RemoteId, RemoteDescription, Refusal> DiscoveredService<RemoteId, RemoteDescription, Refusal> {
    pub(super) fn new() -> Self {
        Self {
            local: None,
            remotes: BTreeMap::new(),
            resolution: Resolution::OutOfScope,
            stale: true,
            resolved: Epoch::default(),
        }
    }

    /// The local service an application offers, none when the only
    /// local service is the mirror the entry imported, listed since an
    /// application opened a port on it.
    pub(crate) fn local(&self) -> Option<&ServiceDescription> {
        if self.local_is_mirror() {
            return None;
        }
        self.local.as_ref().map(|local| &local.description)
    }

    /// The id of the local service's creation, a recreated service has
    /// a new one.
    #[cfg(test)]
    pub(crate) fn creation_id(&self) -> Option<CreationId> {
        self.local.as_ref().map(|local| local.id)
    }

    /// The remote descriptions of the service, in id order.
    pub(crate) fn remotes(&self) -> RemoteDescriptions<'_, RemoteId, RemoteDescription> {
        self.remotes.values().map(|remote| &remote.description)
    }

    /// Whether the local service is the mirror the entry imported, listed
    /// as local since an application opened a port on it.
    fn local_is_mirror(&self) -> bool {
        matches!(
            (&self.local, &self.resolution),
            (Some(local), Resolution::Imported(mirror, _)) if local.description == *mirror
        )
    }

    pub(super) fn is_empty(&self) -> bool {
        self.local.is_none() && self.remotes.is_empty()
    }
}
