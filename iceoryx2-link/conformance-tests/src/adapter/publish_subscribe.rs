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
pub mod adapter_publish_subscribe {
    use alloc::vec::Vec;
    use core::time::Duration;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_adapter::{
        Adapter, PublishSubscribeEndpoints, SampleBytes, SampleBytesRef, TakeOutcome,
    };

    use crate::fixture::{AdapterFixture, MessageEndpoints};
    use crate::testing::retry;

    const TIMEOUT: Duration = Duration::from_secs(10);
    const VALUE: u64 = 7;
    const NO_HEADER: &[u8] = &[];

    /// Publishes a message without a header.
    fn publish<E: PublishSubscribeEndpoints>(endpoints: &mut E, payload: &[u8]) {
        endpoints
            .publish(SampleBytesRef {
                header: NO_HEADER,
                payload,
            })
            .expect("publishing succeeds");
    }

    /// The payload of the pending message, if any.
    fn take<E: PublishSubscribeEndpoints>(endpoints: &mut E) -> Option<Vec<u8>> {
        let mut bytes = SampleBytes::default();
        match endpoints.take(&mut bytes).expect("taking succeeds") {
            TakeOutcome::Taken => Some(bytes.payload),
            TakeOutcome::Declined | TakeOutcome::Skipped | TakeOutcome::Empty => None,
        }
    }

    #[conformance_test]
    pub fn messages_reach_the_remote_endpoints_from_the_adapter<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints and the gateway's endpoints to match.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let mut own = adapter
            .publish_subscribe(remote.description())
            .expect("the adapter's endpoints open");

        // === PUBLISH ===
        assert_that!(remote.sync(TIMEOUT), eq true);

        // Serialize the value to the wire form of the topic.
        let value = remote.value(VALUE);
        let message = remote.encode(&value);
        publish(&mut own, &message);

        retry(
            || match remote.receive() {
                Some(received) if received == value => Ok(()),
                Some(_) => Err("an unexpected message arrived"),
                None => Err("no message arrived"),
            },
            TIMEOUT,
        )
        .expect("the message reaches the remote endpoints");
    }

    #[conformance_test]
    pub fn messages_reach_the_adapter_from_the_remote_endpoints<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints and the gateway's endpoints to match.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let mut own = adapter
            .publish_subscribe(remote.description())
            .expect("the adapter's endpoints open");

        // === SEND ===
        assert_that!(remote.sync(TIMEOUT), eq true);

        let value = remote.value(VALUE);
        remote.send(value.clone());

        // Deserialize the value from the wire form of the topic.
        retry(
            || match take(&mut own).map(|bytes| remote.decode(&bytes)) {
                Some(received) if received == value => Ok(()),
                Some(_) => Err("an unexpected message arrived"),
                None => Err("no message arrived"),
            },
            TIMEOUT,
        )
        .expect("the message reaches the adapter");
    }

    #[conformance_test]
    pub fn the_adapter_does_not_take_its_own_messages<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // The gateways endpoints and matching remote endpoints.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let mut own = adapter
            .publish_subscribe(remote.description())
            .expect("the adapter's endpoints open");

        // === PUBLISH ===
        // What the adapter publishes does not loop back.
        assert_that!(remote.sync(TIMEOUT), eq true);

        // Serialize the value to the wire form of the topic.
        let message = remote.encode(&remote.value(VALUE));
        publish(&mut own, &message);

        assert_that!(take(&mut own), is_none);
    }
}
