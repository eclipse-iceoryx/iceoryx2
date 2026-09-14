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

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_bb_elementary::epoch::Epoch;
use iceoryx2_link_backend::Announcement;
use iceoryx2_link_backend::origin;
use iceoryx2_link_backend::service_description::ServiceDescription;
use iceoryx2_log::{fail, trace};

use crate::discovery_state::CreationId;

/// An exported service as the opposing side was last told of it.
struct Announced {
    /// The creation id of the service that was offered.
    creation_id: CreationId,
    /// The epoch of the update that last announced the service.
    seen: Epoch,
}

/// What the opposing side has been told this side offers.
#[derive(Default)]
pub(crate) struct AnnouncementState {
    announced: BTreeMap<ServiceHash, Announced>,
    /// Moved by every update to detect stale entries.
    epoch: Epoch,
}

impl AnnouncementState {
    /// Begins a update of the exported services, announcing each change through `announce`.
    pub(crate) fn update<A, E>(&mut self, announce: A) -> Update<'_, A>
    where
        A: FnMut(Announcement<'_>) -> Result<(), E>,
    {
        self.epoch = self.epoch.next();
        Update {
            state: self,
            announce,
        }
    }
}

/// One update of what the opposing side is told.
pub(crate) struct Update<'a, A> {
    state: &'a mut AnnouncementState,
    announce: A,
}

impl<A, E> Update<'_, A>
where
    A: FnMut(Announcement<'_>) -> Result<(), E>,
{
    /// Records `description` as exported.
    ///
    /// Previously unseen services are announced. Seen services that have now
    /// a different creation id are re-announced with the new description.
    pub(crate) fn set_exported(
        &mut self,
        description: &ServiceDescription,
        creation_id: CreationId,
    ) -> Result<(), E> {
        let origin = origin!("Update::set_exported");

        let hash = description.hash();
        let epoch = self.state.epoch;
        let announced = &mut self.state.announced;
        if let Some(entry) = announced.get_mut(&hash) {
            if entry.creation_id == creation_id {
                entry.seen = epoch;
                return Ok(());
            }
            trace!(from origin, "Withdrawing {}, it was recreated", description.name());
            fail!(
                from origin,
                when (self.announce)(Announcement::Withdrawn(hash)),
                "Failed to withdraw {}", description.name()
            );
            announced.remove(&hash);
        }
        trace!(from origin, "Announcing {}", description.name());
        fail!(
            from origin,
            when (self.announce)(Announcement::Offered(description)),
            "Failed to announce {}", description.name()
        );
        announced.insert(
            hash,
            Announced {
                creation_id,
                seen: epoch,
            },
        );
        Ok(())
    }

    /// Finalizes the update. A service no longer present is withdrawn.
    pub(crate) fn finalize(self) -> Result<(), E> {
        let origin = origin!("Update::finalize");

        let Update {
            state,
            mut announce,
        } = self;
        let epoch = state.epoch;
        let mut failed = None;
        state.announced.retain(|hash, entry| {
            if entry.seen == epoch || failed.is_some() {
                return true;
            }
            trace!(from origin, "Withdrawing the service {}", hash.as_str());
            match announce(Announcement::Withdrawn(*hash)) {
                Ok(()) => false,
                Err(error) => {
                    failed = Some(error);
                    true
                }
            }
        });
        match failed {
            Some(error) => Err(error),
            None => Ok(()),
        }
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

    /// An announcement as logged, owning what it names.
    #[derive(Debug, PartialEq, Eq)]
    #[allow(clippy::large_enum_variant)]
    enum Logged {
        Offered(ServiceDescription),
        Withdrawn(ServiceHash),
    }

    impl From<Announcement<'_>> for Logged {
        fn from(announcement: Announcement<'_>) -> Self {
            match announcement {
                Announcement::Offered(description) => Logged::Offered(description.clone()),
                Announcement::Withdrawn(hash) => Logged::Withdrawn(hash),
            }
        }
    }

    const FIRST_CREATION: CreationId = 1;
    const SECOND_CREATION: CreationId = 2;

    fn description(name: &str) -> ServiceDescription {
        ServiceDescription::with_default_settings::<local::Service>(
            ServiceName::new(name).expect("valid service name"),
            ServiceTypes::Event,
            &Config::default(),
        )
    }

    /// Updates the state with `exported`, each under its creation,
    /// logging every announcement. Every announcement is accepted.
    fn update(
        sut: &mut AnnouncementState,
        exported: &[(CreationId, &ServiceDescription)],
        log: &mut Vec<Logged>,
    ) {
        let mut update = sut.update(|announcement| {
            log.push(Logged::from(announcement));
            Ok::<(), ()>(())
        });
        for (creation_id, description) in exported {
            update
                .set_exported(description, *creation_id)
                .expect("accepted");
        }
        update.finalize().expect("accepted");
    }

    /// Updates the state with `exported`, refusing every announcement.
    fn update_refused(sut: &mut AnnouncementState, exported: &[(CreationId, &ServiceDescription)]) {
        let mut update = sut.update(|_| Err(()));
        for (creation_id, description) in exported {
            if update.set_exported(description, *creation_id).is_err() {
                return;
            }
        }
        let _ = update.finalize();
    }

    #[test]
    fn an_exported_service_is_offered_once() {
        const SERVICE: &str = "announce/once";

        let mut sut = AnnouncementState::default();
        let service = description(SERVICE);
        let mut log = Vec::new();
        update(&mut sut, &[(FIRST_CREATION, &service)], &mut log);
        update(&mut sut, &[(FIRST_CREATION, &service)], &mut log);

        assert_that!(log, eq alloc::vec![Logged::Offered(service)]);
    }

    #[test]
    fn a_service_no_longer_exported_is_withdrawn() {
        const SERVICE: &str = "announce/withdrawn";

        let mut sut = AnnouncementState::default();
        let service = description(SERVICE);
        let mut log = Vec::new();
        update(&mut sut, &[(FIRST_CREATION, &service)], &mut log);
        update(&mut sut, &[], &mut log);

        let expected = alloc::vec![
            Logged::Offered(service.clone()),
            Logged::Withdrawn(service.hash()),
        ];
        assert_that!(log, eq expected);
    }

    #[test]
    fn a_recreated_service_is_withdrawn_and_offered_anew() {
        const SERVICE: &str = "announce/recreated";

        let mut sut = AnnouncementState::default();
        let service = description(SERVICE);
        let mut log = Vec::new();
        update(&mut sut, &[(FIRST_CREATION, &service)], &mut log);
        update(&mut sut, &[(SECOND_CREATION, &service)], &mut log);

        let expected = alloc::vec![
            Logged::Offered(service.clone()),
            Logged::Withdrawn(service.hash()),
            Logged::Offered(service.clone()),
        ];
        assert_that!(log, eq expected);
    }

    #[test]
    fn a_refused_offer_is_offered_again() {
        const SERVICE: &str = "announce/refused/offer";

        let mut sut = AnnouncementState::default();
        let service = description(SERVICE);
        let mut log = Vec::new();
        update_refused(&mut sut, &[(FIRST_CREATION, &service)]);
        update(&mut sut, &[(FIRST_CREATION, &service)], &mut log);

        assert_that!(log, eq alloc::vec![Logged::Offered(service)]);
    }

    #[test]
    fn a_refused_withdrawal_is_withdrawn_again() {
        const SERVICE: &str = "announce/refused/withdrawal";

        let mut sut = AnnouncementState::default();
        let service = description(SERVICE);
        let mut log = Vec::new();
        update(&mut sut, &[(FIRST_CREATION, &service)], &mut log);
        update_refused(&mut sut, &[]);
        update(&mut sut, &[], &mut log);

        let expected = alloc::vec![
            Logged::Offered(service.clone()),
            Logged::Withdrawn(service.hash()),
        ];
        assert_that!(log, eq expected);
    }
}
