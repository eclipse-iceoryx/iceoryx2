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
pub mod carrier_event {
    use core::time::Duration;

    use iceoryx2::port::event_id::EventId;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_carrier::{Carrier, EventChannel};

    use crate::fixture::CarrierFixture;
    use crate::testing::{event_descriptor, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);

    fn receive<C: EventChannel>(channel: &mut C) -> Option<EventId> {
        channel.receive().expect("receiving succeeds")
    }

    fn expect_id<C: EventChannel>(channel: &mut C, expected: EventId) {
        retry(
            || match receive(channel) {
                Some(id) if id == expected => Ok(()),
                Some(_) => Err("an unexpected id arrived"),
                None => Err("no id arrived"),
            },
            TIMEOUT,
        )
        .expect("the id arrives");
    }

    #[conformance_test]
    pub fn ids_reach_every_other_peer_with_the_channel_open<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/events/reach";
        const ID: EventId = EventId::new(7);

        let mut fixture = F::new();

        // === SETUP ===
        // Three peers with the same channel open.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let mut c = fixture.carrier();
        let descriptor = event_descriptor(SERVICE);

        let mut channel_a = a.open_event_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_event_channel(&descriptor).expect("channel opens");
        let mut channel_c = c.open_event_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // An id from A arrives at B and C, and never at A itself.
        channel_a.send(ID).expect("sending succeeds");

        expect_id(&mut channel_b, ID);
        expect_id(&mut channel_c, ID);
        assert_that!(receive(&mut channel_a), is_none);
    }

    #[conformance_test]
    pub fn ids_are_received_in_order<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/events/order";
        const FIRST_ID: EventId = EventId::new(1);
        const SECOND_ID: EventId = EventId::new(2);

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers with the same channel open.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = event_descriptor(SERVICE);

        let mut channel_a = a.open_event_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_event_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // Two ids from A arrive at B in the order they were sent.
        channel_a.send(FIRST_ID).expect("sending succeeds");
        channel_a.send(SECOND_ID).expect("sending succeeds");

        expect_id(&mut channel_b, FIRST_ID);
        expect_id(&mut channel_b, SECOND_ID);
    }
}
