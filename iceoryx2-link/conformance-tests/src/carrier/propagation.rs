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
pub mod carrier_propagation {
    use alloc::vec::Vec;
    use core::time::Duration;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_carrier::Frame;
    use iceoryx2_link_carrier::{Carrier, Channel};

    use crate::fixture::CarrierFixture;
    use crate::testing::{Taken, descriptor, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const PAYLOAD: &str = "u64";

    fn frame<'a>(header: &'a [u8], payload: &'a [u8]) -> Frame<'a> {
        Frame { header, payload }
    }

    /// The next frame as one buffer, header then payload.
    fn receive<C: Channel>(channel: &mut C) -> Option<Vec<u8>> {
        channel
            .receive(Taken)
            .expect("receiving succeeds")
            .map(|taken| [taken.header, taken.payload].concat())
    }

    fn expect_frame<C: Channel>(channel: &mut C, expected: &[u8]) {
        retry(
            || match receive(channel) {
                Some(frame) if frame == expected => Ok(()),
                Some(_) => Err("an unexpected frame arrived"),
                None => Err("no frame arrived"),
            },
            TIMEOUT,
        )
        .expect("the frame arrives");
    }

    #[conformance_test]
    pub fn frames_reach_every_other_peer_with_the_channel_open<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/reach";

        let mut fixture = F::new();

        // === SETUP ===
        // Three peers with the same channel open.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let mut c = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_channel(&descriptor).expect("channel opens");
        let mut channel_c = c.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A frame from A arrives at B and C as one contiguous sequence,
        // header then payload, and never at A itself.
        channel_a
            .send(frame(b"fra", b"me"))
            .expect("sending succeeds");

        expect_frame(&mut channel_b, b"frame");
        expect_frame(&mut channel_c, b"frame");
        assert_that!(receive(&mut channel_a), is_none);
    }

    #[conformance_test]
    pub fn frames_do_not_reach_peers_without_the_channel_open<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/unopened";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers, only peer A opens the channel.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // Peer A sends before peer B has the channel open.
        channel_a
            .send(frame(b"fra", b"me"))
            .expect("sending succeeds");

        // === OPEN LATE ===
        // Nothing is held for a peer that was not listening.
        let mut channel_b = b.open_channel(&descriptor).expect("channel opens");
        assert_that!(receive(&mut channel_b), is_none);
    }

    #[conformance_test]
    pub fn frames_do_not_cross_between_descriptors_of_one_service<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/descriptors";
        const OTHER_PAYLOAD: &str = "u32";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers on one service under differing types, so their
        // descriptors share the hash and differ in types.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();

        let mut channel_a = a
            .open_channel(&descriptor(SERVICE, PAYLOAD))
            .expect("channel opens");
        let mut channel_b = b
            .open_channel(&descriptor(SERVICE, OTHER_PAYLOAD))
            .expect("channel opens");

        // === SEND ===
        // Different descriptors are different channels.
        channel_a
            .send(frame(b"fra", b"me"))
            .expect("sending succeeds");

        assert_that!(receive(&mut channel_b), is_none);
    }

    #[conformance_test]
    pub fn frames_are_received_in_order<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/order";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers with the same channel open.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // Two frames from A arrive at B in the order they were sent.
        channel_a
            .send(frame(b"fir", b"st"))
            .expect("sending succeeds");
        channel_a
            .send(frame(b"seco", b"nd"))
            .expect("sending succeeds");

        expect_frame(&mut channel_b, b"first");
        expect_frame(&mut channel_b, b"second");
    }

    #[conformance_test]
    pub fn closed_channels_stop_receiving<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/closed";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers open a channel for the same service.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A frame sent while the second peer has its channel open arrives.
        channel_a
            .send(frame(b"fra", b"me"))
            .expect("sending succeeds");
        expect_frame(&mut channel_b, b"frame");

        // === CLOSE ===
        // The second peer closes its channel.
        drop(channel_b);
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A frame sent while the channel is closed.
        channel_a
            .send(frame(b"sec", b"ond"))
            .expect("sending succeeds");

        // === REOPEN ===
        // The second peer reopens its channel.
        let mut channel_b = b.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // The next frame the second peer receives is one sent after it
        // reopened, the frame sent while it was closed is not received.
        channel_a
            .send(frame(b"thi", b"rd"))
            .expect("sending succeeds");
        expect_frame(&mut channel_b, b"third");
    }
}
