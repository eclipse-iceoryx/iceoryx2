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
pub mod adapter_discovery {
    use alloc::vec::Vec;
    use core::time::Duration;

    use iceoryx2_bb_elementary::generation::Generation;
    use iceoryx2_bb_testing::assert_that;
    use iceoryx2_bb_testing_macros::conformance_test;
    use iceoryx2_link_adapter::Adapter;
    use iceoryx2_link_backend::service_description::Identified;

    use crate::fixture::{AdapterFixture, MessageEndpoints};
    use crate::testing::retry;

    const TIMEOUT: Duration = Duration::from_secs(10);

    /// The ids of the endpoints the adapter lists.
    fn listed<A: Adapter>(adapter: &mut A) -> Vec<<A::EndpointSettings as Identified>::Id> {
        let mut ids = Vec::new();
        adapter
            .endpoints(&mut |endpoint| ids.push(endpoint.settings.id()))
            .expect("listing succeeds");
        ids
    }

    #[conformance_test]
    pub fn remote_endpoints_are_listed<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // One adapter and remote endpoints on the middleware.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let id = remote.description().settings.id();

        // === LIST ===
        retry(
            || match listed(&mut adapter).as_slice() {
                [listed] if *listed == id => Ok(()),
                [] => Err("the endpoint is not listed"),
                _ => Err("an unexpected endpoint is listed"),
            },
            TIMEOUT,
        )
        .expect("the endpoint is listed");
    }

    #[conformance_test]
    pub fn the_generation_moves_on_endpoint_changes_and_holds_still_otherwise<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // One adapter, no endpoint yet.
        let mut adapter = fixture.adapter();

        // === OBSERVE ===
        // An adapter without a generation has nothing to hold to.
        let initial = adapter.generation();
        if initial == Generation::Untracked {
            return;
        }

        // Listing moves nothing.
        listed(&mut adapter);
        assert_that!(adapter.generation(), eq initial);

        // Remote endpoints arriving moves the generation.
        let _remote = fixture.remote_endpoints();
        retry(
            || match adapter.generation() != initial {
                true => Ok(()),
                false => Err("the generation has not moved"),
            },
            TIMEOUT,
        )
        .expect("the arrival moves the generation");

        // It holds still until the next change.
        let moved = adapter.generation();
        listed(&mut adapter);
        assert_that!(adapter.generation(), eq moved);
    }

    #[conformance_test]
    pub fn remote_endpoints_that_left_are_not_listed<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // One adapter listing the endpoint of remote endpoints.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        retry(
            || match listed(&mut adapter).is_empty() {
                false => Ok(()),
                true => Err("the endpoint is not listed"),
            },
            TIMEOUT,
        )
        .expect("the endpoint is listed");

        // === LEAVE ===
        // The application leaves and the endpoint with it.
        drop(remote);

        retry(
            || match listed(&mut adapter).is_empty() {
                true => Ok(()),
                false => Err("the endpoint is still listed"),
            },
            TIMEOUT,
        )
        .expect("the endpoint is no longer listed");
    }

    #[conformance_test]
    pub fn the_gateways_own_endpoints_alone_are_not_listed<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // An endpoint remote endpoints once were on, now held open only
        // by an adapter.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let description = remote.description().clone();
        drop(remote);
        let _endpoint = adapter
            .publish_subscribe(&description)
            .expect("endpoint opens");

        // === LIST ===
        // An adapter's own presence never makes an endpoint.
        assert_that!(listed(&mut adapter), len 0);
    }

    #[conformance_test]
    pub fn remote_endpoints_beside_the_gateways_are_listed_once<F: AdapterFixture>() {
        let mut fixture = F::new();

        // === SETUP ===
        // One remote endpoints and one adapter on the same endpoint.
        let mut adapter = fixture.adapter();
        let remote = fixture.remote_endpoints();
        let id = remote.description().settings.id();
        let _endpoint = adapter
            .publish_subscribe(remote.description())
            .expect("endpoint opens");

        // === LIST ===
        // The adapter neither hides the endpoint nor duplicates it.
        retry(
            || match listed(&mut adapter).as_slice() {
                [listed] if *listed == id => Ok(()),
                [] => Err("the endpoint is not listed"),
                _ => Err("the endpoint is listed more than once"),
            },
            TIMEOUT,
        )
        .expect("the endpoint is listed once");
    }
}
