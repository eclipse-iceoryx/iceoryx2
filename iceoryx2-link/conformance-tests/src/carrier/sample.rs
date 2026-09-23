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
pub mod carrier_sample {
    use core::time::Duration;

    use alloc::vec::Vec;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;

    use iceoryx2_link_adapter::SampleBytesRef;
    use iceoryx2_link_carrier::{Carrier, SampleChannel};

    use crate::fixture::CarrierFixture;
    use crate::testing::{descriptor, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const PAYLOAD: &str = "u64";

    fn send<C: SampleChannel>(channel: &mut C, header: &[u8], payload: &[u8]) {
        channel
            .send(SampleBytesRef { header, payload })
            .expect("sending succeeds");
    }

    /// The next sample's bytes, header then payload.
    fn receive<C: SampleChannel>(channel: &mut C) -> Option<Vec<u8>> {
        channel
            .receive()
            .expect("receiving succeeds")
            .map(<[u8]>::to_vec)
    }

    fn expect_sample<C: SampleChannel>(channel: &mut C, header: &[u8], payload: &[u8]) {
        let expected = [header, payload].concat();
        retry(
            || match receive(channel) {
                Some(bytes) if bytes == expected => Ok(()),
                Some(_) => Err("an unexpected sample arrived"),
                None => Err("no sample arrived"),
            },
            TIMEOUT,
        )
        .expect("the sample arrives");
    }

    #[conformance_test]
    pub fn samples_reach_every_other_peer_with_the_channel_open<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/reach";
        const NO_HEADER_BYTES: &[u8] = &[];
        const PAYLOAD_BYTES: &[u8] = b"payload";

        let mut fixture = F::new();

        // === SETUP ===
        // Three peers with the same channel open.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let mut c = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_sample_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        let mut channel_c = c.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A sample from A arrives at B and C, and never at A itself.
        send(&mut channel_a, NO_HEADER_BYTES, PAYLOAD_BYTES);

        expect_sample(&mut channel_b, NO_HEADER_BYTES, PAYLOAD_BYTES);
        expect_sample(&mut channel_c, NO_HEADER_BYTES, PAYLOAD_BYTES);
        assert_that!(receive(&mut channel_a), is_none);
    }

    #[conformance_test]
    pub fn samples_do_not_reach_peers_without_the_channel_open<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/unopened";
        const NO_HEADER_BYTES: &[u8] = &[];
        const PAYLOAD_BYTES: &[u8] = b"payload";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers, only peer A opens the channel.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // Peer A sends before peer B has the channel open.
        send(&mut channel_a, NO_HEADER_BYTES, PAYLOAD_BYTES);

        // === OPEN LATE ===
        // Nothing is held for a peer that was not listening.
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(receive(&mut channel_b), is_none);
    }

    #[conformance_test]
    pub fn samples_do_not_cross_between_descriptors_of_one_service<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/descriptors";
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
            .open_sample_channel(&descriptor(SERVICE, PAYLOAD))
            .expect("channel opens");
        let mut channel_b = b
            .open_sample_channel(&descriptor(SERVICE, OTHER_PAYLOAD))
            .expect("channel opens");

        // === SEND ===
        // Different descriptors are different channels.
        send(&mut channel_a, NO_HEADER_BYTES, PAYLOAD_BYTES);

        assert_that!(receive(&mut channel_b), is_none);
    }

    #[conformance_test]
    pub fn samples_are_received_in_order<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/order";
        const NO_HEADER_BYTES: &[u8] = &[];
        const FIRST_PAYLOAD_BYTES: &[u8] = b"first payload";
        const SECOND_PAYLOAD_BYTES: &[u8] = b"second payload";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers with the same channel open.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_sample_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // Two samples from A arrive at B in the order they were sent.
        send(&mut channel_a, NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES);
        send(&mut channel_a, NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES);

        expect_sample(&mut channel_b, NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES);
        expect_sample(&mut channel_b, NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn closed_channels_stop_receiving<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/closed";
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

        let mut channel_a = a.open_sample_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A sample sent while the second peer has its channel open arrives.
        send(&mut channel_a, NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES);
        expect_sample(&mut channel_b, NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES);

        // === CLOSE ===
        // The second peer closes its channel.
        drop(channel_b);
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A sample sent while the channel is closed.
        send(&mut channel_a, NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES);

        // === REOPEN ===
        // The second peer reopens its channel.
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // The next sample the second peer receives is one sent after it
        // reopened, the sample sent while it was closed is not received.
        send(&mut channel_a, NO_HEADER_BYTES, THIRD_PAYLOAD_BYTES);
        expect_sample(&mut channel_b, NO_HEADER_BYTES, THIRD_PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn header_precedes_payload_in_the_received_bytes<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/regions";
        const HEADER_BYTES: &[u8] = b"header";
        const PAYLOAD_BYTES: &[u8] = b"payload";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers with the same channel open.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor(SERVICE, PAYLOAD);

        let mut channel_a = a.open_sample_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A sample with a header arrives as the header's bytes followed by
        // the payload's.
        send(&mut channel_a, HEADER_BYTES, PAYLOAD_BYTES);

        expect_sample(&mut channel_b, HEADER_BYTES, PAYLOAD_BYTES);
    }
}
