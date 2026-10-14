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
use core::fmt::Display;

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_elementary::epoch::Epoch;
use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::diagnostic::Findings;
use iceoryx2_link_backend::resolver::Resolution;
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_log::{origin, trace, warn};

use crate::discovery::discovered_service::{
    CreationId, DiscoveredService, LocalObservation, RemoteDescriptions, RemoteObservation,
};

pub(super) type Unresolvable<'a, RemoteId, Refusal> = iceoryx2_link_backend::diagnostic::Update<
    RemoteId,
    Refusal,
    &'a mut Findings<RemoteId, Refusal>,
    fn(&RemoteId, &Refusal),
>;

/// One update from the local system's listing. Finalizing the update prunes
/// the local services it did not see and marks the listing's generation as
/// processed.
pub(crate) struct LocalUpdate<'a, RemoteId, RemoteDescription, Refusal> {
    pub(super) entries:
        &'a mut BTreeMap<ServiceHash, DiscoveredService<RemoteId, RemoteDescription, Refusal>>,
    /// The generation of the last update from the listing. Set to `current`
    /// when the update finalizes, left as it is when the update is dropped.
    pub(super) previous: &'a mut Generation,
    /// The generation this update is at.
    pub(super) current: Generation,
    /// The state's epoch, what this update sees is stamped with it.
    pub(super) epoch: Epoch,
}

impl<RemoteId, RemoteDescription, Refusal> LocalUpdate<'_, RemoteId, RemoteDescription, Refusal> {
    /// Marks the local service as seen in this update when it is held under
    /// this creation id. A recreated service has a new id and counts as unseen.
    pub(crate) fn seen(&mut self, hash: &ServiceHash, id: CreationId) -> bool {
        let epoch = self.epoch;
        let Some(local) = self
            .entries
            .get_mut(hash)
            .and_then(|entry| entry.local.as_mut())
        else {
            return false;
        };
        if local.id != id {
            return false;
        }
        local.seen = epoch;
        true
    }

    /// Records the local service as seen in this update, with this
    /// description under this creation id.
    pub(crate) fn insert(&mut self, id: CreationId, description: ServiceDescription) {
        let origin = origin!("LocalUpdate::insert");

        trace!(
            from origin,
            "Discovered the local {:?} service {} ({})",
            description.settings().pattern.messaging_pattern(),
            description.name(),
            description.hash().as_str()
        );
        let epoch = self.epoch;
        let entry = self
            .entries
            .entry(description.hash())
            .or_insert_with(DiscoveredService::new);
        entry.local = Some(LocalObservation {
            description,
            id,
            seen: epoch,
        });
        entry.stale = true;
    }

    /// Prunes every local service not seen in this update and marks the
    /// listing's generation as processed.
    pub(crate) fn finalize(self) {
        let epoch = self.epoch;
        for entry in self.entries.values_mut() {
            if entry
                .local
                .as_ref()
                .is_some_and(|local| local.seen != epoch)
            {
                entry.local = None;
                entry.stale = true;
            }
        }
        self.entries.retain(|_, entry| !entry.is_empty());
        *self.previous = self.current;
    }
}

/// One update from the backend's listing. Finalizing the update prunes the
/// remotes it did not see and marks the listing's generation as processed.
pub(crate) struct RemoteUpdate<'a, RemoteId, RemoteDescription, Refusal> {
    pub(super) entries:
        &'a mut BTreeMap<ServiceHash, DiscoveredService<RemoteId, RemoteDescription, Refusal>>,
    pub(super) listed: &'a mut BTreeMap<RemoteId, ServiceHash>,
    pub(super) unresolvable: Unresolvable<'a, RemoteId, Refusal>,
    /// The generation of the last update from the listing. Set to `current`
    /// when the update finalizes, left as it is when the update is dropped.
    pub(super) previous: &'a mut Generation,
    /// The generation this update is at.
    pub(super) current: Generation,
    /// The state's epoch, what this update sees is stamped with it.
    pub(super) epoch: Epoch,
}

impl<RemoteId: Ord + Clone + Display, RemoteDescription: PartialEq, Refusal: Display + PartialEq>
    RemoteUpdate<'_, RemoteId, RemoteDescription, Refusal>
{
    /// Marks the remote as seen in this update when it is held with this
    /// description. A remote that changed its description counts as unseen.
    pub(crate) fn seen(&mut self, id: &RemoteId, remote: &RemoteDescription) -> bool {
        let epoch = self.epoch;
        let Some(hash) = self.listed.get(id) else {
            return false;
        };
        let Some(held) = self
            .entries
            .get_mut(hash)
            .and_then(|entry| entry.remotes.get_mut(id))
        else {
            return false;
        };
        if held.description != *remote {
            return false;
        }
        held.seen = epoch;
        true
    }

    /// Records the remote as seen in this update, with this description
    /// under this service.
    pub(crate) fn insert(&mut self, hash: ServiceHash, id: RemoteId, remote: RemoteDescription) {
        let origin = origin!("RemoteUpdate::insert");

        trace!(from origin, "Discovered the service {} on {}", hash.as_str(), id);
        let epoch = self.epoch;
        if let Some(previous) = self.listed.insert(id.clone(), hash)
            && previous != hash
            && let Some(entry) = self.entries.get_mut(&previous)
        {
            entry.remotes.remove(&id);
            entry.stale = true;
        }
        let entry = self
            .entries
            .entry(hash)
            .or_insert_with(DiscoveredService::new);
        entry.remotes.insert(
            id,
            RemoteObservation {
                description: remote,
                seen: epoch,
            },
        );
        entry.stale = true;
    }

    /// Records a remote description in scope that belongs to no service,
    /// warned about once.
    pub(crate) fn refuse(&mut self, id: RemoteId, refusal: Refusal) {
        self.unresolvable.record(id, refusal);
    }

    /// Prunes every remote not seen in this update and marks the listing's
    /// generation as processed.
    pub(crate) fn finalize(self) {
        let epoch = self.epoch;
        for entry in self.entries.values_mut() {
            let before = entry.remotes.len();
            entry.remotes.retain(|_, remote| remote.seen == epoch);
            if entry.remotes.len() != before {
                entry.stale = true;
            }
        }
        self.entries.retain(|_, entry| !entry.is_empty());
        let entries = &*self.entries;
        self.listed.retain(|id, hash| {
            entries
                .get(hash)
                .is_some_and(|entry| entry.remotes.contains_key(id))
        });
        self.unresolvable.commit();
        *self.previous = self.current;
    }
}

pub(super) fn warn_unresolvable<RemoteId: core::fmt::Display, Refusal: core::fmt::Display>(
    id: &RemoteId,
    refusal: &Refusal,
) {
    let origin = origin!("RemoteUpdate::finalize");

    warn!(
        from origin,
        "Remote {} belongs to no service, it {}", id, refusal
    );
}

/// An entry a side of which changed since it was last resolved, to be
/// resolved anew.
pub(crate) struct Unresolved<'a, RemoteId, RemoteDescription, Refusal> {
    pub(super) hash: &'a ServiceHash,
    pub(super) entry: &'a mut DiscoveredService<RemoteId, RemoteDescription, Refusal>,
    /// The state's epoch, a resolution is decided in it.
    pub(super) epoch: Epoch,
}

impl<RemoteId, RemoteDescription: PartialEq, Refusal: core::fmt::Display + PartialEq>
    Unresolved<'_, RemoteId, RemoteDescription, Refusal>
{
    #[cfg(test)]
    pub(crate) fn hash(&self) -> &ServiceHash {
        self.hash
    }

    /// Resolves the entry with `resolve`, from its local description and
    /// its remote descriptions, and records the resolution. A refusal is
    /// warned about when it is new.
    pub(crate) fn resolve<R>(self, resolve: R)
    where
        R: FnOnce(
            Option<&ServiceDescription>,
            RemoteDescriptions<'_, RemoteId, RemoteDescription>,
        ) -> Resolution<RemoteDescription, Refusal>,
    {
        let origin = origin!("Unresolved::resolve");

        let resolution = resolve(
            self.entry.local_description(),
            self.entry.remote_descriptions(),
        );
        self.entry.stale = false;
        if self.entry.resolution == resolution {
            return;
        }
        let hash = self.hash.as_str();
        match &resolution {
            Resolution::Exported(_) => {
                match self.entry.local.as_ref().map(|local| &local.description) {
                    Some(local) => trace!(
                        from origin,
                        "{:?} service {} ({}) is exported",
                        local.settings().pattern.messaging_pattern(),
                        local.name(),
                        hash
                    ),
                    None => trace!(from origin, "Service {} is exported", hash),
                }
            }
            Resolution::Imported(mirror, _) => {
                trace!(
                    from origin,
                    "{:?} service {} ({}) is imported",
                    mirror.settings().pattern.messaging_pattern(),
                    mirror.name(),
                    hash
                );
            }
            Resolution::OutOfScope => {
                trace!(from origin, "Service {} is out of scope", hash);
            }
            Resolution::Refused(refusal) => warn!(
                from origin,
                "Service {} is not bridged, it {}", hash, refusal
            ),
        }
        self.entry.resolution = resolution;
        self.entry.resolved = self.epoch;
    }
}
