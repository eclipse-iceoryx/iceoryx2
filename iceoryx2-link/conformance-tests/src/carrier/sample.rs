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

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_adapter::SampleBytes;
    use iceoryx2_link_carrier::{Carrier, SampleChannel, SampleReceiveError};

    use crate::fixture::CarrierFixture;
    use crate::testing::{NotLoanable, UnloanedBuffers, descriptor, descriptor_with_header, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);
    const PAYLOAD: &str = "u64";

    /// The next sample's bytes split into a header of `header_size` bytes
    /// and its payload.
    fn receive<C: SampleChannel>(channel: &mut C, header_size: usize) -> Option<SampleBytes> {
        channel
            .receive(UnloanedBuffers { header_size })
            .expect("receiving succeeds")
    }

    fn expect_sample<C: SampleChannel>(channel: &mut C, header: &[u8], payload: &[u8]) {
        retry(
            || match receive(channel, header.len()) {
                Some(taken) if taken.header == header && taken.payload == payload => Ok(()),
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
        channel_a
            .send(&[NO_HEADER_BYTES, PAYLOAD_BYTES])
            .expect("sending succeeds");

        expect_sample(&mut channel_b, NO_HEADER_BYTES, PAYLOAD_BYTES);
        expect_sample(&mut channel_c, NO_HEADER_BYTES, PAYLOAD_BYTES);
        assert_that!(receive(&mut channel_a, NO_HEADER_BYTES.len()), is_none);
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
        channel_a
            .send(&[NO_HEADER_BYTES, PAYLOAD_BYTES])
            .expect("sending succeeds");

        // === OPEN LATE ===
        // Nothing is held for a peer that was not listening.
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(receive(&mut channel_b, NO_HEADER_BYTES.len()), is_none);
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
        channel_a
            .send(&[NO_HEADER_BYTES, PAYLOAD_BYTES])
            .expect("sending succeeds");

        assert_that!(receive(&mut channel_b, NO_HEADER_BYTES.len()), is_none);
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
        channel_a
            .send(&[NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES])
            .expect("sending succeeds");
        channel_a
            .send(&[NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES])
            .expect("sending succeeds");

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
        channel_a
            .send(&[NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES])
            .expect("sending succeeds");
        expect_sample(&mut channel_b, NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES);

        // === CLOSE ===
        // The second peer closes its channel.
        drop(channel_b);
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // A sample sent while the channel is closed.
        channel_a
            .send(&[NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES])
            .expect("sending succeeds");

        // === REOPEN ===
        // The second peer reopens its channel.
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // The next sample the second peer receives is one sent after it
        // reopened, the sample sent while it was closed is not received.
        channel_a
            .send(&[NO_HEADER_BYTES, THIRD_PAYLOAD_BYTES])
            .expect("sending succeeds");
        expect_sample(&mut channel_b, NO_HEADER_BYTES, THIRD_PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn header_and_payload_are_extracted_from_bytes<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/regions";
        const HEADER_BYTES: &[u8] = b"header";
        const PAYLOAD_BYTES: &[u8] = b"payload";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers on a service whose user header is the header's size.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor_with_header(SERVICE, PAYLOAD, HEADER_BYTES.len());

        let mut channel_a = a.open_sample_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        channel_a
            .send(&[HEADER_BYTES, PAYLOAD_BYTES])
            .expect("sending succeeds");

        expect_sample(&mut channel_b, HEADER_BYTES, PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn bytes_shorter_than_the_header_are_dropped<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/short";
        const HEADER_BYTES: &[u8] = b"header";
        const SHORT_BYTES: &[u8] = b"ab";
        const PAYLOAD_BYTES: &[u8] = b"payload";

        let mut fixture = F::new();

        // === SETUP ===
        // Two peers on a service whose user header is longer than the
        // short bytes.
        let mut a = fixture.carrier();
        let mut b = fixture.carrier();
        let descriptor = descriptor_with_header(SERVICE, PAYLOAD, HEADER_BYTES.len());

        let mut channel_a = a.open_sample_channel(&descriptor).expect("channel opens");
        let mut channel_b = b.open_sample_channel(&descriptor).expect("channel opens");
        assert_that!(fixture.sync(&descriptor.hash, TIMEOUT), eq true);

        // === SEND ===
        // Bytes that cannot hold the header, then a whole sample.
        channel_a.send(&[SHORT_BYTES]).expect("sending succeeds");
        channel_a
            .send(&[HEADER_BYTES, PAYLOAD_BYTES])
            .expect("sending succeeds");

        // === DROP ===
        // The short bytes are reported as malformed and dropped.
        retry(
            || match channel_b.receive(UnloanedBuffers {
                header_size: HEADER_BYTES.len(),
            }) {
                Err(SampleReceiveError::Malformed) => Ok(()),
                Err(_) => Err("an unexpected error"),
                Ok(Some(_)) => Err("the short bytes were received"),
                Ok(None) => Err("nothing arrived"),
            },
            TIMEOUT,
        )
        .expect("the short bytes are dropped");

        // === RECEIVE ===
        // The whole sample arrives without issue.
        expect_sample(&mut channel_b, HEADER_BYTES, PAYLOAD_BYTES);
    }

    #[conformance_test]
    pub fn a_refused_sample_is_dropped<F: CarrierFixture>() {
        const SERVICE: &str = "carrier/samples/refused";
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
        // Two samples from A.
        channel_a
            .send(&[NO_HEADER_BYTES, FIRST_PAYLOAD_BYTES])
            .expect("sending succeeds");
        channel_a
            .send(&[NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES])
            .expect("sending succeeds");

        // === REFUSE ===
        // The loan for the first sample is refused, so it is dropped.
        retry(
            || match channel_b.receive(NotLoanable) {
                Err(SampleReceiveError::Malformed) => Ok(()),
                Err(_) => Err("an unexpected error"),
                Ok(Some(_)) => Err("a refused sample was received"),
                Ok(None) => Err("no sample arrived"),
            },
            TIMEOUT,
        )
        .expect("the first sample is refused");

        // === RECEIVE ===
        // The next sample arrives without issue.
        expect_sample(&mut channel_b, NO_HEADER_BYTES, SECOND_PAYLOAD_BYTES);
    }
}
