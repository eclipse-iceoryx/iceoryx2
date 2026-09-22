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
    use iceoryx2_link_adapter::{Adapter, PublishSubscribeEndpoints, ReceiveOutcome};

    use crate::testing::UnloanedBuffers;

    use crate::fixture::{AdapterFixture, MessageEndpoints};
    use crate::testing::retry;

    const TIMEOUT: Duration = Duration::from_secs(10);

    /// Takes the pending message of endpoints whose messages have no
    /// header, as the suites' remote endpoints send them.
    fn take<E: PublishSubscribeEndpoints>(endpoints: &mut E) -> Option<Vec<u8>> {
        match endpoints
            .take(UnloanedBuffers { header_size: 0 })
            .expect("taking succeeds")
        {
            ReceiveOutcome::Sample(taken) => Some(taken.payload),
            ReceiveOutcome::Skipped | ReceiveOutcome::Empty => None,
        }
    }

    #[conformance_test]
    pub fn messages_reach_the_remote_endpoints_from_the_adapter<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // Remote endpoints and the gateway's endpoints to match.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let mut endpoints = adapter
            .publish_subscribe(remote.description())
            .expect("endpoints open");

        // === PUBLISH ===
        assert_that!(remote.sync(TIMEOUT), eq true);
        endpoints
            .publish(&[], b"message")
            .expect("publishing succeeds");

        retry(
            || match remote.receive_message() {
                Some(message) if message == b"message" => Ok(()),
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
        let mut endpoints = adapter
            .publish_subscribe(remote.description())
            .expect("endpoints open");

        // === SEND ===
        assert_that!(remote.sync(TIMEOUT), eq true);
        remote.send_message(b"message");

        retry(
            || match take(&mut endpoints) {
                Some(message) if message == b"message" => Ok(()),
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
        // Remote endpoints and the gateway's endpoints to match.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let mut endpoints = adapter
            .publish_subscribe(remote.description())
            .expect("endpoints open");

        // === PUBLISH ===
        // What the adapter publishes goes out, never back in.
        assert_that!(remote.sync(TIMEOUT), eq true);
        endpoints
            .publish(&[], b"message")
            .expect("publishing succeeds");

        assert_that!(take(&mut endpoints), is_none);
    }
}
