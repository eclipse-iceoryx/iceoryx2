// Copyright (c) 2025 Contributors to the Eclipse Foundation
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

use std::collections::{BTreeMap, BTreeSet};

use iceoryx2::service::service_hash::ServiceHash;
use iceoryx2_link_backend::service_description::ServiceDescriptor;
use iceoryx2_link_carrier::{Offer, PeerId};

use crate::fingerprint::Fingerprint;

/// One descriptor as offered by the peers.
struct Offered {
    peers: BTreeSet<PeerId>,
    descriptor: Option<ServiceDescriptor>,
}

/// The state of an offer after a peer's offer of it was recorded.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum OfferState {
    /// The peer is new and the descriptor is held.
    New,
    /// The descriptor is not held.
    Pending,
    /// The peer offered it before and the descriptor is held.
    Known,
}

/// The peers' offers by hash and fingerprint.
#[derive(Default)]
pub(crate) struct OfferTable {
    offered: BTreeMap<(ServiceHash, Fingerprint), Offered>,
}

impl OfferTable {
    /// Records `peer` offering the descriptor.
    pub(crate) fn offer(
        &mut self,
        hash: ServiceHash,
        fingerprint: Fingerprint,
        peer: PeerId,
    ) -> OfferState {
        let entry = self
            .offered
            .entry((hash, fingerprint))
            .or_insert_with(|| Offered {
                peers: BTreeSet::new(),
                descriptor: None,
            });
        let new_peer = entry.peers.insert(peer);
        match (new_peer, entry.descriptor.is_some()) {
            (_, false) => OfferState::Pending,
            (true, true) => OfferState::New,
            (false, true) => OfferState::Known,
        }
    }

    /// Records `peer` withdrawing the descriptor. True when the peer was
    /// removed from an offer whose descriptor is held.
    pub(crate) fn withdraw(
        &mut self,
        hash: &ServiceHash,
        fingerprint: &Fingerprint,
        peer: &PeerId,
    ) -> bool {
        let key = (*hash, fingerprint.clone());
        let Some(entry) = self.offered.get_mut(&key) else {
            return false;
        };
        let changed = entry.peers.remove(peer) && entry.descriptor.is_some();
        if entry.peers.is_empty() {
            self.offered.remove(&key);
        }
        changed
    }

    /// Records the descriptor of an offer. True when it was not held before
    /// and the offer has peers.
    pub(crate) fn describe(
        &mut self,
        hash: &ServiceHash,
        fingerprint: &Fingerprint,
        descriptor: ServiceDescriptor,
    ) -> bool {
        let Some(entry) = self.offered.get_mut(&(*hash, fingerprint.clone())) else {
            return false;
        };
        if entry.descriptor.is_some() {
            return false;
        }
        entry.descriptor = Some(descriptor);
        !entry.peers.is_empty()
    }

    /// Whether the offer is held without its descriptor.
    pub(crate) fn is_pending(&self, hash: &ServiceHash, fingerprint: &Fingerprint) -> bool {
        self.offered
            .get(&(*hash, fingerprint.clone()))
            .is_some_and(|offered| offered.descriptor.is_none())
    }

    /// Every offer with a known descriptor.
    pub(crate) fn each(&self, callback: &mut dyn FnMut(Offer)) {
        for offered in self.offered.values() {
            let Some(descriptor) = &offered.descriptor else {
                continue;
            };
            for peer in &offered.peers {
                callback(Offer {
                    peer: *peer,
                    descriptor: descriptor.clone(),
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_link_conformance_tests::testing::descriptor;

    use crate::fingerprint::Encoded;

    const PAYLOAD: &str = "u64";
    const PEER_A: u8 = 1;
    const PEER_B: u8 = 2;

    fn peer(id: u8) -> PeerId {
        PeerId::new([id; PeerId::LENGTH])
    }

    fn fingerprint(descriptor: &ServiceDescriptor) -> Fingerprint {
        Encoded::encode(descriptor)
            .expect("descriptor encodes")
            .fingerprint()
            .clone()
    }

    fn listed(table: &OfferTable) -> Vec<Offer> {
        let mut listed = Vec::new();
        table.each(&mut |offer| listed.push(offer));
        listed
    }

    #[test]
    fn an_offer_without_its_descriptor_is_pending_and_unlisted() {
        const SERVICE: &str = "offers/pending";

        let descriptor = descriptor(SERVICE, PAYLOAD);
        let fingerprint = fingerprint(&descriptor);
        let mut table = OfferTable::default();
        let state = table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_A));

        assert_that!(state, eq OfferState::Pending);
        assert_that!(table.is_pending(&descriptor.hash, &fingerprint), eq true);
        assert_that!(listed(&table), len 0);
    }

    #[test]
    fn an_offer_is_listed_once_described() {
        const SERVICE: &str = "offers/described";

        let descriptor = descriptor(SERVICE, PAYLOAD);
        let fingerprint = fingerprint(&descriptor);
        let mut table = OfferTable::default();
        table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_A));
        let described = table.describe(&descriptor.hash, &fingerprint, descriptor.clone());

        assert_that!(described, eq true);
        assert_that!(table.is_pending(&descriptor.hash, &fingerprint), eq false);
        assert_that!(listed(&table), len 1);
    }

    #[test]
    fn a_further_peer_of_a_listed_offer_changes_the_listing() {
        const SERVICE: &str = "offers/further-peer";

        let descriptor = descriptor(SERVICE, PAYLOAD);
        let fingerprint = fingerprint(&descriptor);
        let mut table = OfferTable::default();
        table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_A));
        table.describe(&descriptor.hash, &fingerprint, descriptor.clone());
        let state = table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_B));

        assert_that!(state, eq OfferState::New);
        assert_that!(listed(&table), len 2);
    }

    #[test]
    fn a_repeated_offer_and_a_repeated_description_change_nothing() {
        const SERVICE: &str = "offers/repeated";

        let descriptor = descriptor(SERVICE, PAYLOAD);
        let fingerprint = fingerprint(&descriptor);
        let mut table = OfferTable::default();
        table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_A));
        table.describe(&descriptor.hash, &fingerprint, descriptor.clone());
        let state = table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_A));
        let described = table.describe(&descriptor.hash, &fingerprint, descriptor.clone());

        assert_that!(state, eq OfferState::Known);
        assert_that!(described, eq false);
        assert_that!(listed(&table), len 1);
    }

    #[test]
    fn a_withdrawal_of_a_pending_offer_changes_nothing() {
        const SERVICE: &str = "offers/withdrawn-pending";

        let descriptor = descriptor(SERVICE, PAYLOAD);
        let fingerprint = fingerprint(&descriptor);
        let mut table = OfferTable::default();
        table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_A));
        let withdrawn = table.withdraw(&descriptor.hash, &fingerprint, &peer(PEER_A));

        assert_that!(withdrawn, eq false);
        assert_that!(table.is_pending(&descriptor.hash, &fingerprint), eq false);
    }

    #[test]
    fn a_withdrawal_of_a_listed_offer_changes_the_listing() {
        const SERVICE: &str = "offers/withdrawn-listed";

        let descriptor = descriptor(SERVICE, PAYLOAD);
        let fingerprint = fingerprint(&descriptor);
        let mut table = OfferTable::default();
        table.offer(descriptor.hash, fingerprint.clone(), peer(PEER_A));
        table.describe(&descriptor.hash, &fingerprint, descriptor.clone());
        let withdrawn = table.withdraw(&descriptor.hash, &fingerprint, &peer(PEER_A));

        assert_that!(withdrawn, eq true);
        assert_that!(listed(&table), len 0);
    }

    #[test]
    fn a_description_without_an_offer_is_dropped() {
        const SERVICE: &str = "offers/unoffered";

        let descriptor = descriptor(SERVICE, PAYLOAD);
        let fingerprint = fingerprint(&descriptor);
        let mut table = OfferTable::default();

        let described = table.describe(&descriptor.hash, &fingerprint, descriptor.clone());

        assert_that!(described, eq false);
        assert_that!(listed(&table), len 0);
    }
}
