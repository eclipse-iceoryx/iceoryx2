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
pub mod adapter_wake {
    use core::time::Duration;

    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_adapter::Adapter;
    use iceoryx2_link_backend::Reactive;

    use crate::fixture::{AdapterFixture, MessageEndpoints};
    use crate::testing::{WakeSource, retry};

    const TIMEOUT: Duration = Duration::from_secs(10);

    #[conformance_test]
    pub fn an_adapter_is_woken_by_remote_endpoints_joining<F: AdapterFixture>()
    where
        F::Adapter: Reactive,
    {
        let mut fixture = F::new();

        // === SETUP ===
        // An adapter waiting on a wake it attached.
        let mut adapter = fixture.adapter();
        let source = WakeSource::new();
        adapter.attach(source.wake.clone());
        assert_that_not_woken(&source);

        // === JOIN ===
        // Remote endpoints joining wakes the adapter.
        let _remote = fixture.remote_endpoints();

        retry(
            || match source.woken() {
                true => Ok(()),
                false => Err("the adapter was not woken"),
            },
            TIMEOUT,
        )
        .expect("the join wakes the adapter");
    }

    #[conformance_test]
    pub fn an_adapter_is_woken_by_a_message<F: AdapterFixture>()
    where
        F::Adapter: Reactive,
    {
        let mut fixture = F::new();

        // === SETUP ===
        // An adapter beside remote endpoints, waiting on its
        // wake after the join was heard.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let _endpoint = adapter
            .publish_subscribe(remote.description())
            .expect("endpoints open");
        let source = WakeSource::new();
        adapter.attach(source.wake.clone());
        assert_that_not_woken(&source);

        // === SEND ===
        // A message from the remote endpoints wakes the adapter.
        assert_that!(remote.sync(TIMEOUT), eq true);
        remote.send_message(b"message");

        retry(
            || match source.woken() {
                true => Ok(()),
                false => Err("the adapter was not woken"),
            },
            TIMEOUT,
        )
        .expect("the message wakes the adapter");
    }

    fn assert_that_not_woken(source: &WakeSource) {
        assert_that!(source.woken(), eq false);
    }
}
