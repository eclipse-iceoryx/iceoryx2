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
    use core::time::Duration;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_backend::wire::sample::WriteError;
    use iceoryx2_link_carrier::Frame;
    use iceoryx2_link_carrier::{Carrier, Channel, ReceiveError};

    use crate::fixture::CarrierFixture;
    use crate::testing::{
        NotLoanable, Taken, TakenMessage, descriptor, descriptor_with_header, retry,
    };

    const TIMEOUT: Duration = Duration::from_secs(10);
    const PAYLOAD: &str = "u64";

    fn frame<'a>(header: &'a [u8], payload: &'a [u8]) -> Frame<'a> {
        Frame { header, payload }
    }

    /// The next frame split into its header and payload.
    fn receive<C: Channel>(channel: &mut C) -> Option<TakenMessage> {
        channel.receive(Taken).expect("receiving succeeds")
    }

    fn expect_frame<C: Channel>(channel: &mut C, header: &[u8], payload: &[u8]) {
        retry(
            || match receive(channel) {
                Some(taken) if taken.header == header && taken.payload == payload => Ok(()),
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
        const NO_HEADER_BYTES: &[u8] = &[];
        const PAYLOAD_BYTES: &[u8] = b"payload";

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
        // A frame from A arrives at B and C, and never at A itself.
        channel_a
            .send(frame(NO_HEADER_BYTES, PAYLOAD_BYTES))
            .expect("sending succeeds");

        expect_frame(&mut channel_b, NO_HEADER_BYTES, PAYLOAD_BYTES);
        expect_frame(&mut channel_c, NO_HEADER_BYTES, PAYLOAD_BYTES);
        assert_that!(receive(&mut channel_a), is_none);
    }

    #[conformance_test]
    pub fn frames_do_not_reach_peers_without_the_channel_open<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/unopened";
        const NO_HEADER_BYTES: &[u8] = &[];
        const PAYLOAD_BYTES: &[u8] = b"payload";

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
            .send(frame(NO_HEADER_BYTES, PAYLOAD_BYTES))
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
        const NO_HEADER_BYTES: &[u8] = &[];
        const PAYLOAD_BYTES: &[u8] = b"payload";

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
            .send(frame(NO_HEADER_BYTES, PAYLOAD_BYTES))
            .expect("sending succeeds");

        assert_that!(receive(&mut channel_b), is_none);
    }

    #[conformance_test]
    pub fn frames_are_received_in_order<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/order";
        const NO_HEADER_BYTES: &[u8] = &[];
        const FIRST_PAYLOAD_BYTES: &[u8] = b"first payload";
        const SECOND_PAYLOAD_BYTES: &[u8] = b"second payload";

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
            .send(frame(NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES))
            .expect("sending succeeds");
        channel_a
            .send(frame(NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES))
            .expect("sending succeeds");

        expect_frame(&mut channel_b, NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES);
        expect_frame(&mut channel_b, NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn closed_channels_stop_receiving<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/closed";
        const NO_HEADER_BYTES: &[u8] = &[];
        const FIRST_PAYLOAD_BYTES: &[u8] = b"first payload";
        const SECOND_PAYLOAD_BYTES: &[u8] = b"second payload";
        const THIRD_PAYLOAD_BYTES: &[u8] = b"third payload";

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
            .send(frame(NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES))
            .expect("sending succeeds");
        expect_frame(&mut channel_b, NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES);

        // === CLOSE ===
        // The second peer closes its channel.
        drop(channel_b);
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A frame sent while the channel is closed.
        channel_a
            .send(frame(NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES))
            .expect("sending succeeds");

        // === REOPEN ===
        // The second peer reopens its channel.
        let mut channel_b = b.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // The next frame the second peer receives is one sent after it
        // reopened, the frame sent while it was closed is not received.
        channel_a
            .send(frame(NO_HEADER_BYTES, THIRD_PAYLOAD_BYTES))
            .expect("sending succeeds");
        expect_frame(&mut channel_b, NO_HEADER_BYTES, THIRD_PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn header_and_payload_are_extracted_from_frames<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/regions";
        const HEADER_BYTES: &[u8] = b"header";
        const PAYLOAD_BYTES: &[u8] = b"payload";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers on a service whose user header is the header's size.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor_with_header(SERVICE, PAYLOAD, HEADER_BYTES.len());

        let mut channel_a = a.open_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        channel_a
            .send(frame(HEADER_BYTES, PAYLOAD_BYTES))
            .expect("sending succeeds");

        expect_frame(&mut channel_b, HEADER_BYTES, PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn a_refused_frame_is_dropped<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/frames/refused";
        const NO_HEADER_BYTES: &[u8] = &[];
        const FIRST_PAYLOAD_BYTES: &[u8] = b"first payload";
        const SECOND_PAYLOAD_BYTES: &[u8] = b"second payload";

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
        // Two frames from A.
        channel_a
            .send(frame(NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES))
            .expect("sending succeeds");
        channel_a
            .send(frame(NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES))
            .expect("sending succeeds");

        // === REFUSE ===
        // The loan for the first frame is refused, so it is dropped.
        retry(
            || match channel_b.receive(NotLoanable) {
                Err(ReceiveError::Rejected(WriteError::Malformed)) => Ok(()),
                Err(_) => Err("an unexpected error"),
                Ok(Some(_)) => Err("a refused frame was received"),
                Ok(None) => Err("no frame arrived"),
            },
            TIMEOUT,
        )
        .expect("the first frame is refused");

        // === RECEIVE ===
        // The next frame arrives without issue.
        expect_frame(&mut channel_b, NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES);
    }
}
