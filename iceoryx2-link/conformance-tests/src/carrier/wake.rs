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
pub mod carrier_wake {
    use core::time::Duration;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_backend::Reactive;
    use iceoryx2_link_carrier::Frame;
    use iceoryx2_link_carrier::{Announcement, Carrier, Channel};

    use crate::fixture::CarrierFixture;
    use crate::testing::{WakeSource, descriptor, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const PAYLOAD: &str = "u64";

    #[conformance_test]
    pub fn a_peer_is_woken_by_an_offer<F: CarrierFixture>()
    where
        F::Carrier: Reactive,
    {
        const SERVICE: &str = "carrier/wake/offer";

        let mut fixture = F::new();

        // === SETUP ===
        // Peer B waits on a wake it attached to its carrier.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let source = WakeSource::new();
        b.attach(source.wake.clone());
        assert_that_not_woken(&source);

        // === OFFER ===
        // Peer A's announcement wakes peer B.
        a.announce(Announcement::Offered {
            descriptor: descriptor(SERVICE, PAYLOAD),
        })
        .expect("announcement succeeds");

        retry(
            || match source.woken() {
                true => Ok(()),
                false => Err("the peer was not woken"),
            },
            TIMEOUT,
        )
        .expect("the offer wakes the peer");
    }

    #[conformance_test]
    pub fn a_peer_is_woken_by_a_frame<F: CarrierFixture>()
    where
        F::Carrier: Reactive,
    {
        const SERVICE: &str = "carrier/wake/frame";

        let mut fixture = F::new();

        // === SETUP ===
        // Peers A and B with the same channel open, B waiting on its wake.
        let a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);
        let channel_a = a.open_channel(&descriptor).expect("channel opens");
        let _channel_b = b.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);
        let source = WakeSource::new();
        b.attach(source.wake.clone());
        assert_that_not_woken(&source);

        // === SEND ===
        // A frame from peer A wakes peer B.
        channel_a
            .send(Frame {
                header: &[],
                payload: b"frame",
            })
            .expect("sending succeeds");

        retry(
            || match source.woken() {
                true => Ok(()),
                false => Err("the peer was not woken"),
            },
            TIMEOUT,
        )
        .expect("the frame wakes the peer");
    }

    fn assert_that_not_woken(source: &WakeSource) {
        assert_that!(source.woken(), eq false);
    }
}
