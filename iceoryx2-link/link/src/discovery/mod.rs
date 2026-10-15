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

//! Discovers the services of both sides, resolves how each is bridged and
//! announces the exported ones to the opposing side.
//!
//! ```text
//!   local listing, one pass                      remote listing, one pass
//!             │                                             │
//!             ▼                                             ▼
//!   ┌────────────────────────┐                  ┌────────────────────────┐
//!   │ LocalUpdate            │                  │ RemoteUpdate           │
//!   │   insert() per service │                  │   insert() per remote  │
//!   │   finalize() prunes    │                  │   finalize() prunes    │
//!   │   what was not seen    │                  │   what was not seen    │
//!   └────────────────────────┘                  └────────────────────────┘
//!             │  records                                    │  records
//!             ▼                                             ▼
//!   ┌────────────────────────┐                  ┌────────────────────────┐
//!   │ LocalObservation       │                  │ RemoteObservation      │
//!   │   one per listed       │                  │   one per listed       │
//!   │   service              │                  │   remote description   │
//!   └────────────────────────┘                  └────────────────────────┘
//!             │                                             │
//!             └──────────────────────┬──────────────────────┘
//!                                    ▼
//!                        ┌────────────────────────┐  DiscoveryState::unresolved() ┌────────────────────────┐
//!                        │ DiscoveredService      │ ────────────────────────────▶ │ Unresolved::resolve    │ ──▶ the backend's
//!                        │   one per service hash │  the services that changed    │   asks the resolver,   │     resolver
//!                        │   local, remotes,      │                               │   stamps the result    │
//!                        │   resolution           │ ◀──────────────────────────── │   with the epoch       │
//!                        └────────────────────────┘  the Resolution, stored       └────────────────────────┘
//!                                    │
//!                ┌───────────────────┴─────────────────────────┐
//!                │  DiscoveryState::bridgeable()               │  DiscoveryState::exported()
//!                │  the set of services that get               │  the set of local services that
//!                │  a bridge                                   │  should be exported
//!                ▼                                             ▼
//!     ┌────────────────────────┐                    ┌────────────────────────┐
//!     │ BridgeableService      │                    │ ExportedService        │
//!     │   a view of a          │                    │   a view of a          │
//!     │   DiscoveredService    │                    │   DiscoveredService    │
//!     └────────────────────────┘                    └────────────────────────┘
//!                │                                             │
//!                ▼                                             ▼
//!       Bridges::reconcile                          ┌────────────────────────┐
//!       opens and closes bridges                    │ announcement::Update   │
//!       to match the set                            │   announce() per       │
//!                                                   │   exported service     │
//!                                                   │   finalize() withdraws │
//!                                                   │   what was not seen    │
//!                                                   └────────────────────────┘
//!                                                             │
//!                                                             ▼
//!                                                     Backend::announce
//! ```
//!
//! - A bridgeable service is either exported or imported. In both cases a
//!   bridge is created in [`Bridges`](crate::Bridges) to propagate data via
//!   the backend.
//! - The exported services are the subset of local services that this side
//!   offers over the backend. The link announces them for the opposing side
//!   to discover.

mod announcement;
mod discovered_service;
mod update;

use alloc::collections::BTreeMap;
use core::fmt::Display;

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_elementary::epoch::Epoch;
use iceoryx2_bb_elementary::generation::Generation;
use iceoryx2_link_backend::diagnostic::Findings;
use iceoryx2_link_backend::resolver::Resolution;

pub(crate) use announcement::AnnouncementState;
use discovered_service::DiscoveredService;
pub(crate) use discovered_service::{BridgeableService, CreationId, ExportedService};
use update::{LocalUpdate, RemoteUpdate, Unresolved, warn_unresolvable};

/// What was discovered on both sides of the boundary and what each
/// service resolves to.
pub(crate) struct DiscoveryState<RemoteId, RemoteDescription, Refusal> {
    entries: BTreeMap<ServiceHash, DiscoveredService<RemoteId, RemoteDescription, Refusal>>,
    /// The remote descriptions listed, each under the service it belongs
    /// to.
    listed: BTreeMap<RemoteId, ServiceHash>,
    /// The remote descriptions in scope that belong to no service.
    unresolvable: Findings<RemoteId, Refusal>,
    /// The generation of the local listing last updated from.
    local_generation: Generation,
    /// The generation of the remote listing last updated from.
    remote_generation: Generation,
    /// Moved by every update, the stamp of what a update saw and of what
    /// changed since an entry was resolved.
    epoch: Epoch,
}

impl<RemoteId, RemoteDescription, Refusal> Default
    for DiscoveryState<RemoteId, RemoteDescription, Refusal>
{
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

impl<RemoteId: Ord + Clone + Display, RemoteDescription: PartialEq, Refusal: Display + PartialEq>
    DiscoveryState<RemoteId, RemoteDescription, Refusal>
{
    /// Begins an update from the local system's listing at `generation`, none
    /// if it has not moved.
    pub(crate) fn update_locals(
        &mut self,
        generation: Generation,
    ) -> Option<LocalUpdate<'_, RemoteId, RemoteDescription, Refusal>> {
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
    ) -> Option<RemoteUpdate<'_, RemoteId, RemoteDescription, Refusal>> {
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

    /// Iterates the updates that need to be resolved by the backend resolver.
    pub(crate) fn unresolved(
        &mut self,
    ) -> impl Iterator<Item = Unresolved<'_, RemoteId, RemoteDescription, Refusal>> {
        let epoch = self.epoch;
        self.entries
            .iter_mut()
            .filter(|(_, entry)| entry.stale)
            .map(move |(hash, entry)| Unresolved { hash, entry, epoch })
    }

    /// Iterates the services that resolved to a bridge, in hash order,
    /// each with the description it is bridged as on this side, the one
    /// it is opened as on the opposing side, and the epoch its
    /// resolution was decided in.
    pub(crate) fn bridgeable(
        &self,
    ) -> impl Iterator<Item = BridgeableService<'_, RemoteDescription>> {
        self.entries.iter().filter_map(|(hash, entry)| {
            let (local_description, remote_description) = match &entry.resolution {
                Resolution::Exported(remote) => (&entry.local.as_ref()?.description, remote),
                Resolution::Imported(mirror, remote) => (mirror, remote),
                _ => return None,
            };

            Some(BridgeableService {
                hash,
                local_description,
                remote_description,
                resolved: entry.resolved,
            })
        })
    }

    /// Iterates the exported local services with their creation ids,
    /// in hash order.
    pub(crate) fn exported(&self) -> impl Iterator<Item = ExportedService<'_>> {
        self.entries.values().filter_map(|entry| {
            let local = entry.local.as_ref()?;
            matches!(entry.resolution, Resolution::Exported(_)).then_some(ExportedService {
                description: &local.description,
                creation_id: local.id,
            })
        })
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
    use iceoryx2_link_backend::service_description::{ServiceDescription, ServiceTypes};

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
        for unresolved in sut.unresolved() {
            resolved.push(*unresolved.hash());
            unresolved.resolve(|_, _| resolution.clone());
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
        assert_that!(entry.local_description(), eq Some(&service));
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
        assert_that!(entry.remote_descriptions().collect::<Vec<_>>(), eq alloc::vec![&remote]);
        assert_that!(entry.local_description(), is_none);
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
        assert_that!(entry.local_description(), is_none);
    }

    #[test]
    fn an_entry_resolved_to_the_same_resolution_keeps_its_resolved_epoch() {
        const SERVICE: &str = "state/resolution/unchanged";

        let mut sut = Sut::default();
        let service = description(SERVICE);
        update_from_locals(&mut sut, FIRST_GENERATION, &[(FIRST_CREATION, &service)]);
        resolve_to(&mut sut, Resolution::Exported(remote(&service)));
        let before = sut.bridgeable().map(|entry| entry.resolved).next();
        update_from_remotes(
            &mut sut,
            FIRST_GENERATION,
            &[(FIRST_REMOTE, remote(&service))],
        );
        resolve_to(&mut sut, Resolution::Exported(remote(&service)));

        let after = sut.bridgeable().map(|entry| entry.resolved).next();
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
        for unresolved in sut.unresolved() {
            let resolution = match *unresolved.hash() {
                hash if hash == exported.hash() => Resolution::Exported(remote(&exported)),
                hash if hash == imported.hash() => {
                    Resolution::Imported(imported.clone(), remote(&imported))
                }
                _ => Resolution::Refused(FakeRefusal),
            };
            unresolved.resolve(|_, _| resolution);
        }

        let bridgeable: Vec<ServiceHash> = sut.bridgeable().map(|entry| *entry.hash).collect();
        let exported_hashes: Vec<ServiceHash> = sut
            .exported()
            .map(|exported| exported.description.hash())
            .collect();

        let mut expected = alloc::vec![exported.hash(), imported.hash()];
        expected.sort();
        assert_that!(bridgeable, eq expected);
        assert_that!(exported_hashes, eq alloc::vec![exported.hash()]);
    }
}
