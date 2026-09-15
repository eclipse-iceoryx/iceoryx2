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

use iceoryx2_bb_testing_macros::conformance_tests;

#[allow(clippy::module_inception)]
#[conformance_tests]
pub mod carrier_discovery {
    use alloc::vec::Vec;
    use core::time::Duration;

    use iceoryx2_bb_elementary::generation::Generation;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_backend::service_description::ServiceDescriptor;
    use iceoryx2_link_carrier::{Announcement, Carrier};

    use crate::fixture::CarrierFixture;
    use crate::testing::{descriptor, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const PAYLOAD: &str = "u64";

    /// Offers the service the descriptor stands for.
    fn offer<C: Carrier>(carrier: &mut C, descriptor: &ServiceDescriptor) {
        carrier
            .announce(Announcement::Offered {
                descriptor: descriptor.clone(),
            })
            .expect("announcement succeeds");
    }

    fn offers<C: Carrier>(carrier: &mut C) -> Vec<ServiceDescriptor> {
        let mut offers = Vec::new();
        carrier
            .offers(&mut |offer| offers.push(offer.descriptor))
            .expect("listing succeeds");
        offers
    }

    #[conformance_test]
    pub fn announced_offers_are_visible_to_other_peers_only<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/offers/visible";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers on one mechanism, peer A offering a service.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        offer(&mut a, &descriptor);

        // === OBSERVE ===
        // Peer B sees the offer as A announced it. A does not see its own.
        retry(
            || match offers(&mut b).as_slice() {
                [seen] if *seen == descriptor => Ok(()),
                [] => Err("the offer is not visible to the other peer"),
                _ => Err("an unexpected offer is visible"),
            },
            TIMEOUT,
        )
        .expect("the offer reaches the other peer");
        assert_that!(offers(&mut a), len 0);
    }

    #[conformance_test]
    pub fn the_generation_moves_on_announcements_and_holds_still_otherwise<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/generation";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers on one mechanism, nothing announced yet.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();

        // === OBSERVE ===
        // A carrier without a generation has nothing to hold to.
        let initial = b.generation();
        if initial == Generation::Untracked {
            return;
        }

        // Listing moves nothing.
        offers(&mut b);
        assert_that!(b.generation(), eq initial);

        // An announcement by A moves B's generation.
        offer(&mut a, &descriptor(SERVICE, PAYLOAD));
        retry(
            || match b.generation() != initial {
                true => Ok(()),
                false => Err("the generation has not moved"),
            },
            TIMEOUT,
        )
        .expect("the announcement moves the generation");

        // It holds still until the next announcement.
        let moved = b.generation();
        offers(&mut b);
        assert_that!(b.generation(), eq moved);
    }

    #[conformance_test]
    pub fn withdrawn_offers_disappear<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/offers/withdrawn";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers on one mechanism, peer A's offer visible to peer B.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        offer(&mut a, &descriptor);
        retry(
            || match offers(&mut b).is_empty() {
                false => Ok(()),
                true => Err("the offer is not visible to the other peer"),
            },
            TIMEOUT,
        )
        .expect("the offer reaches the other peer");

        // === WITHDRAW ===
        // Absence is withdrawal, B's next listing no longer has it.
        a.announce(Announcement::Withdrawn {
            hash: descriptor.hash,
        })
        .expect("withdrawal succeeds");

        retry(
            || match offers(&mut b).is_empty() {
                true => Ok(()),
                false => Err("the offer is still visible"),
            },
            TIMEOUT,
        )
        .expect("the withdrawal reaches the other peer");
    }

    #[conformance_test]
    pub fn a_service_offered_anew_after_a_withdrawal_is_the_current_offer<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/offers/superseded";
        const OTHER_PAYLOAD: &str = "u32";

        let mut fixture = F::new();

        // === SETUP ===
        // Peer A offered a service.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let old = descriptor(SERVICE, OTHER_PAYLOAD);
        let current = descriptor(SERVICE, PAYLOAD);
        offer(&mut a, &old);

        // === WITHDRAW AND OFFER ANEW ===
        // Announcements are seen in the order made, so the withdrawal of
        // the old offer never removes the one made after it.
        a.announce(Announcement::Withdrawn { hash: old.hash })
            .expect("withdrawal succeeds");
        offer(&mut a, &current);

        retry(
            || match offers(&mut b).as_slice() {
                [seen] if *seen == current => Ok(()),
                [] => Err("no offer is visible"),
                _ => Err("more than the current offer is visible"),
            },
            TIMEOUT,
        )
        .expect("only the current offer remains");
    }
}
