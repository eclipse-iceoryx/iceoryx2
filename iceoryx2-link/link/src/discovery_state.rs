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

/// What identifies one creation of a local service. `UniqueServiceId`
/// once it can be generated outside iceoryx2.
pub(crate) type CreationId = u128;
type Unresolvable<'a, Id, Refusal> = iceoryx2_link_backend::diagnostic::Update<
    Id,
    Refusal,
    &'a mut Findings<Id, Refusal>,
    fn(&Id, &Refusal),
>;

/// A local service as the last local update saw it.
struct Local {
    description: ServiceDescription,
    /// A recreated service has a new one.
    id: CreationId,
    /// The epoch of the update that last saw the service.
    seen: Epoch,
}

/// A remote description as the last remote update saw it.
pub(crate) struct Remote<Description> {
    description: Description,
    /// The epoch of the update that last saw the description.
    seen: Epoch,
}

/// Everything discovered about one service, and what it resolves to.
/// The remote descriptions of an entry, in id order.
pub(crate) type Remotes<'a, Id, Description> = core::iter::Map<
    alloc::collections::btree_map::Values<'a, Id, Remote<Description>>,
    fn(&'a Remote<Description>) -> &'a Description,
>;

pub(crate) struct Entry<Id, Description, Refusal> {
    local: Option<Local>,
    remotes: BTreeMap<Id, Remote<Description>>,
    resolution: Resolution<Description, Refusal>,
    /// Whether a side of the entry changed since it was last resolved.
    changed: bool,
    /// The epoch the current resolution was decided in.
    decided: Epoch,
}

impl<Id, Description, Refusal> Entry<Id, Description, Refusal> {
    fn new() -> Self {
        Self {
            local: None,
            remotes: BTreeMap::new(),
            resolution: Resolution::OutOfScope,
            changed: true,
            decided: Epoch::default(),
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
    pub(crate) fn remotes(&self) -> Remotes<'_, Id, Description> {
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

    fn is_empty(&self) -> bool {
        self.local.is_none() && self.remotes.is_empty()
    }
}

/// What was discovered on both sides of the boundary and what each
/// service resolves to.
pub(crate) struct DiscoveryState<Id, Description, Refusal> {
    entries: BTreeMap<ServiceHash, Entry<Id, Description, Refusal>>,
    /// The remote descriptions listed, each under the service it belongs
    /// to.
    listed: BTreeMap<Id, ServiceHash>,
    /// The remote descriptions in scope that belong to no service.
    unresolvable: Findings<Id, Refusal>,
    /// The generation of the local listing last updated from.
    local_generation: Generation,
    /// The generation of the remote listing last updated from.
    remote_generation: Generation,
    /// Moved by every update, the stamp of what a update saw and of what
    /// changed since an entry was resolved.
    epoch: Epoch,
}

impl<Id, Description, Refusal> Default for DiscoveryState<Id, Description, Refusal> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            listed: BTreeMap::new(),
            unresolvable: Findings::default(),
            local_generation: Generation::Untracked,
            remote_generation: Generation::Untracked,
            epoch: Epoch::default(),
        }
    }
}

impl<Id: Ord + Clone + Display, Description: PartialEq, Refusal: Display + PartialEq>
    DiscoveryState<Id, Description, Refusal>
{
    /// Begins an update from the local system's listing at `generation`, none
    /// if it has not moved.
    pub(crate) fn update_locals(
        &mut self,
        generation: Generation,
    ) -> Option<LocalUpdate<'_, Id, Description, Refusal>> {
        if generation.unchanged_since(self.local_generation) {
            return None;
        }
        self.epoch = self.epoch.next();
        Some(LocalUpdate {
            entries: &mut self.entries,
            previous: &mut self.local_generation,
            current: generation,
            epoch: self.epoch,
        })
    }

    /// Begins an update from the backend's listing at `generation`, none if it
    /// has not moved.
    pub(crate) fn update_remotes(
        &mut self,
        generation: Generation,
    ) -> Option<RemoteUpdate<'_, Id, Description, Refusal>> {
        if generation.unchanged_since(self.remote_generation) {
            return None;
        }
        self.epoch = self.epoch.next();
        let epoch = self.epoch;
        Some(RemoteUpdate {
            entries: &mut self.entries,
            listed: &mut self.listed,
            unresolvable: self.unresolvable.update(warn_unresolvable),
            previous: &mut self.remote_generation,
            current: generation,
            epoch,
        })
    }

    /// Iterates the changes since the entries were last resolved, one per
    /// entry a side of which changed, each to have a resolution applied.
    pub(crate) fn changes(&mut self) -> impl Iterator<Item = Change<'_, Id, Description, Refusal>> {
        let epoch = self.epoch;
        self.entries
            .iter_mut()
            .filter(|(_, entry)| entry.changed)
            .map(move |(hash, entry)| Change { hash, entry, epoch })
    }

    /// Iterates the services that resolved to a bridge, in hash order,
    /// each with the description it is bridged as on this side, the one
    /// it is opened as on the opposing side, and the epoch its
    /// resolution was decided in.
    pub(crate) fn bridgeable(
        &self,
    ) -> impl Iterator<Item = (&ServiceHash, &ServiceDescription, &Description, Epoch)> {
        self.entries
            .iter()
            .filter_map(|(hash, entry)| match &entry.resolution {
                Resolution::Exported(remote) => {
                    let local = entry.local.as_ref()?;
                    Some((hash, &local.description, remote, entry.decided))
                }
                Resolution::Imported(mirror, remote) => Some((hash, mirror, remote, entry.decided)),
                _ => None,
            })
    }

    /// Iterates the exported local services with their creation ids,
    /// in hash order.
    pub(crate) fn exported(&self) -> impl Iterator<Item = (&ServiceDescription, CreationId)> {
        self.entries.values().filter_map(|entry| {
            let local = entry.local.as_ref()?;
            matches!(entry.resolution, Resolution::Exported(_))
                .then_some((&local.description, local.id))
        })
    }
}

/// One update from the local system's listing. Dropped without being
/// finalized, nothing is pruned and the generation stands.
pub(crate) struct LocalUpdate<'a, Id, Description, Refusal> {
    entries: &'a mut BTreeMap<ServiceHash, Entry<Id, Description, Refusal>>,
    /// The generation of the last update from the listing. Set to `current`
    /// when the update finalizes, left as it is when the update is dropped.
    previous: &'a mut Generation,
    /// The generation this update is at.
    current: Generation,
    /// The state's epoch, what this update sees is stamped with it.
    epoch: Epoch,
}

impl<Id, Description, Refusal> LocalUpdate<'_, Id, Description, Refusal> {
    /// Whether the service is held under this unique id, in which case it
    /// is seen in this update. A recreated service has a new id and is to
    /// be inserted anew.
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

    /// Holds the local service, seen in this update, replacing a previous
    /// description of it.
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
            .or_insert_with(Entry::new);
        entry.local = Some(Local {
            description,
            id,
            seen: epoch,
        });
        entry.changed = true;
    }

    /// Finalizes the update. A local service not seen is removed, and the
    /// listing is at the generation updated from.
    pub(crate) fn finalize(self) {
        let epoch = self.epoch;
        for entry in self.entries.values_mut() {
            if entry
                .local
                .as_ref()
                .is_some_and(|local| local.seen != epoch)
            {
                entry.local = None;
                entry.changed = true;
            }
        }
        self.entries.retain(|_, entry| !entry.is_empty());
        *self.previous = self.current;
    }
}

/// One update from the backend's listing. Dropped without being finalized,
/// nothing is pruned and the generation stands.
pub(crate) struct RemoteUpdate<'a, Id, Description, Refusal> {
    entries: &'a mut BTreeMap<ServiceHash, Entry<Id, Description, Refusal>>,
    listed: &'a mut BTreeMap<Id, ServiceHash>,
    unresolvable: Unresolvable<'a, Id, Refusal>,
    /// The generation of the last update from the listing. Set to `current`
    /// when the update finalizes, left as it is when the update is dropped.
    previous: &'a mut Generation,
    /// The generation this update is at.
    current: Generation,
    /// The state's epoch, what this update sees is stamped with it.
    epoch: Epoch,
}

impl<Id: Ord + Clone + Display, Description: PartialEq, Refusal: Display + PartialEq>
    RemoteUpdate<'_, Id, Description, Refusal>
{
    /// Whether the remote is held with this same description, in which
    /// case it is seen in this update.
    pub(crate) fn seen(&mut self, id: &Id, remote: &Description) -> bool {
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

    /// Holds the remote description under the service it belongs to,
    /// seen in this update, replacing a previous description of it.
    pub(crate) fn insert(&mut self, hash: ServiceHash, id: Id, remote: Description) {
        let origin = origin!("RemoteUpdate::insert");

        trace!(from origin, "Discovered the service {} on {}", hash.as_str(), id);
        let epoch = self.epoch;
        if let Some(previous) = self.listed.insert(id.clone(), hash)
            && previous != hash
            && let Some(entry) = self.entries.get_mut(&previous)
        {
            entry.remotes.remove(&id);
            entry.changed = true;
        }
        let entry = self.entries.entry(hash).or_insert_with(Entry::new);
        entry.remotes.insert(
            id,
            Remote {
                description: remote,
                seen: epoch,
            },
        );
        entry.changed = true;
    }

    /// Records a remote description in scope that belongs to no service,
    /// warned about once.
    pub(crate) fn refuse(&mut self, id: Id, refusal: Refusal) {
        self.unresolvable.record(id, refusal);
    }

    /// Finalizes the update. A remote description not seen is removed, and
    /// the listing is at the generation updated from.
    pub(crate) fn finalize(self) {
        let epoch = self.epoch;
        for entry in self.entries.values_mut() {
            let before = entry.remotes.len();
            entry.remotes.retain(|_, remote| remote.seen == epoch);
            if entry.remotes.len() != before {
                entry.changed = true;
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

fn warn_unresolvable<Id: core::fmt::Display, Refusal: core::fmt::Display>(
    id: &Id,
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
pub(crate) struct Change<'a, Id, Description, Refusal> {
    hash: &'a ServiceHash,
    entry: &'a mut Entry<Id, Description, Refusal>,
    /// The state's epoch, a resolution is decided in it.
    epoch: Epoch,
}

impl<Id, Description: PartialEq, Refusal: core::fmt::Display + PartialEq>
    Change<'_, Id, Description, Refusal>
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
            Remotes<'_, Id, Description>,
        ) -> Resolution<Description, Refusal>,
    {
        let origin = origin!("Change::resolve");

        let resolution = resolve(self.entry.local(), self.entry.remotes());
        self.entry.changed = false;
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
        self.entry.decided = self.epoch;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec::Vec;

    use iceoryx2::config::Config;
    use iceoryx2::service::local;
    use iceoryx2::service::service_name::ServiceName;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_backend::service_description::ServiceTypes;

    /// A remote description, belonging to the service it names.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct FakeRemote {
        service: ServiceHash,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct FakeRefusal;

    impl core::fmt::Display for FakeRefusal {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "is refused")
        }
    }

    type Sut = DiscoveryState<u8, FakeRemote, FakeRefusal>;

    const FIRST_CREATION: u8 = 1;
    const SECOND_CREATION: u8 = 2;
    const FIRST_REMOTE: u8 = 1;
    const FIRST_GENERATION: Generation = Generation::At(1);
    const SECOND_GENERATION: Generation = Generation::At(2);

    fn description(name: &str) -> ServiceDescription {
        ServiceDescription::with_default_settings::<local::Service>(
            ServiceName::new(name).expect("valid service name"),
            ServiceTypes::Event,
            &Config::default(),
        )
    }

    /// The id of the `n`th creation of a service.
    fn creation_id(n: u8) -> CreationId {
        CreationId::from(n)
    }

    fn remote(service: &ServiceDescription) -> FakeRemote {
        FakeRemote {
            service: service.hash(),
        }
    }

    /// Updates from a local listing at `generation` of these services, each
    /// under its id.
    fn update_from_locals(
        sut: &mut Sut,
        generation: Generation,
        locals: &[(u8, &ServiceDescription)],
    ) {
        let mut update = sut.update_locals(generation).expect("the generation moved");
        for (id, description) in locals {
            if update.seen(&description.hash(), creation_id(*id)) {
                continue;
            }
            update.insert(creation_id(*id), (*description).clone());
        }
        update.finalize();
    }

    /// Updates from a backend listing at `generation` of these remotes.
    fn update_from_remotes(sut: &mut Sut, generation: Generation, remotes: &[(u8, FakeRemote)]) {
        let mut update = sut
            .update_remotes(generation)
            .expect("the generation moved");
        for (id, remote) in remotes {
            if update.seen(id, remote) {
                continue;
            }
            update.insert(remote.service, *id, remote.clone());
        }
        update.finalize();
    }

    /// The hashes of the entries offered for resolution, each resolved
    /// to `resolution`.
    fn resolve_to(
        sut: &mut Sut,
        resolution: Resolution<FakeRemote, FakeRefusal>,
    ) -> Vec<ServiceHash> {
        let mut resolved = Vec::new();
        for change in sut.changes() {
            resolved.push(*change.hash());
            change.resolve(|_, _| resolution.clone());
        }
        resolved
    }

    fn held(sut: &Sut, service: &ServiceDescription) -> bool {
        sut.entries.contains_key(&service.hash())
    }

    #[test]
    fn a_local_service_updated_in_is_held() {
        const SERVICE: &str = "state/local";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);

        let entry = sut.entries.get(&service.hash()).expect("held");
        assert_that!(entry.local(), eq Some(&service));
        assert_that!(entry.creation_id(), eq Some(creation_id(FIRST_CREATION)));
    }

    #[test]
    fn a_local_service_seen_again_keeps_its_creation_id() {
        const SERVICE: &str = "state/local/seen";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);
        update_from_locals(&mut sut, SECOND_GENERATION, &[(FIRST_CREATION, &service)]);

        let entry = sut.entries.get(&service.hash()).expect("held");
        assert_that!(entry.creation_id(), eq Some(creation_id(FIRST_CREATION)));
    }

    #[test]
    fn a_recreated_local_service_takes_its_new_creation_id() {
        const SERVICE: &str = "state/local/recreated";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);
        update_from_locals(&mut sut, SECOND_GENERATION, &[(SECOND_CREATION, &service)]);

        let entry = sut.entries.get(&service.hash()).expect("held");
        assert_that!(entry.creation_id(), eq Some(creation_id(SECOND_CREATION)));
    }

    #[test]
    fn a_local_service_not_seen_again_is_gone() {
        const SERVICE: &str = "state/local/gone";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);
        update_from_locals(&mut sut, SECOND_GENERATION, &[]);

        assert_that!(held(&sut, &service), eq false);
    }

    #[test]
    fn locals_are_updated_only_when_the_generation_moves() {
        let mut sut = Sut::default();
        update_from_locals(&mut sut, FIRST_GENERATION, &[]);

        assert_that!(sut.update_locals(FIRST_GENERATION).is_none(), eq true);
        assert_that!(sut.update_locals(SECOND_GENERATION).is_some(), eq true);
    }

    #[test]
    fn a_remote_updated_in_is_held_under_its_service() {
        const SERVICE: &str = "state/remote";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        let remote = remote(&service);
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote.clone())],
        );

        let entry = sut.entries.get(&service.hash()).expect("held");
        assert_that!(entry.remotes().collect::<Vec<_>>(), eq alloc::vec![&remote]);
        assert_that!(entry.local(), is_none);
    }

    #[test]
    fn a_remote_not_listed_again_is_gone() {
        const SERVICE: &str = "state/remote/gone";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );
        update_from_remotes(&mut sut, SECOND_GENERATION, &[]);

        assert_that!(held(&sut, &service), eq false);
    }

    #[test]
    fn a_remote_listed_under_another_service_moves_there() {
        const SERVICE: &str = "state/remote/moved/from";
        const OTHER_SERVICE: &str = "state/remote/moved/to";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        let other = description(OTHER_SERVICE);
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );
        update_from_remotes(
            &mut sut,
            SECOND_GENERATION,
            &[(FIRST_REMOTE, remote(&other))],
        );

        assert_that!(held(&sut, &service), eq false);
        assert_that!(held(&sut, &other), eq true);
    }

    #[test]
    fn remotes_are_updated_only_when_the_generation_moves() {
        let mut sut = Sut::default();
        update_from_remotes(&mut sut, FIRST_GENERATION, &[]);

        assert_that!(sut.update_remotes(FIRST_GENERATION).is_none(), eq true);
        assert_that!(sut.update_remotes(SECOND_GENERATION).is_some(), eq true);
    }

    #[test]
    fn a_remote_update_dropped_prunes_nothing() {
        const SERVICE: &str = "state/remote/dropped";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );
        sut.update_remotes(SECOND_GENERATION)
            .expect("the generation moved");

        assert_that!(held(&sut, &service), eq true);
        assert_that!(sut.update_remotes(SECOND_GENERATION).is_some(), eq true);
    }

    #[test]
    fn an_entry_a_side_of_which_changed_is_offered_for_resolution_once() {
        const SERVICE: &str = "state/changed";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);

        let first = resolve_to(&mut sut, Resolution::OutOfScope);
        let second = resolve_to(&mut sut, Resolution::OutOfScope);

        assert_that!(first, eq alloc::vec![service.hash()]);
        assert_that!(second, len 0);
    }

    #[test]
    fn an_entry_is_offered_again_when_a_remote_joins() {
        const SERVICE: &str = "state/changed/remote";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);
        resolve_to(&mut sut, Resolution::OutOfScope);
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );

        let offered = resolve_to(&mut sut, Resolution::OutOfScope);

        assert_that!(offered, eq alloc::vec![service.hash()]);
    }

    #[test]
    fn an_entry_is_offered_again_when_its_local_service_goes() {
        const SERVICE: &str = "state/changed/local/gone";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);
        resolve_to(&mut sut, Resolution::OutOfScope);
        update_from_locals(&mut sut, SECOND_GENERATION, &[]);

        let offered = resolve_to(&mut sut, Resolution::OutOfScope);

        assert_that!(offered, eq alloc::vec![service.hash()]);
    }

    #[test]
    fn a_local_service_equal_to_the_imported_mirror_is_not_an_applications() {
        const SERVICE: &str = "state/mirror/own";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );
        resolve_to(
            &mut sut,
            Resolution::Imported(service.clone(), remote(&service)),
        );
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);

        let entry = sut.entries.get(&service.hash()).expect("held");
        assert_that!(entry.local(), is_none);
    }

    #[test]
    fn an_entry_resolved_to_the_same_resolution_keeps_its_decided_epoch() {
        const SERVICE: &str = "state/resolution/unchanged";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);
        resolve_to(&mut sut, Resolution::Exported(remote(&service)));
        let before = sut.bridgeable().map(|(.., epoch)| epoch).next();
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );
        resolve_to(&mut sut, Resolution::Exported(remote(&service)));

        let after = sut.bridgeable().map(|(.., epoch)| epoch).next();
        assert_that!(after, eq before);
    }

    #[test]
    fn exported_and_imported_entries_are_bridgeable() {
        const EXPORTED_SERVICE: &str = "state/bridgeable/exported";
        const IMPORTED_SERVICE: &str = "state/bridgeable/imported";
        const REFUSED_SERVICE: &str = "state/bridgeable/refused";

        let mut sut = Sut::default();
        let exported = description(EXPORTED_SERVICE);
        let imported = description(IMPORTED_SERVICE);
        let refused = description(REFUSED_SERVICE);
        update_from_locals(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_CREATION, &exported), (SECOND_CREATION, &refused)],
        );
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&imported))],
        );
        for change in sut.changes() {
            let resolution = match *change.hash() {
                hash if hash == exported.hash() => Resolution::Exported(remote(&exported)),
                hash if hash == imported.hash() => {
                    Resolution::Imported(imported.clone(), remote(&imported))
                }
                _ => Resolution::Refused(FakeRefusal),
            };
            change.resolve(|_, _| resolution);
        }

        let bridgeable: Vec<ServiceHash> = sut.bridgeable().map(|(hash, ..)| *hash).collect();
        let exported_hashes: Vec<ServiceHash> = sut
            .exported()
            .map(|(description, _)| description.hash())
            .collect();

        let mut expected = alloc::vec![exported.hash(), imported.hash()];
        expected.sort();
        assert_that!(bridgeable, eq expected);
        assert_that!(exported_hashes, eq alloc::vec![exported.hash()]);
    }
}
